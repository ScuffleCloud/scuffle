use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;

use bytes::Bytes;
use num_traits::ToPrimitive;

use super::{FuncFmt, Map};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CelError<'a> {
    #[error("index out of bounds: {0} is out of range for a list of length {1}")]
    IndexOutOfBounds(usize, usize),
    #[error("invalid type for indexing: {0}")]
    IndexWithBadIndex(CelValue<'a>),
    #[error("map key not found: {0:?}")]
    MapKeyNotFound(CelValue<'a>),
    #[error("bad operation: {left} {op} {right}")]
    BadOperation {
        left: CelValue<'a>,
        right: CelValue<'a>,
        op: &'static str,
    },
    #[error("bad unary operation: {op}{value}")]
    BadUnaryOperation { op: &'static str, value: CelValue<'a> },
    #[error("number out of range when performing {op}")]
    NumberOutOfRange { op: &'static str },
    #[error("bad access when trying to member {member} on {container}")]
    BadAccess { member: CelValue<'a>, container: CelValue<'a> },
}

#[derive(Clone, Debug)]
pub enum CelValue<'a> {
    Bool(bool),
    Number(NumberTy),
    String(Arc<str>),
    StringRef(&'a str),
    Bytes(Bytes),
    BytesRef(&'a [u8]),
    List(Arc<[CelValue<'a>]>),
    Map(Arc<[(CelValue<'a>, CelValue<'a>)]>),
    Duration(chrono::Duration),
    Timestamp(chrono::DateTime<chrono::FixedOffset>),
    Null,
}

impl PartialOrd for CelValue<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (CelValue::Number(l), CelValue::Number(r)) => Some(l.partial_cmp(r)?),
            (
                CelValue::String(_) | CelValue::Bytes(_) | CelValue::BytesRef(_) | CelValue::StringRef(_),
                CelValue::String(_) | CelValue::Bytes(_) | CelValue::BytesRef(_) | CelValue::StringRef(_),
            ) => {
                let l = match self {
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::BytesRef(b) => b,
                    CelValue::StringRef(s) => s.as_bytes(),
                    _ => unreachable!(),
                };

                let r = match other {
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::BytesRef(b) => b,
                    CelValue::StringRef(s) => s.as_bytes(),
                    _ => unreachable!(),
                };

                Some(l.cmp(r))
            }
            (CelValue::List(l), CelValue::List(r)) => Some(l.partial_cmp(r)?),
            (CelValue::Map(l), CelValue::Map(r)) => Some(l.partial_cmp(r)?),
            _ => None,
        }
    }
}

impl<'a> CelValue<'a> {
    pub fn access(&self, key: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        let key = key.conv();
        match self {
            CelValue::Map(map) => map
                .iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| v.clone())
                .ok_or(CelError::MapKeyNotFound(key)),
            CelValue::List(list) => {
                if let Some(idx) = key.as_number().and_then(|n| n.to_usize()) {
                    list.get(idx).cloned().ok_or(CelError::IndexOutOfBounds(idx, list.len()))
                } else {
                    Err(CelError::IndexWithBadIndex(key))
                }
            }
            _ => Err(CelError::BadAccess {
                member: key,
                container: self.clone(),
            }),
        }
    }

    pub fn cel_add(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_add(r)?)),
            (CelValue::String(l), CelValue::String(r)) => Ok(CelValue::String(Arc::from(format!("{l}{r}")))),
            (CelValue::Bytes(l), CelValue::Bytes(r)) => Ok(CelValue::Bytes({
                let mut l = l.to_vec();
                l.extend_from_slice(&r);
                Bytes::from(l)
            })),
            (CelValue::List(l), CelValue::List(r)) => Ok(CelValue::List(l.iter().chain(r.iter()).cloned().collect())),
            (CelValue::Map(l), CelValue::Map(r)) => Ok(CelValue::Map(l.iter().chain(r.iter()).cloned().collect())),
            (left, right) => Err(CelError::BadOperation { left, right, op: "+" }),
        }
    }

    pub fn cel_sub(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_sub(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "-" }),
        }
    }

    pub fn cel_mul(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_mul(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "*" }),
        }
    }

    pub fn cel_div(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_div(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "/" }),
        }
    }

    pub fn cel_rem(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_rem(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "%" }),
        }
    }

    fn as_number(&self) -> Option<NumberTy> {
        match self {
            CelValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    // !self
    pub fn cel_neg(self) -> Result<CelValue<'a>, CelError<'a>> {
        match self {
            CelValue::Number(n) => Ok(CelValue::Number(n.cel_neg()?)),
            _ => Err(CelError::BadUnaryOperation { value: self, op: "-" }),
        }
    }

    // left < right
    pub fn cel_lt(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: "<" })
            .map(|o| matches!(o, std::cmp::Ordering::Less))
    }

    // left <= right
    pub fn cel_lte(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: "<=" })
            .map(|o| matches!(o, std::cmp::Ordering::Less | std::cmp::Ordering::Equal))
    }

    // left > right
    pub fn cel_gt(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: ">" })
            .map(|o| matches!(o, std::cmp::Ordering::Greater))
    }

    // left >= right
    pub fn cel_gte(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: ">=" })
            .map(|o| matches!(o, std::cmp::Ordering::Greater | std::cmp::Ordering::Equal))
    }

    // left == right
    pub fn cel_eq(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: "==" })
            .map(|o| matches!(o, std::cmp::Ordering::Equal))
    }

    // left != right
    pub fn cel_ne(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();

        left.partial_cmp(&right)
            .ok_or(CelError::BadOperation { left, right, op: "!=" })
            .map(|o| matches!(o, std::cmp::Ordering::Less | std::cmp::Ordering::Greater))
    }

    // left contains right
    pub fn cel_contains(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        Self::cel_contained_by(right, left).map_err(|err| match err {
            CelError::BadOperation { left, right, op: "in" } => CelError::BadOperation {
                left: right,
                right: left,
                op: "contains",
            },
            err => err,
        })
    }

    // left in right
    pub fn cel_contained_by(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (left, CelValue::List(r)) => Ok(r.contains(&left)),
            (left, CelValue::Map(r)) => Ok(r.iter().any(|(k, _)| k == &left)),
            (
                left @ (CelValue::Bytes(_) | CelValue::BytesRef(_) | CelValue::String(_) | CelValue::StringRef(_)),
                right @ (CelValue::Bytes(_) | CelValue::BytesRef(_) | CelValue::String(_) | CelValue::StringRef(_)),
            ) => {
                let r = match &right {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::BytesRef(b) => b,
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::StringRef(s) => s.as_bytes(),
                    _ => unreachable!(),
                };

                let l = match &left {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::BytesRef(b) => b,
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::StringRef(s) => s.as_bytes(),
                    _ => unreachable!(),
                };

                Ok(r.windows(l.len()).any(|w| w == l))
            }
            (left, right) => Err(CelError::BadOperation { left, right, op: "in" }),
        }
    }

    pub fn cel_size(item: impl CelValueConv<'a>) -> Result<u64, CelError<'a>> {
        match item.conv() {
            Self::Bytes(b) => Ok(b.len() as u64),
            Self::BytesRef(b) => Ok(b.len() as u64),
            Self::String(s) => Ok(s.len() as u64),
            Self::StringRef(s) => Ok(s.len() as u64),
            Self::List(l) => Ok(l.len() as u64),
            Self::Map(m) => Ok(m.len() as u64),
            item => Err(CelError::BadUnaryOperation { op: "size", value: item }),
        }
    }
}

impl PartialEq for CelValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CelValue::Bool(l), CelValue::Bool(r)) => l == r,
            (CelValue::Number(l), CelValue::Number(r)) => l == r,
            (CelValue::String(_) | CelValue::StringRef(_), CelValue::String(_) | CelValue::StringRef(_)) => {
                let l = match self {
                    CelValue::String(s) => &**s,
                    CelValue::StringRef(s) => &**s,
                    _ => unreachable!(),
                };
                let r = match self {
                    CelValue::String(s) => &**s,
                    CelValue::StringRef(s) => &**s,
                    _ => unreachable!(),
                };
                l == r
            }
            (CelValue::Bytes(_) | CelValue::BytesRef(_), CelValue::Bytes(_) | CelValue::BytesRef(_)) => {
                let l = match self {
                    CelValue::Bytes(b) => &**b,
                    CelValue::BytesRef(b) => &**b,
                    _ => unreachable!(),
                };
                let r = match self {
                    CelValue::Bytes(b) => &**b,
                    CelValue::BytesRef(b) => &**b,
                    _ => unreachable!(),
                };
                l == r
            }
            (CelValue::List(l), CelValue::List(r)) => l == r,
            (CelValue::Map(l), CelValue::Map(r)) => l == r,
            _ => false,
        }
    }
}

pub trait CelValueConv<'a> {
    fn conv(self) -> CelValue<'a>;
}

impl<'a, C> CelValueConv<'a> for Option<C>
where
    C: CelValueConv<'a>,
{
    fn conv(self) -> CelValue<'a> {
        match self {
            Some(value) => CelValueConv::conv(value),
            None => CelValue::Null,
        }
    }
}

impl CelValueConv<'_> for () {
    fn conv(self) -> CelValue<'static> {
        CelValue::Null
    }
}

impl CelValueConv<'_> for bool {
    fn conv(self) -> CelValue<'static> {
        CelValue::Bool(self)
    }
}

impl CelValueConv<'_> for i32 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::I64(self as i64))
    }
}

impl CelValueConv<'_> for u32 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::U64(self as u64))
    }
}

impl CelValueConv<'_> for i64 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::I64(self))
    }
}

impl CelValueConv<'_> for u64 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::U64(self))
    }
}

impl CelValueConv<'_> for f32 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::F64(self as f64))
    }
}

impl CelValueConv<'_> for f64 {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(NumberTy::F64(self))
    }
}

impl<'a> CelValueConv<'a> for &'a str {
    fn conv(self) -> CelValue<'a> {
        CelValue::StringRef(self)
    }
}

impl CelValueConv<'_> for Bytes {
    fn conv(self) -> CelValue<'static> {
        CelValue::Bytes(self.clone())
    }
}

impl<'a> CelValueConv<'a> for &'a [u8] {
    fn conv(self) -> CelValue<'a> {
        CelValue::BytesRef(self)
    }
}

impl<'a, const N: usize> CelValueConv<'a> for &'a [u8; N] {
    fn conv(self) -> CelValue<'a> {
        (self as &[u8]).conv()
    }
}

impl<'a> CelValueConv<'a> for &'a Vec<u8> {
    fn conv(self) -> CelValue<'a> {
        CelValue::BytesRef(self)
    }
}

impl<'a, T> CelValueConv<'a> for &'a [T]
where
    &'a T: CelValueConv<'a>,
{
    fn conv(self) -> CelValue<'a> {
        CelValue::List(self.iter().map(CelValueConv::conv).collect())
    }
}

impl<'a, T, const N: usize> CelValueConv<'a> for &'a [T; N]
where
    &'a T: CelValueConv<'a>,
{
    fn conv(self) -> CelValue<'a> {
        (self as &[T]).conv()
    }
}

impl<'a, T> CelValueConv<'a> for &'a Vec<T>
where
    &'a T: CelValueConv<'a>,
{
    fn conv(self) -> CelValue<'a> {
        self.as_slice().conv()
    }
}

impl<'a> CelValueConv<'a> for &'a String {
    fn conv(self) -> CelValue<'a> {
        self.as_str().conv()
    }
}

impl<'a, T> CelValueConv<'a> for &T
where
    T: CelValueConv<'a> + Copy,
{
    fn conv(self) -> CelValue<'a> {
        CelValueConv::conv(*self)
    }
}

impl std::fmt::Display for CelValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CelValue::Bool(b) => std::fmt::Display::fmt(b, f),
            CelValue::Number(n) => std::fmt::Display::fmt(n, f),
            CelValue::String(s) => std::fmt::Display::fmt(s, f),
            CelValue::StringRef(s) => std::fmt::Display::fmt(s, f),
            CelValue::Bytes(b) => std::fmt::Debug::fmt(b, f),
            CelValue::BytesRef(b) => std::fmt::Debug::fmt(b, f),
            CelValue::List(l) => {
                let mut list = f.debug_list();
                for item in l.iter() {
                    list.entry(&FuncFmt(|fmt| item.fmt(fmt)));
                }
                list.finish()
            }
            CelValue::Map(m) => {
                let mut map = f.debug_map();
                for (key, value) in m.iter() {
                    map.entry(&FuncFmt(|fmt| key.fmt(fmt)), &FuncFmt(|fmt| value.fmt(fmt)));
                }
                map.finish()
            }
            CelValue::Null => std::fmt::Display::fmt("null", f),
            CelValue::Duration(d) => std::fmt::Display::fmt(d, f),
            CelValue::Timestamp(t) => std::fmt::Display::fmt(t, f),
        }
    }
}

impl CelValue<'_> {
    pub fn to_bool(&self) -> bool {
        match self {
            CelValue::Bool(b) => *b,
            CelValue::Number(n) => *n != 0,
            CelValue::String(s) => !s.is_empty(),
            CelValue::StringRef(s) => !s.is_empty(),
            CelValue::Bytes(b) => !b.is_empty(),
            CelValue::BytesRef(b) => !b.is_empty(),
            CelValue::List(l) => !l.is_empty(),
            CelValue::Map(m) => !m.is_empty(),
            CelValue::Null => false,
            CelValue::Duration(d) => !d.is_zero(),
            CelValue::Timestamp(t) => t.timestamp_nanos_opt().unwrap_or_default() != 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NumberTy {
    I64(i64),
    U64(u64),
    F64(f64),
}

impl PartialOrd for NumberTy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        NumberTy::promote(*self, *other).and_then(|(l, r)| match (l, r) {
            (NumberTy::I64(l), NumberTy::I64(r)) => Some(l.cmp(&r)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Some(l.cmp(&r)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Some(l.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal)),
            _ => None,
        })
    }
}

impl NumberTy {
    pub fn cel_add(self, other: Self) -> Result<Self, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "addition" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_add(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_add(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l + r)),
            _ => Err(ERROR),
        }
    }

    pub fn cel_sub(self, other: Self) -> Result<Self, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "subtraction" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_sub(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_sub(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l - r)),
            _ => Err(ERROR),
        }
    }

    pub fn cel_mul(self, other: Self) -> Result<Self, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "multiplication" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_mul(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_mul(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l * r)),
            _ => Err(ERROR),
        }
    }

    pub fn cel_div(self, other: Self) -> Result<Self, CelError<'static>> {
        if other == 0 {
            return Err(CelError::NumberOutOfRange { op: "division by zero" });
        }

        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "division" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_div(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_div(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l / r)),
            _ => Err(ERROR),
        }
    }

    pub fn cel_rem(self, other: Self) -> Result<Self, CelError<'static>> {
        if other == 0 {
            return Err(CelError::NumberOutOfRange { op: "remainder by zero" });
        }

        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "remainder" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_rem(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_rem(r).ok_or(ERROR)?)),
            _ => Err(ERROR),
        }
    }

    pub fn cel_neg(self) -> Result<NumberTy, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "negation" };
        match self {
            NumberTy::I64(n) => Ok(NumberTy::I64(n.checked_neg().ok_or(ERROR)?)),
            NumberTy::U64(n) => Ok(NumberTy::I64(n.to_i64().ok_or(ERROR)?.checked_neg().ok_or(ERROR)?)),
            NumberTy::F64(n) => Ok(NumberTy::F64(-n)),
        }
    }
}

impl std::fmt::Display for NumberTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberTy::I64(n) => std::fmt::Display::fmt(n, f),
            NumberTy::U64(n) => std::fmt::Display::fmt(n, f),
            NumberTy::F64(n) => std::fmt::Display::fmt(n, f),
        }
    }
}

impl PartialEq for NumberTy {
    fn eq(&self, other: &Self) -> bool {
        NumberTy::promote(*self, *other)
            .map(|(l, r)| match (l, r) {
                (NumberTy::I64(l), NumberTy::I64(r)) => l == r,
                (NumberTy::U64(l), NumberTy::U64(r)) => l == r,
                (NumberTy::F64(l), NumberTy::F64(r)) => l == r,
                _ => false,
            })
            .unwrap_or(false)
    }
}

macro_rules! impl_eq_number {
    ($ty:ty) => {
        impl PartialEq<$ty> for NumberTy {
            fn eq(&self, other: &$ty) -> bool {
                NumberTy::from(*other) == *self
            }
        }

        impl PartialEq<NumberTy> for $ty {
            fn eq(&self, other: &NumberTy) -> bool {
                other == self
            }
        }
    };
}

impl_eq_number!(i32);
impl_eq_number!(u32);
impl_eq_number!(i64);
impl_eq_number!(u64);
impl_eq_number!(f64);

impl From<i32> for NumberTy {
    fn from(value: i32) -> Self {
        Self::I64(value as i64)
    }
}

impl From<u32> for NumberTy {
    fn from(value: u32) -> Self {
        Self::U64(value as u64)
    }
}

impl From<i64> for NumberTy {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for NumberTy {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f64> for NumberTy {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<f32> for NumberTy {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl CelValueConv<'_> for NumberTy {
    fn conv(self) -> CelValue<'static> {
        CelValue::Number(self)
    }
}

impl<'a> CelValueConv<'a> for CelValue<'a> {
    fn conv(self) -> CelValue<'a> {
        self
    }
}

macro_rules! impl_to_primitive_number {
    ($fn:ident, $ty:ty) => {
        fn $fn(&self) -> Option<$ty> {
            match self {
                NumberTy::I64(i) => i.$fn(),
                NumberTy::U64(u) => u.$fn(),
                NumberTy::F64(f) => f.$fn(),
            }
        }
    };
}

impl num_traits::ToPrimitive for NumberTy {
    impl_to_primitive_number!(to_f32, f32);

    impl_to_primitive_number!(to_f64, f64);

    impl_to_primitive_number!(to_i128, i128);

    impl_to_primitive_number!(to_i16, i16);

    impl_to_primitive_number!(to_i32, i32);

    impl_to_primitive_number!(to_i64, i64);

    impl_to_primitive_number!(to_i8, i8);

    impl_to_primitive_number!(to_u128, u128);

    impl_to_primitive_number!(to_u16, u16);

    impl_to_primitive_number!(to_u32, u32);

    impl_to_primitive_number!(to_u64, u64);
}

impl NumberTy {
    pub fn promote(left: Self, right: Self) -> Option<(Self, Self)> {
        match (left, right) {
            (NumberTy::I64(l), NumberTy::I64(r)) => Some((NumberTy::I64(l), NumberTy::I64(r))),
            (NumberTy::U64(l), NumberTy::U64(r)) => Some((NumberTy::U64(l), NumberTy::U64(r))),
            (NumberTy::F64(_), _) | (_, NumberTy::F64(_)) => Some((Self::F64(left.to_f64()?), Self::F64(right.to_f64()?))),
            (NumberTy::I64(_), _) | (_, NumberTy::I64(_)) => Some((Self::I64(left.to_i64()?), Self::I64(right.to_i64()?))),
        }
    }
}

pub fn array_access<'a, 'b, T>(array: &'a [T], idx: impl CelValueConv<'b>) -> Result<&'a T, CelError<'b>> {
    let idx = idx.conv();
    match idx.as_number().and_then(|n| n.to_usize()) {
        Some(idx) => array.get(idx).ok_or(CelError::IndexOutOfBounds(idx, array.len())),
        _ => Err(CelError::IndexWithBadIndex(idx)),
    }
}

pub fn array_contains<'a, 'b, T: PartialEq<CelValue<'b>>>(array: &'a [T], value: impl CelValueConv<'b>) -> bool {
    let value = value.conv();
    array.iter().any(|v| v == &value)
}

trait MapKeyCast {
    type Borrow: ToOwned + ?Sized;

    fn make_key<'a>(key: &'a CelValue<'a>) -> Option<Cow<'a, Self::Borrow>>
    where
        Self::Borrow: ToOwned;
}

macro_rules! impl_map_key_cast_number {
    ($ty:ty, $fn:ident) => {
        impl MapKeyCast for $ty {
            type Borrow = Self;

            fn make_key<'a>(key: &'a CelValue<'a>) -> Option<Cow<'a, Self>> {
                match key {
                    CelValue::Number(number) => number.$fn().map(Cow::Owned),
                    _ => None,
                }
            }
        }
    };
}

impl_map_key_cast_number!(i32, to_i32);
impl_map_key_cast_number!(u32, to_u32);
impl_map_key_cast_number!(i64, to_i64);
impl_map_key_cast_number!(u64, to_u64);

impl MapKeyCast for String {
    type Borrow = str;

    fn make_key<'a>(key: &'a CelValue<'a>) -> Option<Cow<'a, Self::Borrow>> {
        match key {
            CelValue::String(s) => Some(Cow::Borrowed(s)),
            CelValue::StringRef(s) => Some(Cow::Borrowed(s)),
            _ => None,
        }
    }
}

#[allow(private_bounds)]
pub fn map_access<'a, 'b, K, V>(map: &'a impl Map<K, V>, key: impl CelValueConv<'b>) -> Result<&'a V, CelError<'b>>
where
    K: Ord + Hash + MapKeyCast,
    K: std::borrow::Borrow<K::Borrow>,
    K::Borrow: std::cmp::Eq + std::hash::Hash + std::cmp::Ord,
{
    let key = key.conv();
    K::make_key(&key)
        .and_then(|key| map.get(&key))
        .ok_or(CelError::MapKeyNotFound(key))
}

#[allow(private_bounds)]
pub fn map_contains<'a, 'b, K, V>(map: &'a impl Map<K, V>, key: impl CelValueConv<'b>) -> bool
where
    K: Ord + Hash + MapKeyCast,
    K: std::borrow::Borrow<K::Borrow>,
    K::Borrow: std::cmp::Eq + std::hash::Hash + std::cmp::Ord,
{
    let key = key.conv();
    K::make_key(&key).and_then(|key| map.get(&key)).is_some()
}

#[test]
fn test_map_idx() {
    let map = HashMap::from([("a".to_owned(), 1), ("b".to_owned(), 2)]);
    assert_eq!(map_access(&map, CelValue::StringRef("a")), Ok(&1));
    assert_eq!(
        map_access(&map, CelValue::StringRef("c")),
        Err(CelError::MapKeyNotFound(CelValue::StringRef("c")))
    );
}

pub trait CelBooleanConv {
    fn to_bool(&self) -> bool;
}

impl CelBooleanConv for bool {
    fn to_bool(&self) -> bool {
        *self
    }
}

impl CelBooleanConv for CelValue<'_> {
    fn to_bool(&self) -> bool {
        CelValue::to_bool(self)
    }
}

impl<T: CelBooleanConv> CelBooleanConv for Option<T> {
    fn to_bool(&self) -> bool {
        self.as_ref().map(CelBooleanConv::to_bool).unwrap_or(false)
    }
}

impl<T> CelBooleanConv for Vec<T> {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl<K, V> CelBooleanConv for BTreeMap<K, V> {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl<K, V> CelBooleanConv for HashMap<K, V> {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl<T> CelBooleanConv for &T
where
    T: CelBooleanConv,
{
    fn to_bool(&self) -> bool {
        CelBooleanConv::to_bool(*self)
    }
}

impl CelBooleanConv for str {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl CelBooleanConv for String {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl<T: CelBooleanConv> CelBooleanConv for [T] {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

impl CelBooleanConv for Bytes {
    fn to_bool(&self) -> bool {
        !self.is_empty()
    }
}

pub fn to_bool(value: impl CelBooleanConv) -> bool {
    value.to_bool()
}
