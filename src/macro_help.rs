#[macro_export]
macro_rules! count_idents {
    ($($idents:ident),* $(,)?) => {
        {
            #[derive(Copy, Clone)]
            #[allow(dead_code, non_camel_case_types)]

            enum Counter { $($idents,)* LastIdent }
            Counter::LastIdent as usize
        }
    };
}

pub mod useless {
    pub struct Dummy {}
    pub trait Trait<const SIZE: usize> {
        type Value;
    }
    seq_macro::seq!(N in 0..=8 {
        impl Trait<N> for Dummy {
            type Value = u8;
        }
    });

    seq_macro::seq!(N in 9..=16 {
        impl Trait<N> for Dummy {
            type Value = u16;
        }
    });
    seq_macro::seq!(N in 17..=32 {
        impl Trait<N> for Dummy {
            type Value = u32;
        }
    });
    seq_macro::seq!(N in 33..=64 {
        impl Trait<N> for Dummy {
            type Value = u64;
        }
    });
    seq_macro::seq!(N in 65..=128 {
        impl Trait<N> for Dummy {
            type Value = u128;
        }
    });
}

#[macro_export]
macro_rules! get_type {
    ($num:expr) => {
        <crate::macro_help::useless::Dummy as crate::macro_help::useless::Trait<$num>>::Value
    };
}

pub trait Bitmap {
    const FIELD_COUNT: usize;
    fn empty() -> Self;
    fn is_empty(&self) -> bool;
    fn is(&self, other: Self) -> bool;
}

#[macro_export]
macro_rules! bitmap_inner {
    ($ty:ty, $data:expr, $current_value:ident) => {
        pub const $current_value: $ty = $ty(1 << $data);
    };
    ($ty:ty, $data:expr, $current_value:ident, $($value:ident), +) => {
        crate::bitmap_inner!($ty, $data, $current_value);
        crate::bitmap_inner!($ty, ($data + 1), $($value), +);
    }
}

#[macro_export]
macro_rules! bitmap {
    ($acc:vis $name:ident: $ty:ty[$($value:ident), * $(,)?]) => {
        #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        $acc struct $name($ty);
        #[allow(non_upper_case_globals)]
        impl $name {
            crate::bitmap_inner!($name, 0, $($value), *);
        }

        impl crate::macro_help::Bitmap for $name {
            const FIELD_COUNT: usize = count_idents!($($value), *);
            fn empty() -> Self {
                Default::default()
            }
            fn is_empty(&self) -> bool {
                self.0 == 0
            }
            fn is(&self, other: Self) -> bool {
                !other.is_empty() && (other - *self).is_empty()
            }
        }
        impl std::ops::Not for $name {
            type Output = $name;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl std::ops::BitOr<Self> for $name {
            type Output = $name;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl std::ops::BitAnd<Self> for $name {
            type Output = $name;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }
        impl std::ops::BitXor<Self> for $name {
            type Output = $name;
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }
        impl std::ops::Sub<Self> for $name {
            type Output = $name;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 & !rhs.0)
            }
        }

        impl std::ops::BitOrAssign<Self> for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
        impl std::ops::BitAndAssign<Self> for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }
        impl std::ops::BitXorAssign<Self> for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
        impl std::ops::SubAssign<Self> for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 &= !rhs.0;
            }
        }
    };
    ($acc:vis $name:ident[$($value:ident), * $(,)?]) => {
        bitmap!($acc $name: crate::get_type!({$name::FIELD_COUNT})[$($value), *]);
    };

    ($($acc:vis $name:ident $(:$ty:ty)? [$($value:ident), * $(,)?]) *) => {
        $(bitmap!($acc $name $(:$ty)? [$($value), *]);) *
    }
}

#[macro_export]
macro_rules! match_bits {
    ($m:expr; $($($expr:expr), + => $block:block $(,)?) * $(,)? _ => $else_block:block $(,)?) => {
        match $m {
            $(t if $(t.is($expr) ||) + false => $block,) *
            _ => $else_block
        }
    };
    ($m:expr; $($($expr:expr), + => $block:block $(,)?) * $(,)?) => {
        match_bits!($m; $($($expr), + => $block) * _ => {})
    };
}

#[macro_export]
macro_rules! match_bits_all {
    ($m:expr; $($($expr:expr), + => $block:block $(,)?) * $(,)?) => {
        $(if $($m.is($expr) ||) + true $block) *
    };
}

#[macro_export]
macro_rules! serializable {
    ($i:item) => { #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)] $i };
    ($($i:item) +) => { $(serializable! { $i }) + };
}

#[macro_export]
macro_rules! cmap {
    ($($from:ident), + [$start:literal..$end:literal] $expr:expr) => {
        seq_macro::seq!( N in $start..$end {
            [
                #(
                    ($expr)($($from[N]), +),
                )*
            ]
        })
    };
    ($($from:ident), + [$start:literal..=$end:literal] $expr:expr) => {
        seq_macro::seq!( N in $start..=$end {
            [
                #(
                    ($expr)($($from[N]), +),
                )*
            ]
        })
    };
}
