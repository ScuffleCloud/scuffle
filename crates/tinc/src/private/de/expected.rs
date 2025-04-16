use std::collections::{BTreeMap, HashMap};

pub trait Expected {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<V: Expected> Expected for Box<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        V::expecting(formatter)
    }
}

impl<V: Expected> Expected for Option<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an optional `")?;
        V::expecting(formatter)?;
        write!(formatter, "`")
    }
}

impl<V: Expected> Expected for Vec<V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a sequence of `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}

impl<K: Expected, V: Expected> Expected for BTreeMap<K, V> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map of `")?;
        K::expecting(formatter)?;
        write!(formatter, "`s to `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}

impl<K: Expected, V: Expected, S> Expected for HashMap<K, V, S> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map of `")?;
        K::expecting(formatter)?;
        write!(formatter, "`s to `")?;
        V::expecting(formatter)?;
        write!(formatter, "`s")
    }
}
