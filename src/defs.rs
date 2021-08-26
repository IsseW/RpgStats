#![allow(dead_code)]
use bevy::sprite::TextureAtlasBuilder;
use crate::item::ToolProficiencies;
use bevy::app::EventWriter;
use bevy::ecs::system::{Commands, Res};
use bevy::prelude::AssetServer;
use bevy::utils::HashMap;
use bevy::utils::HashSet;
use bevy_console::PrintConsoleLine;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::{
    cmp::Ordering,
    fs::{read_dir, read_to_string},
};

use crate::item::{ToolPart, ToolProfeciency};

pub trait Definition {
    fn get_name(&self) -> &String;
    fn get_id(&self) -> usize;
    fn get_string_id(&self) -> String;
}

fn gen_id(namespace: &str, id: &str) -> String {
    format!("{}:{}", namespace, id)
}

trait StringKey {
    fn convert(string: &String) -> Self;
}

macro_rules! literal_key {
    ($ty:ty) => {
        impl StringKey for $ty {
            fn convert(string: &String) -> Self {
                serde_json::from_str(string.as_str()).unwrap()
            }
        }
    };
}

macro_rules! string_key {
    ($ty:ty) => {
        impl StringKey for $ty {
            fn convert(string: &String) -> Self {
                serde_json::from_str(format!("\"{}\"", string).as_str()).unwrap()
            }
        }
    };
}

literal_key!(u8);
literal_key!(i8);
literal_key!(u16);
literal_key!(i16);
literal_key!(u32);
literal_key!(i32);
literal_key!(u64);
literal_key!(i64);
literal_key!(u128);
literal_key!(i128);

string_key!(String);
string_key!(ToolPart);

trait ReferenceHolder {
    type Intermediate;
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Self::Intermediate;
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self;
}

impl<T: ReferenceHolder> ReferenceHolder for Vec<T> {
    type Intermediate = Vec<T::Intermediate>;
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Self::Intermediate {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|val| T::intermediate(builder, namespace, val))
            .collect()
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
        reference
            .iter()
            .map(|val| T::convert(builder, val))
            .collect()
    }
}

impl<K: std::cmp::Eq + Clone + std::hash::Hash + StringKey, V: ReferenceHolder> ReferenceHolder
    for HashMap<K, V>
{
    type Intermediate = HashMap<K, V::Intermediate>;
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Self::Intermediate {
        value
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (K::convert(key), V::intermediate(builder, namespace, value)))
            .collect()
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
        reference
            .iter()
            .map(|(key, val)| (key.clone(), V::convert(builder, val)))
            .collect()
    }
}

impl<T: ReferenceHolder + Default + Copy, const LENGTH: usize> ReferenceHolder for [T; LENGTH]
where
    T::Intermediate: Default + Copy,
{
    type Intermediate = [T::Intermediate; LENGTH];
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Self::Intermediate {
        let mut res = [Default::default(); LENGTH];
        if let Some(arr) = value.as_array() {
            if arr.len() == LENGTH {
                for i in 0..LENGTH {
                    res[i] = T::intermediate(builder, namespace, &arr[i]);
                }
            }
        }
        res
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
        let mut res = [Default::default(); LENGTH];
        for i in 0..LENGTH {
            res[i] = T::convert(builder, &reference[i]);
        }
        res
    }
}

macro_rules! ref_struct {
    ($ty:ident [$($field:ident: $field_ty:ty), *][$($ref_field:ident: $ref_field_ty:ty), *]) => {
        struct $ty {
            $($field: $field_ty,) *
            $($ref_field: $ref_field_ty,) *
        }
        paste! {
            #[derive(Debug, Serialize, Deserialize)]
            struct [< $ty Intermediate>] {
                $($field: $field_ty,) *
                $($ref_field: <$ref_field_ty as ReferenceHolder>::Intermediate,) *
            }

            impl ReferenceHolder for $ty {
                type Intermediate = [< $ty Intermediate>];

                fn intermediate(builder: &mut DefinitionBuilder, namespace: &String, value: &serde_json::Value) -> Self::Intermediate {
                    if let Some(obj) = value.as_object() {
                        [< $ty Intermediate>] {
                            $($field: serde_json::from_value(obj[stringify!($field)].clone()).unwrap(),) *
                            $($ref_field: <$ref_field_ty as ReferenceHolder>::intermediate(builder, namespace, &obj[stringify!($ref_field)]),) *
                        }
                    }
                    else {
                        panic!()
                    }
                }
                fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
                    Self {
                        $($field: reference.$field.clone(),) *
                        $($ref_field: <$ref_field_ty as ReferenceHolder>::convert(builder, &reference.$ref_field),) *
                    }
                }
            }
        }

    }
}

macro_rules! ref_enum {
    ($ty:ident [$($field:ident $(: $field_ty:ty)?), *][$($ref_field:ident: $ref_field_ty:ty), *]) => {
        enum $ty {
            $($field $(($field_ty))?,) *
            $($ref_field($ref_field_ty),) *
        }
        paste! {
            #[derive(Debug, Serialize, Deserialize)]
            enum [< $ty Intermediate>] {
                $($field $(($field_ty))?,) *
                $($ref_field(<$ref_field_ty as ReferenceHolder>::Intermediate),) *
            }

            impl ReferenceHolder for $ty {
                type Intermediate = [< $ty Intermediate>];

                fn intermediate(builder: &mut DefinitionBuilder, namespace: &String, value: &serde_json::Value) -> Self::Intermediate {
                    if let Some(obj) = value.as_object() {
                        for (key, val) in obj {
                            let t = match key.as_str() {
                                $(stringify!($field) => Self::Intermediate::$field $((serde_json::from_value::<$field_ty>(val.clone()).unwrap()))?,) *
                                $(stringify!($ref_field) => Self::Intermediate::$ref_field(<$ref_field_ty as ReferenceHolder>::intermediate(builder, namespace, val)),) *
                                _ => { continue; }
                            };
                            return t;
                        }
                        panic!()
                    }
                    else if let Some(string) = value.as_str() {
                        match string {
                            $(stringify!($field) => Self::Intermediate::$field $((<$field_ty as Default>::default()))?,) *
                            _ => panic!()
                        }
                    }
                    else {
                        panic!()
                    }
                }
                fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
                    match reference {
                        $(Self::Intermediate::$field $(([< __ $field_ty:lower >]))? => Self::$field $(([< __ $field_ty:lower >].clone()))?,) *
                        $(Self::Intermediate::$ref_field(data) => Self::$ref_field(<$ref_field_ty as ReferenceHolder>::convert(builder, &data)),) *
                    }
                }
            }
        }

    }
}

ref_enum! {
    SomeRef [Nothing, Something, Constant: f32, String: String][Texture: Texture]
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    fn from_str(s: &str) -> Option<Version> {
        let v: Vec<&str> = s.split('.').collect();
        if v.len() == 3 {
            Some(Self {
                major: u8::from_str_radix(v[0], 10).ok()?,
                minor: u8::from_str_radix(v[1], 10).ok()?,
                patch: u8::from_str_radix(v[2], 10).ok()?,
            })
        } else {
            None
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> std::string::String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.major < other.major {
            Some(Ordering::Less)
        } else if self.major > other.major {
            Some(Ordering::Greater)
        } else if self.minor < other.minor {
            Some(Ordering::Less)
        } else if self.minor > other.minor {
            Some(Ordering::Greater)
        } else if self.patch < other.patch {
            Some(Ordering::Less)
        } else if self.patch > other.patch {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}
#[derive(Debug)]
enum VersionConstraint {
    Exact(Version),
    Min(Version),
    Max(Version),
    Range(Version, Version),
    None,
}
impl Default for VersionConstraint {
    fn default() -> Self {
        Self::None
    }
}
impl VersionConstraint {
    fn is_within(&self, version: Version) -> bool {
        match self {
            Self::Exact(v) => *v == version,
            Self::Min(v) => *v <= version,
            Self::Max(v) => *v >= version,
            Self::Range(v1, v2) => *v1 <= version && *v2 >= version,
            Self::None => true,
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        if s.len() == 0 {
            return Some(Self::None);
        }
        if s.as_bytes()[0] as char == '>' {
            return Some(Self::Min(Version::from_str(&s[1..])?));
        }
        if s.as_bytes()[0] as char == '<' {
            return Some(Self::Max(Version::from_str(&s[1..])?));
        }
        let split: Vec<&str> = s.split("..").collect();
        if split.len() == 1 {
            return Some(Self::Exact(Version::from_str(split[0])?));
        } else if split.len() == 2 {
            return Some(Self::Range(
                Version::from_str(split[0])?,
                Version::from_str(split[1])?,
            ));
        }
        None
    }
}
#[derive(Debug, Default)]
struct Dependency {
    namespace: String,
    constraint: VersionConstraint,
    soft: bool,
}

enum ParseError {
    UndefinedNamespace,
    ExistingNamespace,
    ExpectedObject,
    ExpectedString,
    ExpectedArray,
    UnknownDefinition,
}

type Result<T> = std::result::Result<T, ParseError>;

macro_rules! definitions {
    ($($ty:ty [$($item:ident: $item_type:ty), *] $([$($cross_reference:ident: $cross_reference_type:ty), *])? $(hidden [$($hidden_item:ident: $hidden_item_ty:ty), *])? $(=> $on_done:expr)?), * $(,)?) => {

        paste! {

            $(
                type [< $ty sMapType >] = HashMap<String, ([< $ty DefinitionUnloaded >], usize, String)>;
                pub struct [< $ty Definition >] {
                    name: String,
                    namespace: String,
                    string_id: String,
                    id: usize,
                    $($item: $item_type,) *
                    $($($cross_reference: $cross_reference_type,) *)?
                    $($($hidden_item: $hidden_item_ty,) *)?
                }

                pub struct [< $ty s >] {
                    items: Vec<[< $ty Definition >]>,
                }

                impl IntoIterator for [< $ty s >] {
                    type Item = [< $ty Definition >];
                    type IntoIter = std::vec::IntoIter<Self::Item>;
                    fn into_iter(self) -> Self::IntoIter {
                        self.items.into_iter()
                    }
                }

                impl Index<$ty> for [< $ty s >] {
                    type Output = [< $ty Definition >];
                    fn index(&self, index: $ty) -> &Self::Output {
                        &self.items[index.0]
                    }
                }

                impl IndexMut<$ty> for [< $ty s >] {
                    fn index_mut(&mut self, index: $ty) -> &mut Self::Output {
                        &mut self.items[index.0]
                    }
                }

                #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
                pub struct $ty(usize);

                impl ReferenceHolder for $ty {
                    type Intermediate = String;
                    fn intermediate(builder: &mut DefinitionBuilder, namespace: &String, value: &serde_json::Value) -> Self::Intermediate {
                        builder.[< load_ $ty:lower _cross_ref >](namespace, value)
                    }
                    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Self {
                        builder.[< $ty:lower s >].get(reference).unwrap().1.into()
                    }
                }

                impl From<usize> for $ty {

                    fn from(data: usize) -> Self { Self(data) }
                }

                impl Definition for  [< $ty Definition >] {
                    fn get_name(&self) -> &std::string::String { &self.name }
                    fn get_id(&self) -> usize { self.id }
                    fn get_string_id(&self) -> String { gen_id(&self.namespace, &self.string_id) }
                }

                #[derive(Default, Deserialize, Debug)]
                struct [< $ty DefinitionUnloaded >] {
                    name: String,
                    id: String,
                    $($item: $item_type,) *
                    $($($cross_reference: <$cross_reference_type as ReferenceHolder>::Intermediate,) *)?
                }
            ) *

            #[derive(Default)]
            struct DefinitionBuilder {
                loaded_namespaces: HashMap<String, Version>,
                ids: HashMap<String, usize>,
                $(
                    [< $ty:lower s >]: [< $ty sMapType >],
                    [< $ty:lower _count >]: usize,
                ) *
            }

            impl DefinitionBuilder {

                fn load_namespaces(
                    &mut self, mut values: Vec<(PathBuf, serde_json::Value)>,
                    mut console_line: EventWriter<PrintConsoleLine>
                ) -> &mut Self {
                    // Preload the namespaces.
                    let mut mods = vec![];
                    for value in &values {
                        if let Some(obj) = value.1.as_object() {
                            if let Some(namespace) = obj["namespace"].as_str() {
                                if self.loaded_namespaces.contains_key(namespace) {
                                    console_line.send(PrintConsoleLine::new(format!("Error: Namespace {} has already been loaded.", namespace)));
                                }
                                else if let Some(version_str) = obj["version"].as_str() {
                                    if let Some(version) = Version::from_str(version_str) {

                                        let mut deps = vec![];
                                        if let Some(dependencies) = obj["deps"].as_object() {
                                            for (name, version) in dependencies {
                                                if let Some(version) = version.as_str() {
                                                    if let Some(version) = VersionConstraint::from_str(version) {
                                                        deps.push(Dependency {
                                                            namespace: name.clone(),
                                                            constraint: version,
                                                            soft: false,
                                                        });
                                                    }
                                                    else {
                                                        console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} with namespace {} version constraint for dependency {} is incorrectly formatted.", value.0.display(), namespace, name)));
                                                        continue;
                                                    }
                                                }
                                                else if let Some(version) = version.as_object() {
                                                    let soft = version["soft"].as_bool().unwrap_or(false);
                                                    if let Some(version) = version["version"].as_str() {
                                                        if let Some(version) = VersionConstraint::from_str(version) {
                                                            deps.push(Dependency {
                                                                namespace: name.clone(),
                                                                constraint: version,
                                                                soft: soft,
                                                            });
                                                        }
                                                        else {
                                                            console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} with namespace {} version constraint for dependency {} is incorrectly formatted.", value.0.display(), namespace, name)));
                                                            continue;
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        let defs = if let Some(defs) = obj["defs"].as_object() {
                                            defs
                                        } else {
                                            console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} with namespace {} defs value is incorrectly formatted.", value.0.display(), namespace)));
                                            continue;
                                        };

                                        mods.push((namespace.to_string(), deps, defs));
                                        self.loaded_namespaces.insert(namespace.to_string(), version);

                                    } else {
                                        console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} with namespace {} version value is incorrectly formatted.", value.0.display(), namespace)));
                                    }
                                } else {
                                    console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} with namespace {} is missing namespace key.", value.0.display(), namespace)));
                                }
                            }
                            else {
                                console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} is missing namespace key.", value.0.display())));
                            }
                        } else {
                            console_line.send(PrintConsoleLine::new(format!("Error: Mod in file {} is incorrectly formatted.", value.0.display())));
                        }
                    }

                    // Check dependencies.
                    'check: loop {
                        for i in 0..mods.len() {
                            for dep in &mods[i].1 {
                                let delete = {
                                    if let Some(version) = self.loaded_namespaces.get(&dep.namespace) {
                                        dep.constraint.is_within(*version)
                                    } else {
                                        dep.soft
                                    }
                                };
                                if delete {
                                    console_line.send(PrintConsoleLine::new(format!("Error: Stopped loading {}, because it does not have the required dependency {}.", mods[i].0, dep.namespace)));
                                    self.loaded_namespaces.remove(&dep.namespace);
                                    mods.swap_remove(i);
                                    continue 'check;
                                }
                            }
                        }
                        break;
                    }

                    // Load mods.
                    for module in mods {
                        if let Some(defs) = module.2["defs"].as_object() {
                            for (k, v) in defs {
                                if let Some(arr) = v.as_array() {
                                    match k.as_str() {
                                        $(stringify!([< $ty:lower s >]) => self.[< read_ $ty:lower _defs >](&module.0, arr),) *

                                        $(stringify!([< $ty:lower s_override >]) => self.[< read_ $ty:lower _overrides >](&module.0, arr),) *

                                        other => {
                                            console_line.send(PrintConsoleLine::new(format!("Warning: Unknown definition {} in mod {}.", other, module.0)));
                                        },
                                    }
                                } else {
                                    console_line.send(PrintConsoleLine::new(format!("Error: Expected array for definitions of type {} in mod {}.", k, module.0)));
                                }
                            }
                        }
                    }

                    self
                }

                fn build(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) {
                    $(
                        let mut [< $ty s_defs >] = [< $ty s >] {
                            items: self.[< $ty:lower s >].iter().map(|(k, (def, id, namespace))| {
                                [< $ty Definition >] {
                                    name: def.name.clone(),
                                    namespace: namespace.clone(),
                                    string_id: def.id.clone(),
                                    id: *id,
                                    $($item: def.$item.clone(),) *
                                    $($($cross_reference: $cross_reference_type::convert(self, &def.$cross_reference),) *)?
                                    $($($hidden_item: Default::default(),) *)?
                                }
                            }).collect()
                        };
                        fn [<on_done_ $ty:lower>](commands: &mut Commands, asset_server: &Res<AssetServer>, items: &mut [< $ty s >]) {
                            $(
                                let t = $on_done;
                                t(commands, asset_server, items);
                            )?
                        }
                        [<on_done_ $ty:lower>](commands, asset_server, &mut [< $ty s_defs >]);
                        commands.insert_resource([< $ty s_defs >]);
                    ) *
                }


                $(
                    fn [< read_ $ty:lower _overrides >](&mut self, namespace: &String, values: &Vec<serde_json::Value>) {
                        for value in values {
                            if let Some(obj) = value.as_object() {
                                let id = obj["id"].as_str().unwrap().into();
                                self.[< load_ $ty:lower _def >](namespace, &id, obj);
                            }
                        }
                    }

                    fn [< read_ $ty:lower _defs >](&mut self, namespace: &String, values: &Vec<serde_json::Value>) {
                        for value in values {
                            self.[< read_ $ty:lower _def >](namespace, value);
                        }
                    }

                    fn [< read_ $ty:lower _def >](&mut self, namespace: &String, value: &serde_json::Value) -> Option<String> {
                        if let Some(obj) = value.as_object() {
                            let mut def: [< $ty DefinitionUnloaded >] = Default::default();
                            def.id = obj["id"].as_str().unwrap().into();


                            let string_id = namespace.clone() + def.id.as_str();
                            let id = {
                                let temp = self.[< $ty:lower _count >];
                                self.[< $ty:lower _count >] += 1;
                                temp
                            };

                            self.[< $ty:lower s >].insert(string_id.clone(), (def, id, namespace.clone()));
                            self.[< load_ $ty:lower _def >](namespace, &string_id, &obj);

                            Some(string_id)
                        }
                        else {
                            None
                        }
                    }

                    fn [< load_ $ty:lower _def >](&mut self, namespace: &String, def: &String, obj: &serde_json::Map<String, serde_json::Value>) {

                        for (key, value) in obj {
                            match key.as_str() {
                                "name" => {
                                    let r = self.[< $ty:lower s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.name = value.as_str().unwrap().into();
                                    }
                                }
                                $(stringify!($item) => {
                                    let r = self.[< $ty:lower s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.$item = serde_json::from_value(value.clone()).unwrap();
                                    }
                                }) *
                                $($(stringify!($cross_reference) => {
                                    let value = $cross_reference_type::intermediate(self, namespace, value);
                                    let r = self.[< $ty:lower s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.$cross_reference = value;
                                    }
                                }) *)?

                                _ => {}
                            }
                        }
                    }

                    fn [< load_ $ty:lower _cross_ref >](&mut self, namespace: &String, value: &serde_json::Value) -> String {
                        if let Some(string) = value.as_str() {
                            if string.contains(':') {
                                string.into()
                            } else {
                                gen_id(namespace, string)
                            }
                        }
                        else {

                            self.[< read_ $ty:lower _def >](namespace, value).unwrap()
                        }
                    }
                ) *
            }
        }
    }
}

ref_struct! {
    ToolPartData [part: ToolPart, volume: f32][sprite: Sprite]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum TextureCrop {
    Full,
    Crop(f32, f32, f32, f32),
    Animated(f32, f32),
}

impl Default for TextureCrop {
    fn default() -> Self {
        Self::Full
    }
}
// Usage: 
// $Name[($member_name: $member_type)...] optional<[($cross_reference_name: $cross_reference_type)...]> optional<hidden [($hidden_member_name: $hidden_member_type)...]>
// optional<$lambda (&mut Commands, &Res<AssetServer>, &$Names)>

// First it takes normal members that are exposed in the json. Then there are crossreferences that are exposed in the json.
// Crossreferences are references to other things that are defined in json. hidden members are members that are not exposed in json.
// They should be calculated in the lambda. $Names refers to a resource of all definitions of this type.
definitions! {
    Material[tool_part_proficiency: HashMap<ToolPart, ToolProfeciency>, fuel_duration: Option<f32>, density: f32][sprite: Sprite, smelts_into:Liquid],
    Tool[proficiencies: ToolProficiencies][parts: Vec::<ToolPartData>],

    Tile[friction: f32][sprite: Sprite],
    Liquid[viscocity: f32][sprite: Sprite],

    BodyPart[],

    Species[][],

    Sprite[color: (u8, u8, u8), crop: TextureCrop][texture: Texture],
    Sfx[pitch: f32, volume: f32][sound: Sound],


    Texture[location: String] hidden [] => |commands, server: &Res<AssetServer>, definitions: &mut Textures| { 
        let builder = TextureAtlasBuilder::default();
        for def in definitions.into_iter() {
            builder.add_texture(server.load(def.location.as_str()));
        }
    },
    Sound[location: String],
}

fn init_definitions(
    commands: &mut Commands,
    console_line: EventWriter<PrintConsoleLine>,
    asset_server: Res<AssetServer>,
) {
    let paths = read_dir("./mods/").unwrap();

    let mods: Vec<(PathBuf, serde_json::Value)> = paths
        .filter_map(|path| {
            let path = path.ok()?.path();
            Some((
                path.clone(),
                serde_json::from_str(read_to_string(path).ok()?.as_str()).ok()?,
            ))
        })
        .collect();

    DefinitionBuilder::default()
        .load_namespaces(mods, console_line)
        .build(commands, &asset_server);
}
