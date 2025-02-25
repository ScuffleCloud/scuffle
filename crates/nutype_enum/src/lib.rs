//! The crate provides a macro to create a new enum type with a single field.
//!
//! ## Why do we need this?
//!
//! This is useful when you have a value and you want to have enum like behavior and have a catch all case for all other values.
//!
//! ## Examples
//!
//! ```rust
//! use nutype_enum::nutype_enum;
//!
//! nutype_enum! {
//!     pub enum AacPacketType(u8) {
//!         SeqHdr = 0x0,
//!         Raw = 0x1,
//!     }
//! }
//! ```
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// Helper macro to create a new enum type with a single field.
///
/// The enum type is derived with the `Clone`, `Copy`, `PartialEq`, `Eq`,
/// `PartialOrd`, `Ord`, and `Hash` traits. The nutype also impls `From` and
/// `Into` for the underlying type. As well as a custom `Debug` impl for human
/// readable output.
///
/// # Examples
///
/// ```rust
/// # use nutype_enum::nutype_enum;
/// nutype_enum! {
///     pub enum AacPacketType(u8) {
///         SeqHdr = 0x0,
///         Raw = 0x1,
///     }
/// }
/// ```
#[macro_export]
macro_rules! nutype_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident($type:ty) {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident = $value:expr
            ),*$(,)?
        }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $(#[$attr])*
        #[repr(transparent)]
        $vis struct $name(pub $type);

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        &$name::$variant => write!(f, "{}::{}", stringify!($name), stringify!($variant)),
                    )*
                    _ => write!(f, "{}({:?})", stringify!($name), self.0),
                }
            }
        }

        impl $name {
            $(
                $(#[$variant_attr])*
                #[allow(non_upper_case_globals)]
                pub const $variant: Self = Self($value);
            )*
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $type {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

/// Helper macro to create a bitwise enum.
///
/// The enum type is derived with the `BitAnd`, `BitOr`, `BitXor`, `BitAndAssign`,
/// `BitOrAssign`, and `BitXorAssign` traits.
///
/// # Examples
///
/// ```rust
/// # use nutype_enum::{nutype_enum, bitwise_enum};
/// nutype_enum! {
///     pub enum IoFlags(u8) {
///         Seek = 0x1,
///         Write = 0x2,
///         Read = 0x4,
///     }
/// }
///
/// bitwise_enum!(IoFlags);
/// ```
#[macro_export]
macro_rules! bitwise_enum {
    ($name:ident) => {
        impl ::std::ops::BitAnd for $name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl ::std::ops::BitXor for $name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl ::std::ops::Not for $name {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }

        impl ::std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl ::std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl ::std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
    };
}
