#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![doc(hidden)]

//! Currently is currently a fully private api used by `tinc` and `tinc-build` to
//! Compile & Execute cel expressions.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;

use bytes::Bytes;
use float_cmp::ApproxEq;
use num_traits::ToPrimitive;

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
pub enum CelString<'a> {
    Owned(Arc<str>),
    Borrowed(&'a str),
}

impl PartialEq for CelString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for CelString<'_> {}

impl<'a> From<&'a str> for CelString<'a> {
    fn from(value: &'a str) -> Self {
        CelString::Borrowed(value)
    }
}

impl From<String> for CelString<'_> {
    fn from(value: String) -> Self {
        CelString::Owned(value.into())
    }
}

impl<'a> From<&'a String> for CelString<'a> {
    fn from(value: &'a String) -> Self {
        CelString::Borrowed(value.as_str())
    }
}

impl From<&Arc<str>> for CelString<'static> {
    fn from(value: &Arc<str>) -> Self {
        CelString::Owned(value.clone())
    }
}

impl From<Arc<str>> for CelString<'static> {
    fn from(value: Arc<str>) -> Self {
        CelString::Owned(value)
    }
}

impl AsRef<str> for CelString<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s,
        }
    }
}

impl std::ops::Deref for CelString<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Clone, Debug)]
pub enum CelBytes<'a> {
    Owned(Bytes),
    Borrowed(&'a [u8]),
}

impl PartialEq for CelBytes<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for CelBytes<'_> {}

impl<'a> From<&'a [u8]> for CelBytes<'a> {
    fn from(value: &'a [u8]) -> Self {
        CelBytes::Borrowed(value)
    }
}

impl From<Bytes> for CelBytes<'_> {
    fn from(value: Bytes) -> Self {
        CelBytes::Owned(value)
    }
}

impl From<&Bytes> for CelBytes<'_> {
    fn from(value: &Bytes) -> Self {
        CelBytes::Owned(value.clone())
    }
}

impl From<Vec<u8>> for CelBytes<'static> {
    fn from(value: Vec<u8>) -> Self {
        CelBytes::Owned(value.into())
    }
}

impl<'a> From<&'a Vec<u8>> for CelBytes<'a> {
    fn from(value: &'a Vec<u8>) -> Self {
        CelBytes::Borrowed(value.as_slice())
    }
}

impl AsRef<[u8]> for CelBytes<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CelValue<'a> {
    Bool(bool),
    Number(NumberTy),
    String(CelString<'a>),
    Bytes(CelBytes<'a>),
    List(Arc<[CelValue<'a>]>),
    Map(Arc<[(CelValue<'a>, CelValue<'a>)]>),
    Duration(chrono::Duration),
    Timestamp(chrono::DateTime<chrono::FixedOffset>),
    Enum(CelEnum<'a>),
    Null,
}

impl PartialOrd for CelValue<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (CelValue::Number(l), CelValue::Number(r)) => l.partial_cmp(r),
            (CelValue::String(_) | CelValue::Bytes(_), CelValue::String(_) | CelValue::Bytes(_)) => {
                let l = match self {
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    _ => unreachable!(),
                };

                let r = match other {
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    _ => unreachable!(),
                };

                Some(l.cmp(r))
            }
            _ => None,
        }
    }
}

impl<'a> CelValue<'a> {
    pub fn cel_access<'b>(container: impl CelValueConv<'a>, key: impl CelValueConv<'b>) -> Result<CelValue<'a>, CelError<'b>>
    where
        'a: 'b,
    {
        let key = key.conv();
        match container.conv() {
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
            v => Err(CelError::BadAccess {
                member: key,
                container: v,
            }),
        }
    }

    pub fn cel_add(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_add(r)?)),
            (CelValue::String(l), CelValue::String(r)) => Ok(CelValue::String(CelString::Owned(Arc::from(format!(
                "{}{}",
                l.as_ref(),
                r.as_ref()
            ))))),
            (CelValue::Bytes(l), CelValue::Bytes(r)) => Ok(CelValue::Bytes(CelBytes::Owned({
                let mut l = l.as_ref().to_vec();
                l.extend_from_slice(r.as_ref());
                Bytes::from(l)
            }))),
            (CelValue::List(l), CelValue::List(r)) => Ok(CelValue::List(l.iter().chain(r.iter()).cloned().collect())),
            (CelValue::Map(l), CelValue::Map(r)) => Ok(CelValue::Map(l.iter().chain(r.iter()).cloned().collect())),
            (left, right) => Err(CelError::BadOperation { left, right, op: "+" }),
        }
    }

    pub fn cel_sub(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'static>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_sub(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "-" }),
        }
    }

    pub fn cel_mul(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'static>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_mul(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "*" }),
        }
    }

    pub fn cel_div(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'static>, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (CelValue::Number(l), CelValue::Number(r)) => Ok(CelValue::Number(l.cel_div(r)?)),
            (left, right) => Err(CelError::BadOperation { left, right, op: "/" }),
        }
    }

    pub fn cel_rem(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<CelValue<'static>, CelError<'a>> {
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
    pub fn cel_neg(input: impl CelValueConv<'a>) -> Result<CelValue<'static>, CelError<'a>> {
        match input.conv() {
            CelValue::Number(n) => Ok(CelValue::Number(n.cel_neg()?)),
            value => Err(CelError::BadUnaryOperation { value, op: "-" }),
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
        Ok(left == right)
    }

    // left != right
    pub fn cel_neq(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        let left = left.conv();
        let right = right.conv();
        Ok(left != right)
    }

    // left.contains(right)
    pub fn cel_contains(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        Self::cel_in(right, left).map_err(|err| match err {
            CelError::BadOperation { left, right, op: "in" } => CelError::BadOperation {
                left: right,
                right: left,
                op: "contains",
            },
            // I think this is unreachable
            err => err,
        })
    }

    // left in right
    pub fn cel_in(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (left, CelValue::List(r)) => Ok(r.contains(&left)),
            (left, CelValue::Map(r)) => Ok(r.iter().any(|(k, _)| k == &left)),
            (left @ (CelValue::Bytes(_) | CelValue::String(_)), right @ (CelValue::Bytes(_) | CelValue::String(_))) => {
                let r = match &right {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                let l = match &left {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                Ok(r.windows(l.len()).any(|w| w == l))
            }
            (left, right) => Err(CelError::BadOperation { left, right, op: "in" }),
        }
    }

    pub fn cel_starts_with(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (left @ (CelValue::Bytes(_) | CelValue::String(_)), right @ (CelValue::Bytes(_) | CelValue::String(_))) => {
                let r = match &right {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                let l = match &left {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                Ok(l.starts_with(r))
            }
            (left, right) => Err(CelError::BadOperation {
                left,
                right,
                op: "startsWith",
            }),
        }
    }

    pub fn cel_ends_with(left: impl CelValueConv<'a>, right: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match (left.conv(), right.conv()) {
            (left @ (CelValue::Bytes(_) | CelValue::String(_)), right @ (CelValue::Bytes(_) | CelValue::String(_))) => {
                let r = match &right {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                let l = match &left {
                    CelValue::Bytes(b) => b.as_ref(),
                    CelValue::String(s) => s.as_ref().as_bytes(),
                    _ => unreachable!(),
                };

                Ok(l.ends_with(r))
            }
            (left, right) => Err(CelError::BadOperation {
                left,
                right,
                op: "startsWith",
            }),
        }
    }

    pub fn cel_matches(value: impl CelValueConv<'a>, regex: &regex::Regex) -> Result<bool, CelError<'a>> {
        match value.conv() {
            value @ (CelValue::Bytes(_) | CelValue::String(_)) => {
                let maybe_str = match &value {
                    CelValue::Bytes(b) => std::str::from_utf8(b.as_ref()),
                    CelValue::String(s) => Ok(s.as_ref()),
                    _ => unreachable!(),
                };

                let Ok(input) = maybe_str else {
                    return Ok(false);
                };

                Ok(regex.is_match(input))
            }
            value => Err(CelError::BadUnaryOperation { op: "matches", value }),
        }
    }

    pub fn cel_is_ipv4(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(s.parse::<std::net::Ipv4Addr>().is_ok()),
            CelValue::Bytes(b) => {
                if b.as_ref().len() == 4 {
                    Ok(true)
                } else if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(s.parse::<std::net::Ipv4Addr>().is_ok())
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isIpv4", value }),
        }
    }

    pub fn cel_is_ipv6(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(s.parse::<std::net::Ipv6Addr>().is_ok()),
            CelValue::Bytes(b) => {
                if b.as_ref().len() == 16 {
                    Ok(true)
                } else if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(s.parse::<std::net::Ipv6Addr>().is_ok())
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isIpv6", value }),
        }
    }

    pub fn cel_is_uuid(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(s.parse::<uuid::Uuid>().is_ok()),
            CelValue::Bytes(b) => {
                if b.as_ref().len() == 16 {
                    Ok(true)
                } else if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(s.parse::<uuid::Uuid>().is_ok())
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isUuid", value }),
        }
    }

    pub fn cel_is_hostname(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(matches!(url::Host::parse(&s), Ok(url::Host::Domain(_)))),
            CelValue::Bytes(b) => {
                if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(matches!(url::Host::parse(s), Ok(url::Host::Domain(_))))
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isHostname", value }),
        }
    }

    pub fn cel_is_uri(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(url::Url::parse(&s).is_ok()),
            CelValue::Bytes(b) => {
                if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(url::Url::parse(s).is_ok())
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isUri", value }),
        }
    }

    pub fn cel_is_email(value: impl CelValueConv<'a>) -> Result<bool, CelError<'a>> {
        match value.conv() {
            CelValue::String(s) => Ok(email_address::EmailAddress::is_valid(&s)),
            CelValue::Bytes(b) => {
                if let Ok(s) = std::str::from_utf8(b.as_ref()) {
                    Ok(email_address::EmailAddress::is_valid(s))
                } else {
                    Ok(false)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "isEmail", value }),
        }
    }

    pub fn cel_size(item: impl CelValueConv<'a>) -> Result<u64, CelError<'a>> {
        match item.conv() {
            Self::Bytes(b) => Ok(b.as_ref().len() as u64),
            Self::String(s) => Ok(s.as_ref().len() as u64),
            Self::List(l) => Ok(l.len() as u64),
            Self::Map(m) => Ok(m.len() as u64),
            item => Err(CelError::BadUnaryOperation { op: "size", value: item }),
        }
    }

    pub fn cel_map(
        item: impl CelValueConv<'a>,
        map_fn: impl Fn(CelValue<'a>) -> Result<CelValue<'a>, CelError<'a>>,
    ) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::List(items) => Ok(CelValue::List(items.iter().cloned().map(map_fn).collect::<Result<_, _>>()?)),
            CelValue::Map(map) => Ok(CelValue::List(
                map.iter()
                    .map(|(key, _)| key)
                    .cloned()
                    .map(map_fn)
                    .collect::<Result<_, _>>()?,
            )),
            value => Err(CelError::BadUnaryOperation { op: "map", value }),
        }
    }

    pub fn cel_filter(
        item: impl CelValueConv<'a>,
        map_fn: impl Fn(CelValue<'a>) -> Result<bool, CelError<'a>>,
    ) -> Result<CelValue<'a>, CelError<'a>> {
        let filter_map = |item: CelValue<'a>| match map_fn(item.clone()) {
            Ok(false) => None,
            Ok(true) => Some(Ok(item)),
            Err(err) => Some(Err(err)),
        };

        match item.conv() {
            CelValue::List(items) => Ok(CelValue::List(
                items.iter().cloned().filter_map(filter_map).collect::<Result<_, _>>()?,
            )),
            CelValue::Map(map) => Ok(CelValue::List(
                map.iter()
                    .map(|(key, _)| key)
                    .cloned()
                    .filter_map(filter_map)
                    .collect::<Result<_, _>>()?,
            )),
            value => Err(CelError::BadUnaryOperation { op: "filter", value }),
        }
    }

    pub fn cel_all(
        item: impl CelValueConv<'a>,
        map_fn: impl Fn(CelValue<'a>) -> Result<bool, CelError<'a>>,
    ) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::List(items) => Ok(CelValue::Bool({
                let mut iter = items.iter();
                loop {
                    let Some(item) = iter.next() else {
                        break true;
                    };

                    if !map_fn(item.clone())? {
                        break false;
                    }
                }
            })),
            CelValue::Map(map) => Ok(CelValue::Bool({
                let mut iter = map.iter();
                loop {
                    let Some((item, _)) = iter.next() else {
                        break true;
                    };

                    if !map_fn(item.clone())? {
                        break false;
                    }
                }
            })),
            value => Err(CelError::BadUnaryOperation { op: "all", value }),
        }
    }

    pub fn cel_exists(
        item: impl CelValueConv<'a>,
        map_fn: impl Fn(CelValue<'a>) -> Result<bool, CelError<'a>>,
    ) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::List(items) => Ok(CelValue::Bool({
                let mut iter = items.iter();
                loop {
                    let Some(item) = iter.next() else {
                        break false;
                    };

                    if map_fn(item.clone())? {
                        break true;
                    }
                }
            })),
            CelValue::Map(map) => Ok(CelValue::Bool({
                let mut iter = map.iter();
                loop {
                    let Some((item, _)) = iter.next() else {
                        break false;
                    };

                    if map_fn(item.clone())? {
                        break true;
                    }
                }
            })),
            value => Err(CelError::BadUnaryOperation { op: "existsOne", value }),
        }
    }

    pub fn cel_exists_one(
        item: impl CelValueConv<'a>,
        map_fn: impl Fn(CelValue<'a>) -> Result<bool, CelError<'a>>,
    ) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::List(items) => Ok(CelValue::Bool({
                let mut iter = items.iter();
                let mut seen = false;
                loop {
                    let Some(item) = iter.next() else {
                        break seen;
                    };

                    if map_fn(item.clone())? {
                        if seen {
                            break false;
                        }

                        seen = true;
                    }
                }
            })),
            CelValue::Map(map) => Ok(CelValue::Bool({
                let mut iter = map.iter();
                let mut seen = false;
                loop {
                    let Some((item, _)) = iter.next() else {
                        break seen;
                    };

                    if map_fn(item.clone())? {
                        if seen {
                            break false;
                        }

                        seen = true;
                    }
                }
            })),
            value => Err(CelError::BadUnaryOperation { op: "existsOne", value }),
        }
    }

    pub fn cel_to_string(item: impl CelValueConv<'a>) -> CelValue<'a> {
        match item.conv() {
            item @ CelValue::String(_) => item,
            CelValue::Bytes(CelBytes::Owned(bytes)) => {
                CelValue::String(CelString::Owned(String::from_utf8_lossy(bytes.as_ref()).into()))
            }
            CelValue::Bytes(CelBytes::Borrowed(b)) => match String::from_utf8_lossy(b) {
                Cow::Borrowed(b) => CelValue::String(CelString::Borrowed(b)),
                Cow::Owned(o) => CelValue::String(CelString::Owned(o.into())),
            },
            item => CelValue::String(CelString::Owned(item.to_string().into())),
        }
    }

    pub fn cel_to_bytes(item: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            item @ CelValue::Bytes(_) => Ok(item.clone()),
            CelValue::String(CelString::Owned(s)) => Ok(CelValue::Bytes(CelBytes::Owned(s.as_bytes().to_vec().into()))),
            CelValue::String(CelString::Borrowed(s)) => Ok(CelValue::Bytes(CelBytes::Borrowed(s.as_bytes()))),
            value => Err(CelError::BadUnaryOperation { op: "bytes", value }),
        }
    }

    pub fn cel_to_int(item: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::String(s) => {
                if let Ok(number) = s.as_ref().parse() {
                    Ok(CelValue::Number(NumberTy::I64(number)))
                } else {
                    Ok(CelValue::Null)
                }
            }
            CelValue::Number(number) => {
                if let Ok(number) = number.to_int() {
                    Ok(CelValue::Number(number))
                } else {
                    Ok(CelValue::Null)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "int", value }),
        }
    }

    pub fn cel_to_uint(item: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::String(s) => {
                if let Ok(number) = s.as_ref().parse() {
                    Ok(CelValue::Number(NumberTy::U64(number)))
                } else {
                    Ok(CelValue::Null)
                }
            }
            CelValue::Number(number) => {
                if let Ok(number) = number.to_uint() {
                    Ok(CelValue::Number(number))
                } else {
                    Ok(CelValue::Null)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "uint", value }),
        }
    }

    pub fn cel_to_double(item: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match item.conv() {
            CelValue::String(s) => {
                if let Ok(number) = s.as_ref().parse() {
                    Ok(CelValue::Number(NumberTy::F64(number)))
                } else {
                    Ok(CelValue::Null)
                }
            }
            CelValue::Number(number) => {
                if let Ok(number) = number.to_double() {
                    Ok(CelValue::Number(number))
                } else {
                    // I think this is unreachable as well
                    Ok(CelValue::Null)
                }
            }
            value => Err(CelError::BadUnaryOperation { op: "double", value }),
        }
    }

    pub fn cel_to_enum(item: impl CelValueConv<'a>, path: impl CelValueConv<'a>) -> Result<CelValue<'a>, CelError<'a>> {
        match (item.conv(), path.conv()) {
            (CelValue::Number(number), CelValue::String(tag)) => {
                let Some(value) = number.to_i32() else {
                    return Ok(CelValue::Null);
                };

                Ok(CelValue::Enum(CelEnum { tag, value }))
            }
            (CelValue::Enum(CelEnum { value, .. }), CelValue::String(tag)) => Ok(CelValue::Enum(CelEnum { tag, value })),
            (value, path) => Err(CelError::BadOperation {
                op: "enum",
                left: value,
                right: path,
            }),
        }
    }
}

impl PartialEq for CelValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CelValue::Bool(left), CelValue::Bool(right)) => left == right,
            (left @ (CelValue::Bytes(_) | CelValue::String(_)), right @ (CelValue::Bytes(_) | CelValue::String(_))) => {
                let left = match left {
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    _ => unreachable!(),
                };

                let right = match right {
                    CelValue::String(s) => s.as_bytes(),
                    CelValue::Bytes(b) => b.as_ref(),
                    _ => unreachable!(),
                };

                left == right
            }
            (CelValue::Duration(left), CelValue::Duration(right)) => left == right,
            (CelValue::Duration(dur), CelValue::Number(seconds)) | (CelValue::Number(seconds), CelValue::Duration(dur)) => {
                (dur.num_seconds() as f64) + dur.subsec_nanos() as f64 / 1_000_000_000.0 == *seconds
            }
            (CelValue::Timestamp(left), CelValue::Timestamp(right)) => left == right,
            (CelValue::Enum(left), CelValue::Enum(right)) => left == right,
            (CelValue::Enum(enum_), CelValue::Number(value)) | (CelValue::Number(value), CelValue::Enum(enum_)) => {
                enum_.value == *value
            }
            (CelValue::List(left), CelValue::List(right)) => left == right,
            (CelValue::Map(left), CelValue::Map(right)) => left == right,
            (CelValue::Number(left), CelValue::Number(right)) => left == right,
            (CelValue::Null, CelValue::Null) => true,
            _ => false,
        }
    }
}

pub trait CelValueConv<'a> {
    fn conv(self) -> CelValue<'a>;
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
        CelValue::String(CelString::Borrowed(self))
    }
}

impl CelValueConv<'_> for Bytes {
    fn conv(self) -> CelValue<'static> {
        CelValue::Bytes(CelBytes::Owned(self.clone()))
    }
}

impl<'a> CelValueConv<'a> for &'a [u8] {
    fn conv(self) -> CelValue<'a> {
        CelValue::Bytes(CelBytes::Borrowed(self))
    }
}

impl<'a, const N: usize> CelValueConv<'a> for &'a [u8; N] {
    fn conv(self) -> CelValue<'a> {
        (self as &[u8]).conv()
    }
}

impl<'a> CelValueConv<'a> for &'a Vec<u8> {
    fn conv(self) -> CelValue<'a> {
        CelValue::Bytes(CelBytes::Borrowed(self))
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

impl<'a> CelValueConv<'a> for &CelValue<'a> {
    fn conv(self) -> CelValue<'a> {
        self.clone()
    }
}

impl std::fmt::Display for CelValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CelValue::Bool(b) => std::fmt::Display::fmt(b, f),
            CelValue::Number(n) => std::fmt::Display::fmt(n, f),
            CelValue::String(s) => std::fmt::Display::fmt(s.as_ref(), f),
            CelValue::Bytes(b) => std::fmt::Debug::fmt(b.as_ref(), f),
            CelValue::List(l) => {
                let mut list = f.debug_list();
                for item in l.iter() {
                    list.entry(&fmtools::fmt(|fmt| item.fmt(fmt)));
                }
                list.finish()
            }
            CelValue::Map(m) => {
                let mut map = f.debug_map();
                for (key, value) in m.iter() {
                    map.entry(&fmtools::fmt(|fmt| key.fmt(fmt)), &fmtools::fmt(|fmt| value.fmt(fmt)));
                }
                map.finish()
            }
            CelValue::Null => std::fmt::Display::fmt("null", f),
            CelValue::Duration(d) => std::fmt::Display::fmt(d, f),
            CelValue::Timestamp(t) => std::fmt::Display::fmt(t, f),
            #[cfg(feature = "runtime")]
            CelValue::Enum(e) => e.into_string().fmt(f),
            #[cfg(not(feature = "runtime"))]
            CelValue::Enum(_) => panic!("enum to string called during build-time"),
        }
    }
}

impl CelValue<'_> {
    pub fn to_bool(&self) -> bool {
        match self {
            CelValue::Bool(b) => *b,
            CelValue::Number(n) => *n != 0,
            CelValue::String(s) => !s.as_ref().is_empty(),
            CelValue::Bytes(b) => !b.as_ref().is_empty(),
            CelValue::List(l) => !l.is_empty(),
            CelValue::Map(m) => !m.is_empty(),
            CelValue::Null => false,
            CelValue::Duration(d) => !d.is_zero(),
            CelValue::Timestamp(t) => t.timestamp_nanos_opt().unwrap_or_default() != 0,
            #[cfg(feature = "runtime")]
            CelValue::Enum(t) => t.is_valid(),
            #[cfg(not(feature = "runtime"))]
            CelValue::Enum(_) => panic!("enum to bool called during build-time"),
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
            (NumberTy::F64(l), NumberTy::F64(r)) => Some(if l.approx_eq(r, float_cmp::F64Margin::default()) {
                std::cmp::Ordering::Equal
            } else {
                l.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal)
            }),
            // I think this is unreachable
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
            // I think this is unreachable
            _ => Err(ERROR),
        }
    }

    pub fn cel_sub(self, other: Self) -> Result<Self, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "subtraction" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_sub(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_sub(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l - r)),
            // I think this is unreachable
            _ => Err(ERROR),
        }
    }

    pub fn cel_mul(self, other: Self) -> Result<Self, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "multiplication" };
        match NumberTy::promote(self, other).ok_or(ERROR)? {
            (NumberTy::I64(l), NumberTy::I64(r)) => Ok(NumberTy::I64(l.checked_mul(r).ok_or(ERROR)?)),
            (NumberTy::U64(l), NumberTy::U64(r)) => Ok(NumberTy::U64(l.checked_mul(r).ok_or(ERROR)?)),
            (NumberTy::F64(l), NumberTy::F64(r)) => Ok(NumberTy::F64(l * r)),
            // I think this is unreachable
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
            // I think this is unreachable
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

    pub fn to_int(self) -> Result<NumberTy, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "int" };
        match self {
            NumberTy::I64(n) => Ok(NumberTy::I64(n)),
            NumberTy::U64(n) => Ok(NumberTy::I64(n.to_i64().ok_or(ERROR)?)),
            NumberTy::F64(n) => Ok(NumberTy::I64(n.to_i64().ok_or(ERROR)?)),
        }
    }

    pub fn to_uint(self) -> Result<NumberTy, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "int" };
        match self {
            NumberTy::I64(n) => Ok(NumberTy::U64(n.to_u64().ok_or(ERROR)?)),
            NumberTy::U64(n) => Ok(NumberTy::U64(n)),
            NumberTy::F64(n) => Ok(NumberTy::U64(n.to_u64().ok_or(ERROR)?)),
        }
    }

    pub fn to_double(self) -> Result<NumberTy, CelError<'static>> {
        const ERROR: CelError<'static> = CelError::NumberOutOfRange { op: "int" };
        match self {
            NumberTy::I64(n) => Ok(NumberTy::F64(n.to_f64().ok_or(ERROR)?)),
            NumberTy::U64(n) => Ok(NumberTy::F64(n.to_f64().ok_or(ERROR)?)),
            NumberTy::F64(n) => Ok(NumberTy::F64(n)),
        }
    }
}

impl std::fmt::Display for NumberTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberTy::I64(n) => std::fmt::Display::fmt(n, f),
            NumberTy::U64(n) => std::fmt::Display::fmt(n, f),
            NumberTy::F64(n) => write!(f, "{n:.2}"), // limit to 2 decimal places
        }
    }
}

impl PartialEq for NumberTy {
    fn eq(&self, other: &Self) -> bool {
        NumberTy::promote(*self, *other)
            .map(|(l, r)| match (l, r) {
                (NumberTy::I64(l), NumberTy::I64(r)) => l == r,
                (NumberTy::U64(l), NumberTy::U64(r)) => l == r,
                (NumberTy::F64(l), NumberTy::F64(r)) => l.approx_eq(r, float_cmp::F64Margin::default()),
                // I think this is unreachable
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

macro_rules! impl_partial_eq {
    ($($ty:ty),*$(,)?) => {
        $(
            impl PartialEq<$ty> for CelValue<'_> {
                fn eq(&self, other: &$ty) -> bool {
                    self == &other.conv()
                }
            }

            impl PartialEq<CelValue<'_>> for $ty {
                fn eq(&self, other: &CelValue<'_>) -> bool {
                    other == self
                }
            }
        )*
    };
}

impl_partial_eq!(String, i32, i64, f64, f32, Vec<u8>, u32, u64);

impl PartialEq<Bytes> for CelValue<'_> {
    fn eq(&self, other: &Bytes) -> bool {
        self == &other.clone().conv()
    }
}

impl PartialEq<CelValue<'_>> for Bytes {
    fn eq(&self, other: &CelValue<'_>) -> bool {
        other == self
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
            CelValue::String(s) => Some(Cow::Borrowed(s.as_ref())),
            _ => None,
        }
    }
}

trait Map<K, V> {
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + std::cmp::Ord + ?Sized;
}

impl<K, V, S> Map<K, V> for HashMap<K, V, S>
where
    K: std::hash::Hash + std::cmp::Eq,
    S: std::hash::BuildHasher,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + std::cmp::Eq + ?Sized,
    {
        HashMap::get(self, key)
    }
}

impl<K, V> Map<K, V> for BTreeMap<K, V>
where
    K: std::cmp::Ord,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::cmp::Ord + ?Sized,
    {
        BTreeMap::get(self, key)
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

#[cfg(feature = "runtime")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CelMode {
    Proto,
    Json,
}

#[cfg(feature = "runtime")]
thread_local! {
    static CEL_MODE: std::cell::Cell<CelMode> = const { std::cell::Cell::new(CelMode::Proto) };
}

#[cfg(feature = "runtime")]
impl CelMode {
    pub fn set(self) {
        CEL_MODE.set(self);
    }

    pub fn current() -> CelMode {
        CEL_MODE.get()
    }

    pub fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }

    pub fn is_proto(self) -> bool {
        matches!(self, Self::Proto)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CelEnum<'a> {
    pub tag: CelString<'a>,
    pub value: i32,
}

impl<'a> CelEnum<'a> {
    pub fn new(tag: CelString<'a>, value: i32) -> CelEnum<'a> {
        CelEnum { tag, value }
    }

    #[cfg(feature = "runtime")]
    pub fn into_string(&self) -> CelValue<'static> {
        EnumVtable::from_tag(self.tag.as_ref())
            .map(|vt| match CEL_MODE.get() {
                CelMode::Json => (vt.to_json)(self.value),
                CelMode::Proto => (vt.to_proto)(self.value),
            })
            .unwrap_or(CelValue::Number(NumberTy::I64(self.value as i64)))
    }

    #[cfg(feature = "runtime")]
    pub fn is_valid(&self) -> bool {
        EnumVtable::from_tag(self.tag.as_ref()).is_some_and(|vt| (vt.is_valid)(self.value))
    }
}

#[cfg(feature = "runtime")]
#[derive(Debug, Copy, Clone)]
pub struct EnumVtable {
    pub proto_path: &'static str,
    pub is_valid: fn(i32) -> bool,
    pub to_json: fn(i32) -> CelValue<'static>,
    pub to_proto: fn(i32) -> CelValue<'static>,
}

#[cfg(feature = "runtime")]
impl EnumVtable {
    pub fn from_tag(tag: &str) -> Option<&'static EnumVtable> {
        static LOOKUP: std::sync::LazyLock<HashMap<&'static str, &'static EnumVtable>> =
            std::sync::LazyLock::new(|| TINC_CEL_ENUM_VTABLE.into_iter().map(|item| (item.proto_path, item)).collect());

        LOOKUP.get(tag).copied()
    }
}

#[cfg(feature = "runtime")]
#[linkme::distributed_slice]
pub static TINC_CEL_ENUM_VTABLE: [EnumVtable];

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::borrow::Cow;
    use std::cmp::Ordering;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    use bytes::Bytes;
    use chrono::{DateTime, Duration, FixedOffset};
    use num_traits::ToPrimitive;
    use regex::Regex;
    use uuid::Uuid;

    use super::CelString;
    use crate::{
        CelBooleanConv, CelBytes, CelEnum, CelError, CelValue, CelValueConv, MapKeyCast, NumberTy, array_access,
        array_contains, map_access, map_contains,
    };

    #[test]
    fn celstring_eq() {
        // borrowed vs borrowed
        let b1 = CelString::Borrowed("foo");
        let b2 = CelString::Borrowed("foo");
        assert_eq!(b1, b2);

        // owned vs owned
        let o1 = CelString::Owned(Arc::from("foo"));
        let o2 = CelString::Owned(Arc::from("foo"));
        assert_eq!(o1, o2);

        // borrowed vs owned (both directions)
        let b = CelString::Borrowed("foo");
        let o = CelString::Owned(Arc::from("foo"));
        assert_eq!(b, o.clone());
        assert_eq!(o, b);

        // inequality
        let bar_b = CelString::Borrowed("bar");
        let bar_o = CelString::Owned(Arc::from("bar"));
        assert_ne!(b1, bar_b);
        assert_ne!(o1, bar_o);
    }

    #[test]
    fn celstring_borrowed() {
        let original = String::from("hello");
        let cs: CelString = (&original).into();

        match cs {
            CelString::Borrowed(s) => {
                assert_eq!(s, "hello");
                // ensure it really is a borrow, not an owned Arc
                let orig_ptr = original.as_ptr();
                let borrow_ptr = s.as_ptr();
                assert_eq!(orig_ptr, borrow_ptr);
            }
            _ => panic!("expected CelString::Borrowed"),
        }
    }

    #[test]
    fn celstring_owned() {
        let arc: Arc<str> = Arc::from("world");
        let cs: CelString<'static> = (&arc).into();

        match cs {
            CelString::Owned(o) => {
                assert_eq!(o.as_ref(), "world");
                assert!(Arc::ptr_eq(&o, &arc));
                assert_eq!(Arc::strong_count(&arc), 2);
            }
            _ => panic!("expected CelString::Owned"),
        }
    }

    #[test]
    fn borrowed_eq_borrowed() {
        let slice1: &[u8] = &[1, 2, 3];
        let slice2: &[u8] = &[1, 2, 3];
        let b1: CelBytes = slice1.into();
        let b2: CelBytes = slice2.into();
        assert_eq!(b1, b2);
    }

    #[test]
    fn owned_eq_owned() {
        let data = vec![10, 20, 30];
        let o1: CelBytes<'static> = Bytes::from(data.clone()).into();
        let o2: CelBytes<'static> = Bytes::from(data.clone()).into();
        assert_eq!(o1, o2);
    }

    #[test]
    fn borrowed_eq_owned() {
        let v = vec![5, 6, 7];
        let owned: CelBytes<'static> = Bytes::from(v.clone()).into();
        let borrowed: CelBytes = v.as_slice().into();

        // Owned vs Borrowed
        assert_eq!(owned, borrowed);
        // Borrowed vs Owned
        assert_eq!(borrowed, owned);
    }

    #[test]
    fn celbytes_neq() {
        let b1: CelBytes = (&[1, 2, 3][..]).into();
        let b2: CelBytes = (&[4, 5, 6][..]).into();
        assert_ne!(b1, b2);

        let o1: CelBytes<'static> = Bytes::from(vec![1, 2, 3]).into();
        let o2: CelBytes<'static> = Bytes::from(vec![7, 8, 9]).into();
        assert_ne!(o1, o2);
    }

    #[test]
    fn celbytes_borrowed_slice() {
        let arr: [u8; 4] = [9, 8, 7, 6];
        let cb: CelBytes = arr.as_slice().into();
        match cb {
            CelBytes::Borrowed(s) => {
                assert_eq!(s, arr.as_slice());
                // pointer equality check
                assert_eq!(s.as_ptr(), arr.as_ptr());
            }
            _ => panic!("Expected CelBytes::Borrowed from slice"),
        }
    }

    #[test]
    fn celbytes_bstr_owned() {
        let bytes = Bytes::from_static(b"rust");
        let cb: CelBytes = bytes.clone().into();
        match cb {
            CelBytes::Owned(b) => {
                assert_eq!(b, bytes);
            }
            _ => panic!("Expected CelBytes::Owned from Bytes"),
        }
    }

    #[test]
    fn celbytes_vec_owned() {
        let data = vec![0x10, 0x20, 0x30];
        let cb: CelBytes<'static> = data.clone().into();

        match cb {
            CelBytes::Owned(bytes) => {
                assert_eq!(bytes.as_ref(), &[0x10, 0x20, 0x30]);
                assert_eq!(bytes, Bytes::from(data));
            }
            _ => panic!("Expected CelBytes::Owned variant"),
        }
    }

    #[test]
    fn celbytes_vec_borrowed() {
        let data = vec![4u8, 5, 6];
        let cb: CelBytes = (&data).into();

        match cb {
            CelBytes::Borrowed(slice) => {
                assert_eq!(slice, data.as_slice());

                let data_ptr = data.as_ptr();
                let slice_ptr = slice.as_ptr();
                assert_eq!(data_ptr, slice_ptr);
            }
            _ => panic!("Expected CelBytes::Borrowed variant"),
        }
    }

    #[test]
    fn celvalue_partial_cmp() {
        let one = 1i32.conv();
        let two = 2i32.conv();
        assert_eq!(one.partial_cmp(&two), Some(Ordering::Less));
        assert_eq!(two.partial_cmp(&one), Some(Ordering::Greater));
        assert_eq!(one.partial_cmp(&1i32.conv()), Some(Ordering::Equal));
    }

    #[test]
    fn celvalue_str_byte_partial_cmp() {
        let s1 = "abc".conv();
        let s2 = "abd".conv();
        assert_eq!(s1.partial_cmp(&s2), Some(Ordering::Less));

        let b1 = Bytes::from_static(b"abc").conv();
        let b2 = Bytes::from_static(b"abd").conv();
        assert_eq!(b1.partial_cmp(&b2), Some(Ordering::Less));

        // cross: string vs bytes
        assert_eq!(s1.partial_cmp(&b1), Some(Ordering::Equal));
        assert_eq!(b1.partial_cmp(&s2), Some(Ordering::Less));
    }

    #[test]
    fn celvalue_mismatched_partial_cmp() {
        let num = 1i32.conv();
        let strv = "a".conv();
        assert_eq!(num.partial_cmp(&strv), None);
        assert_eq!(strv.partial_cmp(&num), None);

        let binding = Vec::<i32>::new();
        let list = (&binding).conv();
        let map = CelValue::Map(Arc::from(vec![]));
        assert_eq!(list.partial_cmp(&map), None);
    }

    // Helpers to build list and map CelValues
    fn make_list(vals: &[i32]) -> CelValue<'static> {
        let items: Vec<_> = vals.iter().map(|&n| n.conv()).collect();
        CelValue::List(Arc::from(items))
    }

    fn make_map(pairs: &[(i32, i32)]) -> CelValue<'static> {
        let items: Vec<_> = pairs.iter().map(|&(k, v)| (k.conv(), v.conv())).collect();
        CelValue::Map(Arc::from(items))
    }

    #[test]
    fn celvalue_pos_neg_ints() {
        let num = CelValue::Number(NumberTy::I64(42));
        assert_eq!(num.as_number(), Some(NumberTy::I64(42)));

        let neg = CelValue::cel_neg(5i32);
        assert_eq!(neg.unwrap(), CelValue::Number(NumberTy::I64(-5)));

        let err = CelValue::cel_neg("foo").unwrap_err();
        matches!(err, CelError::BadUnaryOperation { op: "-", .. });
    }

    #[test]
    fn celvalue_map_keys() {
        let map = make_map(&[(1, 10), (2, 20)]);
        let v = CelValue::cel_access(map.clone(), 2i32).unwrap();
        assert_eq!(v, 20i32.conv());

        let err = CelValue::cel_access(map, 3i32).unwrap_err();
        matches!(err, CelError::MapKeyNotFound(k) if k == 3i32.conv());
    }

    #[test]
    fn celvalue_list_access() {
        let list = make_list(&[100, 200, 300]);
        let v = CelValue::cel_access(list.clone(), 1u32).unwrap();
        assert_eq!(v, 200i32.conv());

        let err = CelValue::cel_access(list.clone(), 5i32).unwrap_err();
        matches!(err, CelError::IndexOutOfBounds(5, 3));

        let err2 = CelValue::cel_access(list, "not_index").unwrap_err();
        matches!(err2, CelError::IndexWithBadIndex(k) if k == "not_index".conv());
    }

    #[test]
    fn celvalue_bad_access() {
        let s = "hello".conv();
        let err = CelValue::cel_access(s.clone(), 0i32).unwrap_err();
        matches!(err, CelError::BadAccess { member, container } if member == 0i32.conv() && container == s);
    }

    #[test]
    fn celvalue_add() {
        // number
        assert_eq!(CelValue::cel_add(3i32, 4i32).unwrap(), 7i32.conv());
        // string
        let s = CelValue::cel_add("foo", "bar").unwrap();
        assert_eq!(s, CelValue::String(CelString::Owned(Arc::from("foobar"))));
        // bytes
        let b = CelValue::cel_add(Bytes::from_static(b"ab"), Bytes::from_static(b"cd")).unwrap();
        assert_eq!(b, CelValue::Bytes(CelBytes::Owned(Bytes::from_static(b"abcd"))));
        // list
        let l = CelValue::cel_add(make_list(&[1, 2]), make_list(&[3])).unwrap();
        assert_eq!(l, make_list(&[1, 2, 3]));
        // map
        let m1 = make_map(&[(1, 1)]);
        let m2 = make_map(&[(2, 2)]);
        let m3 = CelValue::cel_add(m1.clone(), m2.clone()).unwrap();
        assert_eq!(m3, make_map(&[(1, 1), (2, 2)]));
        // bad operation
        let err = CelValue::cel_add(1i32, "x").unwrap_err();
        matches!(err, CelError::BadOperation { op: "+", .. });
    }

    #[test]
    fn celvalue_sub_mul_div_rem() {
        // sub
        assert_eq!(CelValue::cel_sub(10i32, 3i32).unwrap(), 7i32.conv());
        assert!(matches!(
            CelValue::cel_sub(1i32, "x").unwrap_err(),
            CelError::BadOperation { op: "-", .. }
        ));
        // mul
        assert_eq!(CelValue::cel_mul(6i32, 7i32).unwrap(), 42i32.conv());
        assert!(matches!(
            CelValue::cel_mul("a", 2i32).unwrap_err(),
            CelError::BadOperation { op: "*", .. }
        ));
        // div
        assert_eq!(CelValue::cel_div(8i32, 2i32).unwrap(), 4i32.conv());
        assert!(matches!(
            CelValue::cel_div(8i32, "x").unwrap_err(),
            CelError::BadOperation { op: "/", .. }
        ));
        // rem
        assert_eq!(CelValue::cel_rem(9i32, 4i32).unwrap(), 1i32.conv());
        assert!(matches!(
            CelValue::cel_rem("a", 1i32).unwrap_err(),
            CelError::BadOperation { op: "%", .. }
        ));
    }

    // helper to build a map CelValue from &[(K, V)]
    fn as_map(pairs: &[(i32, i32)]) -> CelValue<'static> {
        let items: Vec<_> = pairs.iter().map(|&(k, v)| (k.conv(), v.conv())).collect();
        CelValue::Map(Arc::from(items))
    }

    #[test]
    fn celvalue_neq() {
        assert!(CelValue::cel_neq(1i32, 2i32).unwrap());
        assert!(!CelValue::cel_neq("foo", "foo").unwrap());
    }

    #[test]
    fn celvalue_in_and_contains_ints() {
        let list = [1, 2, 3].conv();
        assert!(CelValue::cel_in(2i32, &list).unwrap());
        assert!(!CelValue::cel_in(4i32, &list).unwrap());

        let map = as_map(&[(10, 100), (20, 200)]);
        assert!(CelValue::cel_in(10i32, &map).unwrap());
        assert!(!CelValue::cel_in(30i32, &map).unwrap());

        // contains flips in
        assert!(CelValue::cel_contains(&list, 3i32).unwrap());
        assert!(!CelValue::cel_contains(&map, 30i32).unwrap());
    }

    #[test]
    fn celvalue_contains_bad_operation() {
        let err = CelValue::cel_contains(1i32, "foo").unwrap_err();
        if let CelError::BadOperation { left, right, op } = err {
            assert_eq!(op, "contains");
            assert_eq!(left, 1i32.conv());
            assert_eq!(right, "foo".conv());
        } else {
            panic!("expected CelError::BadOperation with op=\"contains\"");
        }
    }

    #[test]
    fn celvalue_in_and_contains_bytes() {
        let s = "hello world";
        let b = Bytes::from_static(b"hello world");
        let b_again = Bytes::from_static(b"hello world");

        // substring
        assert!(CelValue::cel_in("world", s).unwrap());
        assert!(CelValue::cel_in(Bytes::from_static(b"wor"), b).unwrap());

        // contains
        assert!(CelValue::cel_contains(s, "lo wo").unwrap());
        assert!(CelValue::cel_contains(b_again, Bytes::from_static(b"lo")).unwrap());

        // not found
        assert!(!CelValue::cel_in("abc", s).unwrap());
        assert!(!CelValue::cel_contains(s, "xyz").unwrap());
    }

    #[test]
    fn celvalue_in_and_contains_bad_operations() {
        let err = CelValue::cel_in(1i32, "foo").unwrap_err();
        match err {
            CelError::BadOperation { op, .. } => assert_eq!(op, "in"),
            _ => panic!("Expected BadOperation"),
        }

        let err2 = CelValue::cel_contains(1i32, "foo").unwrap_err();
        match err2 {
            CelError::BadOperation { op, .. } => assert_eq!(op, "contains"),
            _ => panic!("Expected BadOperation contains"),
        }
    }

    #[test]
    fn celvalue_starts_with_and_ends_with() {
        // starts_with & ends_with string
        assert!(CelValue::cel_starts_with("rustacean", "rust").unwrap());
        assert!(CelValue::cel_ends_with("rustacean", "acean").unwrap());

        // bytes
        let b = Bytes::from_static(b"0123456");
        let b_again = Bytes::from_static(b"0123456");
        assert!(CelValue::cel_starts_with(b, Bytes::from_static(b"01")).unwrap());
        assert!(CelValue::cel_ends_with(b_again, Bytes::from_static(b"56")).unwrap());

        // type errors
        let e1 = CelValue::cel_starts_with(123i32, "1").unwrap_err();
        assert!(matches!(e1, CelError::BadOperation { op, .. } if op=="startsWith"));
        let e2 = CelValue::cel_ends_with(123i32, "1").unwrap_err();
        assert!(matches!(e2, CelError::BadOperation { op, .. } if op=="startsWith"));
    }

    #[test]
    fn celvalue_matches() {
        let re = Regex::new(r"^a.*z$").unwrap();
        assert!(CelValue::cel_matches("abcz", &re).unwrap());

        let b = Bytes::from_static(b"abcz");
        assert!(CelValue::cel_matches(b, &re).unwrap());

        // non-utf8 bytes -> Ok(false)
        let bad = CelValue::cel_matches(Bytes::from_static(&[0xff, 0xfe]), &Regex::new(".*").unwrap()).unwrap();
        assert!(!bad);

        let err = CelValue::cel_matches(1i32, &re).unwrap_err();
        assert!(matches!(err, CelError::BadUnaryOperation { op, .. } if op=="matches"));
    }

    #[test]
    fn celvalue_ip_and_uuid_hostname_uri_email() {
        // IPv4
        assert!(CelValue::cel_is_ipv4("127.0.0.1").unwrap());
        assert!(CelValue::cel_is_ipv4(Bytes::from_static(&[127, 0, 0, 1])).unwrap());
        assert!(!CelValue::cel_is_ipv4(Bytes::from_static(b"notip")).unwrap());
        assert!(matches!(
            CelValue::cel_is_ipv4(true).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isIpv4"
        ));

        // IPv6
        assert!(CelValue::cel_is_ipv6("::1").unwrap());
        let octets = [0u8; 16];
        assert!(CelValue::cel_is_ipv6(&octets).unwrap());
        assert!(!CelValue::cel_is_ipv6(Bytes::from_static(b"bad")).unwrap());
        assert!(matches!(
            CelValue::cel_is_ipv6(1i32).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isIpv6"
        ));

        // UUID
        let uuid_str_nil = Uuid::nil().to_string();
        assert!(CelValue::cel_is_uuid(&uuid_str_nil).unwrap());
        let uuid_str_max = Uuid::max().to_string();
        assert!(CelValue::cel_is_uuid(&uuid_str_max).unwrap());

        let mut bytes16 = [0u8; 16];
        bytes16[0] = 1;
        assert!(CelValue::cel_is_uuid(&bytes16).unwrap());
        assert!(!CelValue::cel_is_uuid(Bytes::from_static(b"short")).unwrap());
        assert!(matches!(
            CelValue::cel_is_uuid(1i32).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isUuid"
        ));

        // hostname
        assert!(CelValue::cel_is_hostname("example.com").unwrap());
        assert!(!CelValue::cel_is_hostname("not valid!").unwrap());
        assert!(matches!(
            CelValue::cel_is_hostname(1i32).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isHostname"
        ));

        // URI str
        assert!(CelValue::cel_is_uri("https://rust-lang.org").unwrap());
        assert!(!CelValue::cel_is_uri(Bytes::from_static(b":bad")).unwrap());
        assert!(matches!(
            CelValue::cel_is_uri(1i32).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isUri"
        ));

        // email str
        assert!(CelValue::cel_is_email("user@example.com").unwrap());
        assert!(!CelValue::cel_is_email(Bytes::from_static(b"noatsign")).unwrap());
        assert!(matches!(
            CelValue::cel_is_email(1i32).unwrap_err(),
            CelError::BadUnaryOperation { op, .. } if op == "isEmail"
        ));
    }

    #[test]
    fn celvalue_ipv4_invalid() {
        let invalid = Bytes::from_static(&[0xff, 0xfe, 0xff, 0xff, 0xff]);
        let result = CelValue::cel_is_ipv4(invalid).unwrap();
        assert!(!result, "Expected false for non-UTF8, non-4-byte input");
    }

    #[test]
    fn celvalue_ipv6_invalid() {
        let invalid = Bytes::from_static(&[0xff, 0xfe, 0xff]);
        let result = CelValue::cel_is_ipv6(invalid).unwrap();
        assert!(!result, "Expected false for non-UTF8, non-16-byte input");
    }

    #[test]
    fn celvalue_uuid_invalid() {
        // length != 16 and invalid UTF-8 → should hit the final `Ok(false)` branch
        let invalid = Bytes::from_static(&[0xff, 0xfe, 0xff]);
        let result = CelValue::cel_is_uuid(invalid).unwrap();
        assert!(!result, "Expected false for non-UTF8, non-16-byte input");
    }

    #[test]
    fn celvalue_hostname_invalid() {
        let valid = CelValue::cel_is_hostname(Bytes::from_static(b"example.com")).unwrap();
        assert!(valid, "Expected true for valid hostname bytes");

        let invalid = CelValue::cel_is_hostname(Bytes::from_static(&[0xff, 0xfe, 0xff])).unwrap();
        assert!(!invalid, "Expected false for invalid UTF-8 bytes");
    }

    #[test]
    fn celvalue_uri_invalid() {
        let invalid = Bytes::from_static(&[0xff, 0xfe, 0xff]);
        let result = CelValue::cel_is_uri(invalid).unwrap();
        assert!(!result, "Expected false for invalid UTF-8 uri bytes");
    }

    #[test]
    fn celvalue_email_invalid() {
        let invalid = Bytes::from_static(&[0xff, 0xfe, 0xff]);
        let result = CelValue::cel_is_email(invalid).unwrap();
        assert!(!result, "Expected false for invalid UTF-8 email bytes");
    }

    #[test]
    fn celvalue_size() {
        assert_eq!(CelValue::cel_size(Bytes::from_static(b"abc")).unwrap(), 3);
        assert_eq!(CelValue::cel_size("hello").unwrap(), 5);
        assert_eq!(CelValue::cel_size([1, 2, 3].conv()).unwrap(), 3);
        assert_eq!(CelValue::cel_size(as_map(&[(1, 1), (2, 2)])).unwrap(), 2);

        let err = CelValue::cel_size(123i32).unwrap_err();
        assert!(matches!(err, CelError::BadUnaryOperation { op, .. } if op=="size"));
    }

    #[test]
    fn celvalue_map_and_filter() {
        // map: double each number
        let m = CelValue::cel_map([1, 2, 3].conv(), |v| {
            let n = v.as_number().unwrap().to_i64().unwrap();
            Ok((n * 2).conv())
        })
        .unwrap();
        assert_eq!(m, [2, 4, 6].conv());

        // map over map produces list of keys
        let keys = CelValue::cel_map(as_map(&[(10, 100), (20, 200)]), Ok).unwrap();
        assert_eq!(keys, [10, 20].conv());

        // filter: keep evens
        let f =
            CelValue::cel_filter([1, 2, 3, 4].conv(), |v| Ok(v.as_number().unwrap().to_i64().unwrap() % 2 == 0)).unwrap();
        assert_eq!(f, [2, 4].conv());

        // filter on map => list of keys
        let fk = CelValue::cel_filter(as_map(&[(7, 70), (8, 80)]), |v| {
            Ok(v.as_number().unwrap().to_i64().unwrap() == 8)
        })
        .unwrap();
        assert_eq!(fk, [8].conv());

        // error on wrong type
        let err_map = CelValue::cel_map(1i32, |_| Ok(1i32.conv())).unwrap_err();
        assert!(matches!(err_map, CelError::BadUnaryOperation { op, .. } if op=="map"));
        let err_filter = CelValue::cel_filter(1i32, |_| Ok(true)).unwrap_err();
        assert!(matches!(err_filter, CelError::BadUnaryOperation { op, .. } if op=="filter"));
    }

    #[test]
    fn celvalue_list_and_filter() {
        let list = [1i32, 2, 3].conv();

        let err = CelValue::cel_filter(list, |v| {
            if v == 2i32.conv() {
                Err(CelError::BadUnaryOperation { op: "test", value: v })
            } else {
                Ok(true)
            }
        })
        .unwrap_err();

        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "test");
            assert_eq!(value, 2i32.conv());
        } else {
            panic!("expected BadUnaryOperation from map_fn");
        }
    }

    #[test]
    fn celvalue_list_and_map_all() {
        let list = [1, 2, 3].conv();
        let all_pos = CelValue::cel_all(list.clone(), |v| Ok(v.as_number().unwrap().to_i64().unwrap() > 0)).unwrap();
        assert_eq!(all_pos, CelValue::Bool(true));

        let list2 = [1, 0, 3].conv();
        let any_zero = CelValue::cel_all(list2, |v| Ok(v.as_number().unwrap().to_i64().unwrap() > 0)).unwrap();
        assert_eq!(any_zero, CelValue::Bool(false));

        let map = as_map(&[(2, 20), (4, 40)]);
        let all_keys = CelValue::cel_all(map.clone(), |v| Ok(v.as_number().unwrap().to_i64().unwrap() < 5)).unwrap();
        assert_eq!(all_keys, CelValue::Bool(true));

        let map2 = as_map(&[(2, 20), (6, 60)]);
        let some_ge5 = CelValue::cel_all(map2, |v| Ok(v.as_number().unwrap().to_i64().unwrap() < 5)).unwrap();
        assert_eq!(some_ge5, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_list_error_propagation() {
        let list = [1, 2, 3].conv();
        let err = CelValue::cel_all(list, |v| {
            if v == 2i32.conv() {
                Err(CelError::BadUnaryOperation {
                    op: "all_test",
                    value: v,
                })
            } else {
                Ok(true)
            }
        })
        .unwrap_err();

        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "all_test");
            assert_eq!(value, 2i32.conv());
        } else {
            panic!("Expected BadUnaryOperation from map_fn");
        }
    }

    #[test]
    fn celvalue_all_bad_operation() {
        let err = CelValue::cel_all(42i32, |_| Ok(true)).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "all");
            assert_eq!(value, 42i32.conv());
        } else {
            panic!("Expected BadUnaryOperation with op=\"all\"");
        }
    }

    #[test]
    fn celvalue_exists() {
        let list = [1, 2, 3].conv();
        let result = CelValue::cel_exists(list, |v| Ok(v == 2i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(true));
    }

    #[test]
    fn celvalue_exists_list_false() {
        let list = [1, 2, 3].conv();
        let result = CelValue::cel_exists(list, |_| Ok(false)).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_map_true() {
        let map = as_map(&[(10, 100), (20, 200)]);
        let result = CelValue::cel_exists(map, |v| Ok(v == 20i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(true));
    }

    #[test]
    fn celvalue_exists_map_false() {
        let map = as_map(&[(10, 100), (20, 200)]);
        let result = CelValue::cel_exists(map, |_| Ok(false)).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_list_propagates_error() {
        let list = [1, 2, 3].conv();
        let err = CelValue::cel_exists(list, |v| {
            if v == 2i32.conv() {
                Err(CelError::BadUnaryOperation {
                    op: "exists_test",
                    value: v,
                })
            } else {
                Ok(false)
            }
        })
        .unwrap_err();

        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "exists_test");
            assert_eq!(value, 2i32.conv());
        } else {
            panic!("Expected BadUnaryOperation from map_fn");
        }
    }

    #[test]
    fn celvalue_exists_non_collection_error() {
        let err = CelValue::cel_exists(42i32, |_| Ok(true)).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "existsOne");
            assert_eq!(value, 42i32.conv());
        } else {
            panic!("Expected BadUnaryOperation with op=\"existsOne\"");
        }
    }

    #[test]
    fn celvalue_exists_one_list() {
        let list = [1, 2, 3].conv();
        let result = CelValue::cel_exists_one(list, |v| Ok(v == 2i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(true));
    }

    #[test]
    fn celvalue_exists_one_list_zero() {
        let list = [1, 2, 3].conv();
        let result = CelValue::cel_exists_one(list, |_| Ok(false)).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_one_list_multiple() {
        let list = [1, 2, 2, 3].conv();
        let result = CelValue::cel_exists_one(list, |v| Ok(v == 2i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_one_map() {
        let map = as_map(&[(10, 100), (20, 200)]);
        let result = CelValue::cel_exists_one(map, |v| Ok(v == 20i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(true));
    }

    #[test]
    fn celvalue_exists_one_map_zero() {
        let map = as_map(&[(10, 100), (20, 200)]);
        let result = CelValue::cel_exists_one(map, |_| Ok(false)).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_one_map_multiple() {
        let map = as_map(&[(1, 10), (1, 20), (2, 30)]);
        let result = CelValue::cel_exists_one(map, |v| Ok(v == 1i32.conv())).unwrap();
        assert_eq!(result, CelValue::Bool(false));
    }

    #[test]
    fn celvalue_exists_one_propagates_error() {
        let list = [1, 2, 3].conv();
        let err = CelValue::cel_exists_one(list, |v| {
            if v == 2i32.conv() {
                Err(CelError::BadUnaryOperation {
                    op: "test_one",
                    value: v,
                })
            } else {
                Ok(false)
            }
        })
        .unwrap_err();

        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "test_one");
            assert_eq!(value, 2i32.conv());
        } else {
            panic!("Expected BadUnaryOperation from map_fn");
        }
    }

    #[test]
    fn celvalue_exists_one_non_collection_error() {
        let err = CelValue::cel_exists_one(42i32, |_| Ok(true)).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "existsOne");
            assert_eq!(value, 42i32.conv());
        } else {
            panic!("Expected BadUnaryOperation with op=\"existsOne\"");
        }
    }

    #[test]
    fn celvalue_to_string_variant_passthrough() {
        let original = "hello";
        let cv = original.conv();
        let out = CelValue::cel_to_string(cv.clone());

        assert!(matches!(out, CelValue::String(_)));
        assert_eq!(out, cv);
    }

    #[test]
    fn celvalue_to_string_owned_bytes() {
        let bytes = Bytes::from_static(b"foo");
        let out = CelValue::cel_to_string(bytes.clone());

        assert_eq!(out, CelValue::String(CelString::Owned(Arc::from("foo"))));
    }

    #[test]
    fn celvalue_to_string_borrowed_bytes() {
        let slice: &[u8] = b"bar";
        let out = CelValue::cel_to_string(slice);

        match out {
            CelValue::String(CelString::Borrowed(s)) => assert_eq!(s, "bar"),
            _ => panic!("expected Borrowed variant"),
        }
    }

    #[test]
    fn celvalue_to_string_borrowed_bytes_invalid_utf8_to_owned() {
        let slice: &[u8] = &[0xff, 0xfe];
        let out = CelValue::cel_to_string(slice);

        match out {
            CelValue::String(CelString::Owned(o)) => {
                assert_eq!(o.as_ref(), "\u{FFFD}\u{FFFD}");
            }
            _ => panic!("expected Owned variant"),
        }
    }

    #[test]
    fn celvalue_to_string_num_and_bool() {
        let out_num = CelValue::cel_to_string(42i32);
        assert_eq!(out_num, CelValue::String(CelString::Owned(Arc::from("42"))));

        let out_bool = CelValue::cel_to_string(true);
        assert_eq!(out_bool, CelValue::String(CelString::Owned(Arc::from("true"))));
    }

    #[test]
    fn celvalue_to_bytes_variant_passthrough() {
        let bytes = Bytes::from_static(b"xyz");
        let cv = CelValue::cel_to_bytes(bytes.clone()).unwrap();
        match cv {
            CelValue::Bytes(CelBytes::Owned(b)) => assert_eq!(b, bytes),
            _ => panic!("expected Owned bytes passthrough"),
        }
    }

    #[test]
    fn celvalue_to_bytes_from_owned_string() {
        let owned_str = CelString::Owned(Arc::from("hello"));
        let cv_in = CelValue::String(owned_str.clone());
        let cv = CelValue::cel_to_bytes(cv_in).unwrap();
        match cv {
            CelValue::Bytes(CelBytes::Owned(b)) => {
                assert_eq!(b.as_ref(), b"hello");
            }
            _ => panic!("expected Owned bytes from Owned string"),
        }
    }

    #[test]
    fn celvalue_to_bytes_from_borrowed_string() {
        let s = "world";
        let cv = CelValue::cel_to_bytes(s).unwrap();
        match cv {
            CelValue::Bytes(CelBytes::Borrowed(b)) => {
                assert_eq!(b, b"world");
            }
            _ => panic!("expected Borrowed bytes from Borrowed string"),
        }
    }

    #[test]
    fn celvalue_error_on_non_string_bytes() {
        let err = CelValue::cel_to_bytes(123i32).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "bytes");
            assert_eq!(value, 123i32.conv());
        } else {
            panic!("expected BadUnaryOperation for non-bytes/string");
        }
    }

    #[test]
    fn celvalue_to_int_from_string() {
        let result = CelValue::cel_to_int("123").unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::I64(123)));
    }

    #[test]
    fn celvalue_to_int_from_nan() {
        let result = CelValue::cel_to_int("not_a_number").unwrap();
        assert_eq!(result, CelValue::Null);
    }

    #[test]
    fn celvalue_to_int_from_float() {
        let result = CelValue::cel_to_int(3.99f64).unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::I64(3)));
    }

    #[test]
    fn celvalue_to_int_too_large() {
        let large = u64::MAX.conv();
        let result = CelValue::cel_to_int(large).unwrap();
        assert_eq!(result, CelValue::Null);
    }

    #[test]
    fn celvalue_to_int_from_bytes_bad_operation() {
        let err = CelValue::cel_to_int(&[1, 2, 3][..]).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "int");
            assert_eq!(value, (&[1, 2, 3][..]).conv());
        } else {
            panic!("Expected BadUnaryOperation for non-string/number");
        }
    }

    #[test]
    fn celvalue_to_uint_from_string() {
        let result = CelValue::cel_to_uint("456").unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::U64(456)));
    }

    #[test]
    fn celvalue_to_uint_from_nan() {
        let result = CelValue::cel_to_uint("not_uint").unwrap();
        assert_eq!(result, CelValue::Null);
    }

    #[test]
    fn celvalue_to_uint_from_int_float_uint() {
        let result_i = CelValue::cel_to_uint(42i32).unwrap();
        assert_eq!(result_i, CelValue::Number(NumberTy::U64(42)));

        let result_f = CelValue::cel_to_uint(3.7f64).unwrap();
        assert_eq!(result_f, CelValue::Number(NumberTy::U64(3)));

        let result_u = CelValue::cel_to_uint(100u64).unwrap();
        assert_eq!(result_u, CelValue::Number(NumberTy::U64(100)));
    }

    #[test]
    fn celvalue_to_uint_neg_and_too_large() {
        let result_neg = CelValue::cel_to_uint(-5i32).unwrap();
        assert_eq!(result_neg, CelValue::Null);

        let big = f64::INFINITY;
        let result_inf = CelValue::cel_to_uint(big).unwrap();
        assert_eq!(result_inf, CelValue::Null);
    }

    #[test]
    fn celvalue_to_uint_from_bytes_bad_operation() {
        let err = CelValue::cel_to_uint(&[1, 2, 3][..]).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "uint");
            assert_eq!(value, (&[1, 2, 3][..]).conv());
        } else {
            panic!("Expected BadUnaryOperation for non-string/number");
        }
    }

    #[test]
    fn celvalue_to_double_from_string_valid() {
        let result = CelValue::cel_to_double("3.141592653589793").unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::F64(std::f64::consts::PI)));
    }

    #[test]
    fn celvalue_to_double_from_string_invalid_returns_null() {
        let result = CelValue::cel_to_double("not_a_double").unwrap();
        assert_eq!(result, CelValue::Null);
    }

    #[test]
    fn celvalue_to_double_from_integer_number() {
        let result = CelValue::cel_to_double(42i32).unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::F64(42.0)));
    }

    #[test]
    fn celvalue_to_double_from_f64_number() {
        let result = CelValue::cel_to_double(std::f64::consts::PI).unwrap();
        assert_eq!(result, CelValue::Number(NumberTy::F64(std::f64::consts::PI)));
    }

    #[test]
    fn celvalue_to_double_from_nan() {
        let err = CelValue::cel_to_double(&[1, 2, 3][..]).unwrap_err();
        if let CelError::BadUnaryOperation { op, value } = err {
            assert_eq!(op, "double");
            assert_eq!(value, (&[1, 2, 3][..]).conv());
        } else {
            panic!("Expected BadUnaryOperation for non-string/number");
        }
    }

    #[test]
    fn celvalue_to_enum_from_number_and_string() {
        let v = CelValue::cel_to_enum(10i32, "MyEnum").unwrap();
        assert_eq!(v, CelValue::Enum(CelEnum::new("MyEnum".into(), 10)));
    }

    #[test]
    fn celvalue_to_enum_number_out_of_range() {
        let overflow = i32::MAX as i64 + 1;
        let v = CelValue::cel_to_enum(overflow, "Tag").unwrap();
        assert_eq!(v, CelValue::Null);
    }

    #[test]
    fn celvalue_to_enum_from_enum_and_string() {
        let original = CelValue::Enum(CelEnum::new("Orig".into(), 42));
        let v = CelValue::cel_to_enum(original.clone(), "NewTag").unwrap();
        assert_eq!(v, CelValue::Enum(CelEnum::new("NewTag".into(), 42)));
    }

    #[test]
    fn celvalue_to_enum_bad_operation_for_invalid_inputs() {
        let err = CelValue::cel_to_enum(true, 123i32).unwrap_err();
        if let CelError::BadOperation { op, left, right } = err {
            assert_eq!(op, "enum");
            assert_eq!(left, true.conv());
            assert_eq!(right, 123i32.conv());
        } else {
            panic!("Expected BadOperation for invalid cel_to_enum inputs");
        }
    }

    #[test]
    fn celvalue_eq_bool_variants() {
        assert_eq!(CelValue::Bool(true), CelValue::Bool(true));
        assert_ne!(CelValue::Bool(true), CelValue::Bool(false));
    }

    #[test]
    fn celvalue_eq_string_and_bytes_variants() {
        let s1 = "abc".conv();
        let s2 = "abc".conv();
        let b1 = Bytes::from_static(b"abc").conv();
        let b2 = Bytes::from_static(b"abc").conv();
        assert_eq!(s1, s2);
        assert_eq!(b1, b2);

        assert_eq!(s1.clone(), b1.clone());
        assert_eq!(b1, s2);
    }

    #[test]
    fn celvalue_eq_duration_and_number() {
        let dur = CelValue::Duration(chrono::Duration::seconds(5));
        let num = 5i32.conv();

        assert_eq!(dur.clone(), num.clone());
        assert_eq!(num, dur);
    }

    #[test]
    fn celvalue_eq_duration_variants() {
        use chrono::Duration;

        let d1 = CelValue::Duration(Duration::seconds(42));
        let d2 = CelValue::Duration(Duration::seconds(42));
        let d3 = CelValue::Duration(Duration::seconds(43));

        assert_eq!(d1, d2, "Two identical Durations should be equal");
        assert_ne!(d1, d3, "Different Durations should not be equal");
    }

    #[test]
    fn celvalue_eq_timestamp_variants() {
        use chrono::{DateTime, FixedOffset};

        let dt1: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2021-01-01T12:00:00+00:00").unwrap();
        let dt2: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2021-01-01T12:00:00+00:00").unwrap();

        let t1 = CelValue::Timestamp(dt1);
        let t2 = CelValue::Timestamp(dt2);
        assert_eq!(t1, t2);
    }

    #[test]
    fn celvalue_eq_enum_and_number_variants() {
        let e = CelValue::Enum(CelEnum::new("Tag".into(), 42));
        let n = 42i32.conv();

        assert_eq!(e.clone(), n.clone());
        assert_eq!(n, e);
    }

    #[test]
    fn celvalue_eq_list_and_map_variants() {
        let list1 = (&[1, 2, 3][..]).conv();
        let list2 = (&[1, 2, 3][..]).conv();
        assert_eq!(list1, list2);

        let map1 = CelValue::Map(Arc::from(vec![(1i32.conv(), 10i32.conv()), (2i32.conv(), 20i32.conv())]));
        let map2 = CelValue::Map(Arc::from(vec![(1i32.conv(), 10i32.conv()), (2i32.conv(), 20i32.conv())]));
        assert_eq!(map1, map2);
    }

    #[test]
    fn celvalue_eq_number_and_null_variants() {
        assert_eq!(1i32.conv(), 1i32.conv());
        assert_ne!(1i32.conv(), 2i32.conv());
        assert_eq!(CelValue::Null, CelValue::Null);
    }

    #[test]
    fn celvalue_eq_mismatched_variants() {
        assert_ne!(CelValue::Bool(true), 1i32.conv());
        assert_ne!(
            CelValue::List(Arc::from(vec![].into_boxed_slice())),
            CelValue::Map(Arc::from(vec![].into_boxed_slice()))
        );
    }

    #[test]
    fn celvalue_conv_unit_conv() {
        let v: CelValue = ().conv();
        assert_eq!(v, CelValue::Null);
    }

    #[test]
    fn celvalue_display() {
        let ts: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2025-05-04T00:00:00+00:00").unwrap();

        // Build a simple map: {1: "x", 2: "y"}
        let map_val = CelValue::Map(Arc::from(vec![(1i32.conv(), "x".conv()), (2i32.conv(), "y".conv())]));

        let outputs = vec![
            format!("{}", CelValue::Bool(false)),
            format!("{}", 42i32.conv()),
            format!("{}", "foo".conv()),
            format!("{}", Bytes::from_static(b"bar").conv()),
            format!("{}", (&[1, 2, 3][..]).conv()),
            format!("{}", CelValue::Null),
            format!("{}", CelValue::Duration(Duration::seconds(5))),
            format!("{}", CelValue::Timestamp(ts)),
            format!("{}", map_val),
        ]
        .join("\n");

        insta::assert_snapshot!(outputs, @r###"
        false
        42
        foo
        [98, 97, 114]
        [1, 2, 3]
        null
        PT5S
        2025-05-04 00:00:00 +00:00
        {1: x, 2: y}
        "###);
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn celvalue_display_enum_runtime() {
        use crate::CelMode;

        CelMode::set(CelMode::Proto);

        let enum_val = CelValue::Enum(CelEnum::new(CelString::Owned("MyTag".into()), 123));
        assert_eq!(format!("{enum_val}"), "123");

        CelMode::set(CelMode::Json);
        let enum_val_json = CelValue::Enum(CelEnum::new(CelString::Owned("MyTag".into()), 456));
        assert_eq!(format!("{enum_val_json}"), "456");
    }

    #[test]
    fn celvalue_to_bool_all_variants() {
        // Bool
        assert!(CelValue::Bool(true).to_bool());
        assert!(!CelValue::Bool(false).to_bool());

        // Number
        assert!(42i32.conv().to_bool());
        assert!(!0i32.conv().to_bool());

        // String
        assert!(CelValue::String(CelString::Borrowed("hello")).to_bool());
        assert!(!CelValue::String(CelString::Borrowed("")).to_bool());

        // Bytes
        assert!(Bytes::from_static(b"x").conv().to_bool());
        assert!(!Bytes::from_static(b"").conv().to_bool());

        // List
        let non_empty_list = (&[1, 2, 3][..]).conv();
        assert!(non_empty_list.to_bool());
        let empty_list = CelValue::List(Arc::from(Vec::<CelValue>::new().into_boxed_slice()));
        assert!(!empty_list.to_bool());

        // Map
        let non_empty_map = CelValue::Map(Arc::from(vec![(1i32.conv(), 2i32.conv())]));
        assert!(non_empty_map.to_bool());
        let empty_map = CelValue::Map(Arc::from(Vec::<(CelValue, CelValue)>::new().into_boxed_slice()));
        assert!(!empty_map.to_bool());

        // Null
        assert!(!CelValue::Null.to_bool());

        // Duration
        assert!(CelValue::Duration(Duration::seconds(5)).to_bool());
        assert!(!CelValue::Duration(Duration::zero()).to_bool());

        // Timestamp
        let epoch: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("1970-01-01T00:00:00+00:00").unwrap();
        assert!(!CelValue::Timestamp(epoch).to_bool());
        let later: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2025-05-04T00:00:00+00:00").unwrap();
        assert!(CelValue::Timestamp(later).to_bool());
    }

    #[test]
    fn numberty_partial_cmp_i64_variants() {
        let a = NumberTy::I64(1);
        let b = NumberTy::I64(2);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        assert_eq!(b.partial_cmp(&a), Some(Ordering::Greater));
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
    }

    #[test]
    fn numberty_partial_cmp_u64_variants() {
        let a = NumberTy::U64(10);
        let b = NumberTy::U64(20);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        assert_eq!(b.partial_cmp(&a), Some(Ordering::Greater));
        assert_eq!(b.partial_cmp(&b), Some(Ordering::Equal));
    }

    #[test]
    fn numberty_partial_cmp_mixed_i64_u64() {
        let a = NumberTy::I64(3);
        let b = NumberTy::U64(4);
        // promoted to I64 comparison
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        assert_eq!(b.partial_cmp(&a), Some(Ordering::Greater));

        let c = NumberTy::I64(5);
        let d = NumberTy::U64(5);
        assert_eq!(c.partial_cmp(&d), Some(Ordering::Equal));
    }

    #[test]
    fn numberty_partial_cmp_f64_exact_and_order() {
        let x = NumberTy::F64(1.23);
        let y = NumberTy::F64(1.23);
        let z = NumberTy::F64(4.56);

        assert_eq!(x.partial_cmp(&y), Some(Ordering::Equal));
        assert_eq!(x.partial_cmp(&z), Some(Ordering::Less));
        assert_eq!(z.partial_cmp(&x), Some(Ordering::Greater));
    }

    #[test]
    fn numberty_partial_cmp_mixed_f64_and_integer() {
        let f = NumberTy::F64(2.0);
        let i = NumberTy::I64(2);
        // promoted to F64 and compared
        assert_eq!(f.partial_cmp(&i), Some(Ordering::Equal));
        assert_eq!(i.partial_cmp(&f), Some(Ordering::Equal));
    }

    #[test]
    fn numberty_cel_add_i64_success() {
        let a = NumberTy::I64(5);
        let b = NumberTy::I64(7);
        assert_eq!(a.cel_add(b).unwrap(), NumberTy::I64(12));
    }

    #[test]
    fn numberty_cel_add_i64_overflow_errors() {
        let a = NumberTy::I64(i64::MAX);
        let b = NumberTy::I64(1);
        let err = a.cel_add(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="addition"));
    }

    #[test]
    fn numberty_cel_add_u64_success() {
        let a = NumberTy::U64(10);
        let b = NumberTy::U64(20);
        assert_eq!(a.cel_add(b).unwrap(), NumberTy::U64(30));
    }

    #[test]
    fn numberty_cel_add_f64_success() {
        let a = NumberTy::F64(1.5);
        let b = NumberTy::F64(2.25);
        assert_eq!(a.cel_add(b).unwrap(), NumberTy::F64(3.75));
    }

    #[test]
    fn numberty_cel_sub_i64_underflow_errors() {
        let a = NumberTy::I64(i64::MIN);
        let b = NumberTy::I64(1);
        let err = a.cel_sub(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="subtraction"));
    }

    #[test]
    fn numberty_cel_sub_u64_underflow_errors() {
        let a = NumberTy::U64(0);
        let b = NumberTy::U64(1);
        let err = a.cel_sub(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="subtraction"));
    }

    #[test]
    fn numberty_cel_sub_f64_success() {
        let a = NumberTy::F64(5.5);
        let b = NumberTy::F64(2.25);
        assert_eq!(a.cel_sub(b).unwrap(), NumberTy::F64(3.25));
    }

    #[test]
    fn numberty_cel_mul_i64_overflow_errors() {
        let a = NumberTy::I64(i64::MAX / 2 + 1);
        let b = NumberTy::I64(2);
        let err = a.cel_mul(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="multiplication"));
    }

    #[test]
    fn numberty_cel_mul_u64_overflow_errors() {
        let a = NumberTy::U64(u64::MAX / 2 + 1);
        let b = NumberTy::U64(2);
        let err = a.cel_mul(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="multiplication"));
    }

    #[test]
    fn numberty_cel_mul_f64_success() {
        let a = NumberTy::F64(3.0);
        let b = NumberTy::F64(2.5);
        assert_eq!(a.cel_mul(b).unwrap(), NumberTy::F64(7.5));
    }

    #[test]
    fn numberty_cel_div_by_zero_errors() {
        let a = NumberTy::I64(10);
        let b = NumberTy::I64(0);
        let err = a.cel_div(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="division by zero"));
    }

    #[test]
    fn numberty_cel_div_i64_success() {
        let a = NumberTy::I64(10);
        let b = NumberTy::I64(2);
        assert_eq!(a.cel_div(b).unwrap(), NumberTy::I64(5));
    }

    #[test]
    fn numberty_cel_div_u64_success() {
        let a = NumberTy::U64(20);
        let b = NumberTy::U64(5);
        assert_eq!(a.cel_div(b).unwrap(), NumberTy::U64(4));
    }

    #[test]
    fn numberty_cel_div_f64_success() {
        let a = NumberTy::F64(9.0);
        let b = NumberTy::F64(2.0);
        assert_eq!(a.cel_div(b).unwrap(), NumberTy::F64(4.5));
    }

    #[test]
    fn numberty_cel_rem_by_zero_errors() {
        let a = NumberTy::I64(10);
        let b = NumberTy::I64(0);
        let err = a.cel_rem(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="remainder by zero"));
    }

    #[test]
    fn numberty_cel_rem_i64_success() {
        let a = NumberTy::I64(10);
        let b = NumberTy::I64(3);
        assert_eq!(a.cel_rem(b).unwrap(), NumberTy::I64(1));
    }

    #[test]
    fn numberty_cel_rem_u64_success() {
        let a = NumberTy::U64(10);
        let b = NumberTy::U64(3);
        assert_eq!(a.cel_rem(b).unwrap(), NumberTy::U64(1));
    }

    #[test]
    fn numberty_cel_rem_f64_errors() {
        let a = NumberTy::F64(10.0);
        let b = NumberTy::F64(3.0);
        let err = a.cel_rem(b).unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="remainder"));
    }

    #[test]
    fn numberty_cel_neg_i64_success() {
        let a = NumberTy::I64(5);
        assert_eq!(a.cel_neg().unwrap(), NumberTy::I64(-5));
    }

    #[test]
    fn numberty_cel_neg_i64_overflow_errors() {
        let a = NumberTy::I64(i64::MIN);
        let err = a.cel_neg().unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="negation"));
    }

    #[test]
    fn numberty_cel_neg_u64_success() {
        let a = NumberTy::U64(5);
        assert_eq!(a.cel_neg().unwrap(), NumberTy::I64(-5));
    }

    #[test]
    fn numberty_cel_neg_u64_overflow_errors() {
        let a = NumberTy::U64(1 << 63); // too large for i64
        let err = a.cel_neg().unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="negation"));
    }

    #[test]
    fn numberty_cel_neg_f64_success() {
        let a = NumberTy::F64(2.5);
        assert_eq!(a.cel_neg().unwrap(), NumberTy::F64(-2.5));
    }

    #[test]
    fn numberty_to_int_success_and_error() {
        assert_eq!(NumberTy::I64(42).to_int().unwrap(), NumberTy::I64(42));
        let err = NumberTy::F64(f64::INFINITY).to_int().unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="int"));
    }

    #[test]
    fn numberty_to_uint_success_and_error() {
        assert_eq!(NumberTy::I64(42).to_uint().unwrap(), NumberTy::U64(42));
        let err = NumberTy::I64(-1).to_uint().unwrap_err();
        assert!(matches!(err, CelError::NumberOutOfRange { op } if op=="int"));
    }

    #[test]
    fn numberty_to_double_always_success() {
        assert_eq!(NumberTy::I64(3).to_double().unwrap(), NumberTy::F64(3.0));
        assert_eq!(NumberTy::U64(4).to_double().unwrap(), NumberTy::F64(4.0));
        assert_eq!(NumberTy::F64(2.5).to_double().unwrap(), NumberTy::F64(2.5));
    }

    #[test]
    fn numberty_from_u32_creates_u64_variant() {
        let input: u32 = 123;
        let nt: NumberTy = input.into();
        assert_eq!(nt, NumberTy::U64(123));
    }

    #[test]
    fn numberty_from_i64_creates_i64_variant() {
        let input: i64 = -42;
        let nt: NumberTy = input.into();
        assert_eq!(nt, NumberTy::I64(-42));
    }

    #[test]
    fn numberty_from_u64_creates_u64_variant() {
        let input: u64 = 9876543210;
        let nt: NumberTy = input.into();
        assert_eq!(nt, NumberTy::U64(9876543210));
    }

    #[test]
    fn numberty_from_f32_matches_raw_cast_to_f64() {
        let input: f32 = 1.23;
        let expected = input as f64;
        let nt: NumberTy = input.into();
        match nt {
            NumberTy::F64(val) => assert_eq!(val, expected),
            _ => panic!("Expected F64 variant"),
        }
    }

    #[test]
    fn numberty_conv_wraps_into_celvalue_number() {
        let nt = NumberTy::I64(-5);
        let cv: CelValue = nt.conv();
        assert_eq!(cv, CelValue::Number(NumberTy::I64(-5)));
    }

    #[test]
    fn array_access_valid_index_returns_element() {
        let arr = [10, 20, 30];
        // using u32 index
        let v = array_access(&arr, 1u32).unwrap();
        assert_eq!(*v, 20);

        // using i64 index
        let v2 = array_access(&arr, 2i64).unwrap();
        assert_eq!(*v2, 30);
    }

    #[test]
    fn array_access_index_out_of_bounds_errors() {
        let arr = [1, 2];
        let err = array_access(&arr, 5i32).unwrap_err();
        if let CelError::IndexOutOfBounds(idx, len) = err {
            assert_eq!(idx, 5);
            assert_eq!(len, 2);
        } else {
            panic!("Expected IndexOutOfBounds, got {err:?}");
        }
    }

    #[test]
    fn array_access_non_numeric_index_errors() {
        let arr = [100, 200];
        let err = array_access(&arr, "not_a_number").unwrap_err();
        if let CelError::IndexWithBadIndex(value) = err {
            assert_eq!(value, "not_a_number".conv());
        } else {
            panic!("Expected IndexWithBadIndex, got {err:?}");
        }
    }

    #[test]
    fn celvalue_eq_string_and_string_conv() {
        let cv = CelValue::String(CelString::Owned(Arc::from("hello")));
        let s = "hello".to_string();
        assert_eq!(cv, s);
        assert_eq!(s, cv);
    }

    #[test]
    fn celvalue_eq_i32_and_conv() {
        let cv = 42i32.conv();
        assert_eq!(cv, 42i32);
        assert_eq!(42i32, cv);
    }

    #[test]
    fn celvalue_eq_i64_and_conv() {
        let cv = 123i64.conv();
        assert_eq!(cv, 123i64);
        assert_eq!(123i64, cv);
    }

    #[test]
    fn celvalue_eq_u32_and_conv() {
        let cv = 7u32.conv();
        assert_eq!(cv, 7u32);
        assert_eq!(7u32, cv);
    }

    #[test]
    fn celvalue_eq_u64_and_conv() {
        let cv = 99u64.conv();
        assert_eq!(cv, 99u64);
        assert_eq!(99u64, cv);
    }

    #[test]
    fn celvalue_eq_f32_and_conv() {
        let cv = 1.5f32.conv();
        assert!(cv == 1.5f32);
        assert!(1.5f32 == cv);
    }

    #[test]
    fn celvalue_eq_f64_and_conv() {
        let cv = 2.75f64.conv();
        assert_eq!(cv, 2.75f64);
        assert_eq!(2.75f64, cv);
    }

    #[test]
    fn celvalue_eq_vec_u8_and_conv() {
        let vec = vec![10u8, 20, 30];
        let cv = (&vec).conv();
        assert_eq!(cv, vec);
        assert_eq!(vec, cv);
    }

    #[test]
    fn celvalue_eq_bytes_variant() {
        let b = Bytes::from_static(b"xyz");
        let cv = CelValue::Bytes(CelBytes::Owned(b.clone()));
        assert_eq!(cv, b);
    }

    #[test]
    fn bytes_eq_celvalue_variant() {
        let b = Bytes::from_static(b"hello");
        let cv = CelValue::Bytes(CelBytes::Owned(b.clone()));
        assert_eq!(b, cv);
    }

    #[test]
    fn array_contains_with_integers() {
        let arr = [1i32, 2, 3];
        assert!(array_contains(&arr, 2i32));
        assert!(!array_contains(&arr, 4i32));
    }

    #[test]
    fn array_contains_with_bytes() {
        let b1 = Bytes::from_static(b"a");
        let b2 = Bytes::from_static(b"b");
        let arr = [b1.clone(), b2.clone()];
        assert!(array_contains(&arr, b2.clone()));
        assert!(!array_contains(&arr, Bytes::from_static(b"c")));
    }

    #[test]
    fn map_access_and_contains_with_hashmap_i32_key() {
        let mut hm: HashMap<i32, &str> = HashMap::new();
        hm.insert(5, "five");

        let v = map_access(&hm, 5i32).unwrap();
        assert_eq!(*v, "five");

        assert!(map_contains(&hm, 5i32));
        assert!(!map_contains(&hm, 6i32));
    }

    #[test]
    fn map_access_and_contains_with_btreemap_u32_key() {
        let mut bt: BTreeMap<u32, &str> = BTreeMap::new();
        bt.insert(10, "ten");

        let v = map_access(&bt, 10u32).unwrap();
        assert_eq!(*v, "ten");

        assert!(map_contains(&bt, 10u32));
        assert!(!map_contains(&bt, 11u32));
    }

    #[test]
    fn map_access_key_not_found_errors() {
        let mut hm: HashMap<i32, &str> = HashMap::new();
        hm.insert(1, "one");

        let err = map_access(&hm, 2i32).unwrap_err();
        if let CelError::MapKeyNotFound(k) = err {
            assert_eq!(k, 2i32.conv());
        } else {
            panic!("Expected MapKeyNotFound");
        }
    }

    #[test]
    fn map_key_cast_string_some_for_borrowed() {
        let cv = "hello".conv();
        let key: Option<Cow<str>> = <String as MapKeyCast>::make_key(&cv);
        match key {
            Some(Cow::Borrowed(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected Some(Cow::Borrowed)"),
        }
    }

    #[test]
    fn map_key_cast_string_some_for_owned() {
        let arc: Arc<str> = Arc::from("world");
        let cv = CelValue::String(CelString::Owned(arc.clone()));
        let key: Option<Cow<str>> = <String as MapKeyCast>::make_key(&cv);
        match key {
            Some(Cow::Borrowed(s)) => assert_eq!(s, "world"),
            _ => panic!("Expected Some(Cow::Borrowed)"),
        }
    }

    #[test]
    fn map_key_cast_string_none_for_non_string() {
        let cv = 42i32.conv();
        assert!(<String as MapKeyCast>::make_key(&cv).is_none());
    }

    #[test]
    fn map_key_cast_number_none_for_non_number_value() {
        let cv = "not_a_number".conv();
        let result: Option<Cow<'_, i32>> = <i32 as MapKeyCast>::make_key(&cv);
        assert!(result.is_none(), "Expected None for non-Number CelValue");
    }

    #[test]
    fn option_to_bool() {
        assert!(Some(true).to_bool(), "Some(true) should be true");
        assert!(!Some(false).to_bool(), "Some(false) should be false");
        let none: Option<bool> = None;
        assert!(!none.to_bool(), "None should be false");
    }

    #[test]
    fn vec_to_bool() {
        let empty: Vec<i32> = Vec::new();
        assert!(!empty.to_bool(), "Empty Vec should be false");
        let non_empty = vec![1, 2, 3];
        assert!(non_empty.to_bool(), "Non-empty Vec should be true");
    }

    #[test]
    fn btreemap_to_bool() {
        let mut map: BTreeMap<i32, i32> = BTreeMap::new();
        assert!(!map.to_bool(), "Empty BTreeMap should be false");
        map.insert(1, 10);
        assert!(map.to_bool(), "Non-empty BTreeMap should be true");
    }

    #[test]
    fn hashmap_to_bool() {
        let mut map: HashMap<&str, i32> = HashMap::new();
        assert!(!map.to_bool(), "Empty HashMap should be false");
        map.insert("key", 42);
        assert!(map.to_bool(), "Non-empty HashMap should be true");
    }

    #[test]
    fn str_and_string_to_bool() {
        assert!("hello".to_bool(), "Non-empty &str should be true");
        assert!(!"".to_bool(), "Empty &str should be false");
        let s = String::from("world");
        assert!(s.to_bool(), "Non-empty String should be true");
        let empty = String::new();
        assert!(!empty.to_bool(), "Empty String should be false");
    }

    #[test]
    fn array_slice_to_bool() {
        let empty: [bool; 0] = [];
        assert!(!empty.to_bool(), "Empty [T] slice should be false");
        let non_empty = [true, false];
        assert!(non_empty.to_bool(), "Non-empty [T] slice should be true");
    }

    #[test]
    fn bytes_to_bool() {
        let empty = Bytes::new();
        assert!(!empty.to_bool(), "Empty Bytes should be false");
        let non_empty = Bytes::from_static(b"x");
        assert!(non_empty.to_bool(), "Non-empty Bytes should be true");
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn celmode_json_and_proto_flags() {
        use crate::CelMode;

        CelMode::set(CelMode::Json);
        let current = CelMode::current();
        assert!(current.is_json(), "CelMode should report JSON when set to Json");
        assert!(!current.is_proto(), "CelMode should not report Proto when set to Json");

        CelMode::set(CelMode::Proto);
        let current = CelMode::current();
        assert!(current.is_proto(), "CelMode should report Proto when set to Proto");
        assert!(!current.is_json(), "CelMode should not report JSON when set to Proto");
    }
}
