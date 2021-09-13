#![allow(dead_code)]
use crate::item::ToolProficiencies;
use bevy::prelude::*;
use bevy::utils::HashMap;
use core::fmt::Display;
use paste::paste;
use seq_macro::seq;
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

trait StringKey
where
    Self: Sized,
{
    fn convert(string: &String) -> Option<Self>;
}

macro_rules! literal_key {
    ($ty:ty) => {
        impl StringKey for $ty {
            fn convert(string: &String) -> Option<Self> {
                serde_json::from_str(string.as_str()).ok()
            }
        }
    };
}

macro_rules! string_key {
    ($ty:ty) => {
        impl StringKey for $ty {
            fn convert(string: &String) -> Option<Self> {
                serde_json::from_str(format!("\"{}\"", string).as_str()).ok()
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

trait NonReference {}

impl<T: StringKey> NonReference for T {}
impl NonReference for f32 {}
impl NonReference for f64 {}

trait ReferenceHolder
where
    Self: Sized,
{
    type Intermediate;
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Option<Self::Intermediate>;
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self>;
}

impl<T: ReferenceHolder> ReferenceHolder for Vec<T> {
    type Intermediate = Vec<T::Intermediate>;
    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &String,
        value: &serde_json::Value,
    ) -> Option<Self::Intermediate> {
        value
            .as_array()?
            .iter()
            .map(|val| T::intermediate(builder, namespace, val))
            .collect()
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
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
    ) -> Option<Self::Intermediate> {
        value
            .as_object()?
            .iter()
            .map(|(key, value)| K::convert(key).zip(V::intermediate(builder, namespace, value)))
            .collect()
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
        reference
            .iter()
            .map(|(key, val)| Some(key.clone()).zip(V::convert(builder, val)))
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
    ) -> Option<Self::Intermediate> {
        if let Some(arr) = value.as_array() {
            if arr.len() == LENGTH {
                let mut res = [Default::default(); LENGTH];
                for i in 0..LENGTH {
                    res[i] = T::intermediate(builder, namespace, &arr[i])?;
                }
                Some(res)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
        let mut res = [Default::default(); LENGTH];
        for i in 0..LENGTH {
            res[i] = T::convert(builder, &reference[i])?;
        }
        Some(res)
    }
}

impl<T: ReferenceHolder> ReferenceHolder for Option<T> {
    type Intermediate = Option<T::Intermediate>;

    fn intermediate(
        builder: &mut DefinitionBuilder,
        namespace: &std::string::String,
        value: &serde_json::Value,
    ) -> Option<Self::Intermediate> {
        if let Some(string) = value.as_str() {
            match string {
                "None" => Some(None),
                _ => Some(T::intermediate(builder, namespace, value)),
            }
        } else if let Some(_) = value.as_null() {
            Some(None)
        } else {
            Some(T::intermediate(builder, namespace, value))
        }
    }

    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
        match reference {
            Some(reference) => Some(T::convert(builder, reference)),
            None => Some(None),
        }
    }
}

impl<'de, T: NonReference + Clone + serde::de::DeserializeOwned> ReferenceHolder for T {
    type Intermediate = T;

    fn intermediate(
        _builder: &mut DefinitionBuilder,
        _namespace: &String,
        value: &serde_json::Value,
    ) -> Option<Self::Intermediate> {
        serde_json::from_value(value.clone()).ok()
    }

    fn convert(_builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
        Some(reference.clone())
    }
}

macro_rules! ref_tuples {
    ($($num_elements:literal), *) => {
        $(
            seq!(N in 0..$num_elements {
                impl<#(T#N : ReferenceHolder,)*> ReferenceHolder for (#(T#N,)*) {
                    type Intermediate = (#(T#N::Intermediate,)*);

                    fn intermediate(
                        builder: &mut DefinitionBuilder,
                        namespace: &String,
                        value: &serde_json::Value,
                    ) -> Option<Self::Intermediate> {
                        if let Some(arr) = value.as_array() {
                            Some((
                                #(T#N::intermediate(builder, namespace, &arr[N])?,)*
                            ))
                        } else {
                            None
                        }
                    }

                    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
                        Some((
                                #(T#N::convert(builder, &reference.N)?,)*
                            ))
                    }
                }
            });
        ) *
    };
}

ref_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

macro_rules! ref_struct {
    ($ty:ident [$($field:ident: $field_ty:ty), *][$($ref_field:ident: $ref_field_ty:ty), *]) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
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

                fn intermediate(_builder: &mut DefinitionBuilder, _namespace: &String, value: &serde_json::Value) -> Option<Self::Intermediate> {
                    if let Some(obj) = value.as_object() {
                        Some([< $ty Intermediate>] {
                            $($field: serde_json::from_value(obj[stringify!($field)].clone()).ok()?,) *
                            $($ref_field: <$ref_field_ty as ReferenceHolder>::intermediate(_builder, _namespace, &obj[stringify!($ref_field)])?,) *
                        })
                    }
                    else {
                        None
                    }
                }
                fn convert(_builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
                    Some(Self {
                        $($field: reference.$field.clone(),) *
                        $($ref_field: <$ref_field_ty as ReferenceHolder>::convert(_builder, &reference.$ref_field)?,) *
                    })
                }
            }
        }

    }
}

macro_rules! ref_enum {
    ($ty:ident [$($field:ident $(: $field_ty:ty)?), *][$($ref_field:ident: $ref_field_ty:ty), *]) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
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

                fn intermediate(builder: &mut DefinitionBuilder, namespace: &String, value: &serde_json::Value) -> Option<Self::Intermediate> {
                    if let Some(obj) = value.as_object() {
                        for (key, val) in obj {
                            let t = match key.as_str() {
                                $(stringify!($field) => Self::Intermediate::$field $((serde_json::from_value::<$field_ty>(val.clone()).ok()?))?,) *
                                $(stringify!($ref_field) => Self::Intermediate::$ref_field(<$ref_field_ty as ReferenceHolder>::intermediate(builder, namespace, val)?),) *
                                _ => { continue; }
                            };
                            return Some(t);
                        }
                        None
                    }
                    else if let Some(string) = value.as_str() {
                        match string {
                            $(stringify!($field) => Some(Self::Intermediate::$field $((<$field_ty as Default>::default()))?),) *
                            _ => None
                        }
                    }
                    else {
                        None
                    }
                }
                fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
                    match reference {
                        $(Self::Intermediate::$field $(([< __ $field_ty:snake >]))? => Some(Self::$field $(([< __ $field_ty:snake >].clone()))?),) *
                        $(Self::Intermediate::$ref_field(data) => Some(Self::$ref_field(<$ref_field_ty as ReferenceHolder>::convert(builder, &data)?)),) *
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
    ($($ty:ty [$($item:ident: $item_type:ty), * $(,)?] $([$($cross_reference:ident: $cross_reference_type:ty), * $(,)?])? $(hidden [$($hidden_item:ident: $hidden_item_ty:ty), * $(,)?])? $(=> $on_done:expr)?), * $(,)?) => {

        paste! {

            $(
                type [< $ty sMapType >] = HashMap<String, ([< $ty DefinitionUnloaded >], usize, String)>;

                #[derive(Deserialize, Serialize, Debug, Clone)]
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

                impl [< $ty s >] {
                    pub fn add(&mut self, item: [< $ty Definition >]) {
                        self.items.push(item);
                    }

                    pub fn iter(&self) -> std::slice::Iter<[< $ty Definition >]> {
                        self.items.iter()
                    }

                    pub fn iter_mut(&mut self) -> std::slice::IterMut<[< $ty Definition >]> {
                        self.items.iter_mut()
                    }

                    pub fn next_id(&self) -> usize {
                        self.items.len()
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

                #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
                pub struct $ty(usize);

                impl ReferenceHolder for $ty {
                    type Intermediate = String;
                    fn intermediate(builder: &mut DefinitionBuilder, namespace: &String, value: &serde_json::Value) -> Option<Self::Intermediate> {
                        builder.[< load_ $ty:snake _cross_ref >](namespace, value)
                    }
                    fn convert(builder: &DefinitionBuilder, reference: &Self::Intermediate) -> Option<Self> {
                        Some(builder.[< $ty:snake s >].get(reference)?.1.into())
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
                $(
                    [< $ty:snake s >]: [< $ty sMapType >],
                    [< $ty:snake _count >]: usize,
                ) *
            }

            struct Mod {
                namespace: String,
                $([< $ty:snake s>]: HashMap<String, usize>), *
            }

            struct Mods {
                mods: HashMap<String, Mod>,
            }

            impl DefinitionBuilder {

                fn load_namespaces(
                    &mut self, values: Vec<(PathBuf, serde_json::Value)>,
                    printer: &mut EventWriter<Message>
                ) -> &mut Self {
                    // Preload the namespaces.
                    let mut mods = vec![];
                    for value in &values {
                        if let Some(obj) = value.1.as_object() {
                            if let Some(namespace) = obj["namespace"].as_str() {
                                if self.loaded_namespaces.contains_key(namespace) {
                                    printer.send(Message::error(format!("Namespace {} has already been loaded.", namespace)));
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
                                                        printer.send(Message::error(format!("Mod in file {} with namespace {} version constraint for dependency {} is incorrectly formatted.", value.0.display(), namespace, name)));
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
                                                            printer.send(Message::error(format!("Mod in file {} with namespace {} version constraint for dependency {} is incorrectly formatted.", value.0.display(), namespace, name)));
                                                            continue;
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        let defs = if let Some(defs) = obj["defs"].as_object() {
                                            defs
                                        } else {
                                            printer.send(Message::error(format!("Mod in file {} with namespace {} defs value is incorrectly formatted.", value.0.display(), namespace)));
                                            continue;
                                        };

                                        mods.push((namespace.to_string(), deps, defs));
                                        self.loaded_namespaces.insert(namespace.to_string(), version);

                                    } else {
                                        printer.send(Message::error(format!("Mod in file {} with namespace {} version value is incorrectly formatted.", value.0.display(), namespace)));
                                    }
                                } else {
                                    printer.send(Message::error(format!("Mod in file {} with namespace {} is missing namespace key.", value.0.display(), namespace)));
                                }
                            }
                            else {
                                printer.send(Message::error(format!("Mod in file {} is missing namespace key.", value.0.display())));
                            }
                        } else {
                            printer.send(Message::error(format!("Mod in file {} is incorrectly formatted.", value.0.display())));
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
                                    printer.send(Message::error(format!("Stopped loading {}, because it does not have the required dependency {}.", mods[i].0, dep.namespace)));
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
                                    if match k.as_str() {
                                        $(stringify!([< $ty:snake s >]) => self.[< read_ $ty:snake _defs >](&module.0, arr),) *

                                        $(stringify!([< $ty:snake s_override >]) => self.[< read_ $ty:snake _overrides >](&module.0, arr),) *

                                        other => {
                                            printer.send(Message::warning(format!("Unknown definition {} in mod {}.", other, module.0)));
                                            Some(())
                                        },
                                    }.is_none() {
                                        printer.send(Message::error(format!("In definition {} in mod {}.", k.as_str(), module.0)));
                                    }
                                } else {
                                    printer.send(Message::error(format!("Expected array for definitions of type {} in mod {}.", k, module.0)));
                                }
                            }
                        }
                    }

                    self
                }

                fn build(&self, commands: &mut Commands) -> Option<()> {
                    $(
                        #[allow(unused_mut)]
                        let mut [< $ty:snake s_defs >] = [< $ty s >] {
                                items: self.[< $ty:snake s >].iter().map(|(_k, (def, id, namespace))| {
                                    Some([< $ty Definition >] {
                                        name: def.name.clone(),
                                        namespace: namespace.clone(),
                                        string_id: def.id.clone(),
                                        id: *id,
                                        $($item: def.$item.clone(),) *
                                        $($($cross_reference: <$cross_reference_type>::convert(self, &def.$cross_reference)?,) *)?
                                        $($($hidden_item: Default::default(),) *)?
                                    })
                                }).collect::<Option<Vec<[< $ty Definition >]>>>()?
                            };
                        $($on_done(&mut [< $ty:snake s_defs >]);)?
                        commands.insert_resource([< $ty:snake s_defs >]);
                    ) *
                    Some(())
                }


                $(
                    fn [< read_ $ty:snake _overrides >](&mut self, namespace: &String, values: &Vec<serde_json::Value>) -> Option<()> {
                        for value in values {
                            if let Some(obj) = value.as_object() {
                                let id = obj["id"].as_str()?.into();
                                self.[< load_ $ty:snake _def >](namespace, &id, obj)?;
                            }
                            else {
                                return None;
                            }
                        }
                        Some(())
                    }

                    fn [< read_ $ty:snake _defs >](&mut self, namespace: &String, values: &Vec<serde_json::Value>) -> Option<()> {
                        for value in values {
                            self.[< read_ $ty:snake _def >](namespace, value)?;
                        }
                        Some(())
                    }

                    fn [< read_ $ty:snake _def >](&mut self, namespace: &String, value: &serde_json::Value) -> Option<String> {
                        if let Some(obj) = value.as_object() {
                            let mut def: [< $ty DefinitionUnloaded >] = Default::default();
                            def.id = obj["id"].as_str()?.into();


                            let string_id = namespace.clone() + def.id.as_str();
                            let id = {
                                let temp = self.[< $ty:snake _count >];
                                self.[< $ty:snake _count >] += 1;
                                temp
                            };

                            self.[< $ty:snake s >].insert(string_id.clone(), (def, id, namespace.clone()));
                            self.[< load_ $ty:snake _def >](namespace, &string_id, &obj)?;

                            Some(string_id)
                        }
                        else {
                            None
                        }
                    }

                    #[allow(unused_variables)]
                    fn [< load_ $ty:snake _def >](&mut self, namespace: &String, def: &String, obj: &serde_json::Map<String, serde_json::Value>) -> Option<()> {

                        for (key, value) in obj {
                            match key.as_str() {
                                "name" => {
                                    let r = self.[< $ty:snake s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.name = value.as_str().unwrap().into();
                                    }
                                }
                                $(stringify!($item) => {
                                    let r = self.[< $ty:snake s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.$item = serde_json::from_value(value.clone()).unwrap();
                                    }
                                }) *
                                $($(stringify!($cross_reference) => {
                                    let value = <$cross_reference_type>::intermediate(self, namespace, value);
                                    let r = self.[< $ty:snake s >].get_mut(def);
                                    if let Some(r) = r {
                                        r.0.$cross_reference = value?;
                                    }
                                }) *)?

                                _ => {}
                            }
                        }
                        Some(())
                    }

                    fn [< load_ $ty:snake _cross_ref >](&mut self, namespace: &String, value: &serde_json::Value) -> Option<String> {
                        if let Some(string) = value.as_str() {
                            if string.contains(':') {
                                Some(string.into())
                            } else {
                                Some(gen_id(namespace, string))
                            }
                        }
                        else {
                            self.[< read_ $ty:snake _def >](namespace, value)
                        }
                    }
                ) *
            }

            struct DefinitionBinary {
                data: Vec<u8>,
            }

            pub fn generate_binary(
                mut commands: Commands,
                $([< $ty:snake s>]: Res<[< $ty s>]>), *
            ) {
                let mut obj = serde_json::Map::<String, serde_json::Value>::default();
                $(
                    obj.insert(stringify!([< $ty:snake s>]).to_string(), serde_json::to_value([< $ty:snake s>].items.clone()).unwrap());
                ) *
                let mut data: Vec<u8> = vec![];
                serde_cbor::to_writer(&mut data, &obj).expect("Failed to write json object to byte vector. Maybe low on ram?");
                commands.insert_resource(DefinitionBinary { data });
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
ref_struct! {
    OreData[min_drop: usize, max_drop: usize][material: Material]
}

ref_enum! {
    BlockType[Stone: bool][Ore: OreData, Wood: Sprite]
}

enum MeterialType {
    Metal,
    Mineral,
    Wood,
}

ref_struct! {
    Alloying[min_temp: f32, max_temp: f32][]
}

ref_enum! {
    RecipeType
    []
    [
        Alloying: Alloying
    ]
}

// Usage:
// $Name[($member_name: $member_type)...] optional<[($cross_reference_name: $cross_reference_type)...]> optional<hidden [($hidden_member_name: $hidden_member_type)...]>
// optional<$lambda (&mut Commands, &Res<AssetServer>, &$Names)>

// First it takes normal members that are exposed in the json. Then there are crossreferences that are exposed in the json.
// Crossreferences are references to other things that are defined in json. hidden members are members that are not exposed in json.
// They should be calculated in the lambda. $Names refers to a resource of all definitions of this type.
definitions! {
    Material[
            tool_part_proficiency: Option<HashMap<ToolPart, ToolProfeciency>>,
            fuel_duration: Option<f32>,
            density: f32,
            formable: bool,
        ]
        [
            sprite: Sprite,
            block_sprite: Option::<Sprite>,
            smelts_into: Option::<Liquid>,
        ],
    Tool[proficiencies: ToolProficiencies][parts: Vec::<ToolPartData>],

    Model[faces: Vec<(f32, f32, f32, f32)>],

    Block[friction: f32][sprite: Sprite],
    Liquid[viscocity: f32][sprite: Sprite],
    Recipe[],

    BodyPart[],

    Species[][],

    Sprite[color: (u8, u8, u8), crop: TextureCrop][texture: (Texture, u8, f32, Sound, (BodyPart, Species))],
    Sfx[pitch: f32, volume: f32][sound: Sound],


    Texture[location: String],
    Sound[location: String],
}

pub enum MessageType {
    Error,
    Warning,
    Info,
    Message,
}
pub struct Message {
    ty: MessageType,
    message: String,
}

impl Message {
    fn error(message: String) -> Self {
        Self {
            ty: MessageType::Error,
            message,
        }
    }
    fn warning(message: String) -> Self {
        Self {
            ty: MessageType::Warning,
            message: message,
        }
    }
}

impl Display for Message {
    fn fmt(&self, fm: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.ty {
            MessageType::Error => write!(fm, "Error: "),
            MessageType::Warning => write!(fm, "Warning: "),
            MessageType::Info => write!(fm, "Info: "),
            _ => Ok(()),
        }?;
        write!(fm, "{}", self.message)
    }
}

fn init_definitions(mut commands: Commands, mut printer: EventWriter<Message>) {
    if let Ok(paths) = read_dir("./mods/") {
        let mods: Vec<(PathBuf, serde_json::Value)> = paths
            .filter_map(|path| {
                let path = path.ok()?.path();
                Some((
                    path.clone(),
                    serde_json::from_str(read_to_string(path).ok()?.as_str()).ok()?,
                ))
            })
            .collect();

        if DefinitionBuilder::default()
            .load_namespaces(mods, &mut printer)
            .build(&mut commands)
            .is_none()
        {
            printer.send(Message::error(
                "Error when building. Probably because of missing/misspelled definition".into(),
            ));
        }
    } else {
        printer.send(Message::error("Unable to find mod folder".into()));
        DefinitionBuilder::default().build(&mut commands);
    }
}

fn generate_blocks(
    mut blocks: ResMut<Blocks>,
    mut recipes: ResMut<Recipes>,
    materials: Res<Materials>,
    liquids: Res<Liquids>,
) {
    for liquid in liquids.iter() {
        let id = blocks.next_id();
        blocks.add(BlockDefinition {
            id,
            name: liquid.name.clone(),
            string_id: format!("liquid_{}", liquid.id),
            namespace: liquid.namespace.clone(),
            friction: liquid.viscocity,
            sprite: liquid.sprite,
        })
    }
    for material in materials.iter() {}
}

pub struct Definitions;

impl Plugin for Definitions {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_stage(
            "init",
            SystemStage::single_threaded().with_system(init_definitions.system()),
        )
        .add_startup_stage_after(
            "init",
            "generate",
            SystemStage::parallel().with_system(generate_blocks.system()),
        )
        .add_startup_stage_after(
            "generate",
            "binary",
            SystemStage::single_threaded().with_system(generate_binary.system()),
        );
    }
}
