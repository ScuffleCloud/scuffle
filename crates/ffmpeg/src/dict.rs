use core::ffi::CStr;
use std::borrow::Cow;
use std::ffi::CString;
use std::ptr::NonNull;

use crate::AVDictionaryFlags;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::smart_object::SmartPtr;

/// A dictionary of key-value pairs.
pub struct Dictionary {
    ptr: SmartPtr<AVDictionary>,
}

/// Safety: `Dictionary` is safe to send between threads.
unsafe impl Send for Dictionary {}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Dictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();

        for (key, value) in self.iter() {
            map.entry(&key, &value);
        }

        map.finish()
    }
}

impl Clone for Dictionary {
    fn clone(&self) -> Self {
        let mut dict = Self::new();

        Self::clone_from(&mut dict, self);

        dict
    }

    fn clone_from(&mut self, source: &Self) {
        // Safety: av_dict_copy is safe to call
        FfmpegErrorCode::from(unsafe { av_dict_copy(self.as_mut_ptr_ref(), source.as_ptr(), 0) })
            .result()
            .expect("Failed to clone dictionary");
    }
}

/// A trait for types that can be converted to a `CStr`.
///
/// This is used to allow for a few different types:
/// - [`&str`] - Will be copied and converted to a `CString`.
/// - [`CStr`] - Will be borrowed.
/// - [`String`] - Will be copied and converted to a `CString`.
/// - [`CString`] - Will be owned.
///
/// If the string is empty, the [`Option::None`] will be returned.
///
/// # Examples
///
/// ```rust
/// use scuffle_ffmpeg::dict::Dictionary;
///
/// let mut dict = Dictionary::new();
///
/// // "key" is a &CStr, so it will be borrowed.
/// dict.set(c"key", c"value").expect("Failed to set key");
///
/// // "key" is a &str, so it will be copied and converted to a CString.
/// assert_eq!(dict.get("key"), Some(c"value"));
///
/// // "nonexistent_key" is a &str, so it will be copied and converted to a CString.
/// assert_eq!(dict.set("nonexistent_key".to_owned(), "value"), Ok(()));
///
/// // "nonexistent_key" is a CString, so it will be borrowed.
/// assert_eq!(dict.get(c"nonexistent_key".to_owned()), Some(c"value"));
/// ```
pub trait CStringLike<'a> {
    /// Convert the type to a `CStr`.
    fn into_c_str(self) -> Option<Cow<'a, CStr>>;
}

impl<'a> CStringLike<'a> for String {
    fn into_c_str(self) -> Option<Cow<'a, CStr>> {
        if self.is_empty() {
            return None;
        }

        Some(Cow::Owned(CString::new(Vec::from(self)).ok()?))
    }
}

impl<'a> CStringLike<'a> for &str {
    fn into_c_str(self) -> Option<Cow<'a, CStr>> {
        if self.is_empty() {
            return None;
        }

        Some(Cow::Owned(CString::new(self.as_bytes().to_vec()).ok()?))
    }
}

impl<'a> CStringLike<'a> for &'a CStr {
    fn into_c_str(self) -> Option<Cow<'a, CStr>> {
        if self.is_empty() {
            return None;
        }

        Some(Cow::Borrowed(self))
    }
}

impl<'a> CStringLike<'a> for CString {
    fn into_c_str(self) -> Option<Cow<'a, CStr>> {
        if self.is_empty() {
            return None;
        }

        Some(Cow::Owned(self))
    }
}

impl Dictionary {
    /// Creates a new dictionary.
    pub const fn new() -> Self {
        Self {
            // Safety: A null pointer is a valid dictionary, and a valid pointer.
            ptr: SmartPtr::null(|ptr| {
                // Safety: av_dict_free is safe to call
                unsafe { av_dict_free(ptr) }
            }),
        }
    }

    /// Wrap a pointer to a [`AVDictionary`] in a [`Dictionary`].
    /// Without taking ownership of the dictionary.
    /// # Safety
    /// `ptr` must be a valid pointer.
    /// The caller must also ensure that the dictionary is not freed while this
    /// object is alive, and that we don't use the pointer as mutable
    pub const unsafe fn from_ptr_ref(ptr: *mut AVDictionary) -> Self {
        Self {
            // Safety: The safety comment of the function implies this is safe.
            // We don't own the dictionary, so we don't need to free it
            ptr: unsafe { SmartPtr::wrap(ptr as _, |_| {}) },
        }
    }

    /// Wrap a pointer to a [`AVDictionary`] in a [`Dictionary`].
    /// Takes ownership of the dictionary.
    /// Meaning it will be freed when the [`Dictionary`] is dropped.
    /// # Safety
    /// `ptr` must be a valid pointer.
    pub const unsafe fn from_ptr_owned(ptr: *mut AVDictionary) -> Self {
        let destructor = |ptr: &mut *mut AVDictionary| {
            // Safety: av_dict_free is safe to call & we own the pointer.
            unsafe { av_dict_free(ptr) }
        };

        Self {
            // Safety: The safety comment of the function implies this is safe.
            ptr: unsafe { SmartPtr::wrap(ptr, destructor) },
        }
    }

    /// Sets a key-value pair in the dictionary.
    /// Key and value must not be empty.
    pub fn set<'a>(&mut self, key: impl CStringLike<'a>, value: impl CStringLike<'a>) -> Result<(), FfmpegError> {
        let key = key.into_c_str().ok_or(FfmpegError::Arguments("key cannot be empty"))?;
        let value = value.into_c_str().ok_or(FfmpegError::Arguments("value cannot be empty"))?;

        // Safety: av_dict_set is safe to call
        FfmpegErrorCode(unsafe { av_dict_set(self.ptr.as_mut(), key.as_ptr() as *const _, value.as_ptr() as *const _, 0) })
            .result()?;
        Ok(())
    }

    /// Returns the value associated with the given key.
    /// If the key is not found, the [`Option::None`] will be returned.
    pub fn get<'a>(&self, key: impl CStringLike<'a>) -> Option<&CStr> {
        let key = key.into_c_str()?;

        let mut entry =
            // Safety: av_dict_get is safe to call
            NonNull::new(unsafe {
                av_dict_get(
                    self.as_ptr(),
                    key.as_ptr() as *const _,
                    std::ptr::null_mut(),
                    AVDictionaryFlags::IgnoreSuffix.into(),
                )
            })?;

        // Safety: The pointer here is valid.
        let mut_ref = unsafe { entry.as_mut() };

        // Safety: The pointer here is valid.
        Some(unsafe { CStr::from_ptr(mut_ref.value as *const _) })
    }

    /// Returns true if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// Returns an iterator over the dictionary.
    pub const fn iter(&self) -> DictionaryIterator<'_> {
        DictionaryIterator::new(self)
    }

    /// Returns the pointer to the dictionary.
    pub const fn as_ptr(&self) -> *const AVDictionary {
        self.ptr.as_ptr()
    }

    /// Returns a mutable reference to the pointer to the dictionary.
    pub const fn as_mut_ptr_ref(&mut self) -> &mut *mut AVDictionary {
        self.ptr.as_mut()
    }

    /// Returns the pointer to the dictionary.
    pub fn leak(self) -> *mut AVDictionary {
        self.ptr.into_inner()
    }

    /// Extends a dictionary with an iterator of key-value pairs.
    pub fn extend<'a, K, V>(&mut self, iter: impl IntoIterator<Item = (K, V)>) -> Result<(), FfmpegError>
    where
        K: CStringLike<'a>,
        V: CStringLike<'a>,
    {
        for (key, value) in iter {
            // This is less then ideal, we shouldnt ignore the error but it only happens if the key or value is empty.
            self.set(key, value)?;
        }

        Ok(())
    }

    /// Creates a new dictionary from an iterator of key-value pairs.
    pub fn try_from_iter<'a, K, V>(iter: impl IntoIterator<Item = (K, V)>) -> Result<Self, FfmpegError>
    where
        K: CStringLike<'a>,
        V: CStringLike<'a>,
    {
        let mut dict = Self::new();
        dict.extend(iter)?;
        Ok(dict)
    }
}

/// An iterator over the dictionary.
pub struct DictionaryIterator<'a> {
    dict: &'a Dictionary,
    entry: *mut AVDictionaryEntry,
}

impl<'a> DictionaryIterator<'a> {
    /// Creates a new dictionary iterator.
    const fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            entry: std::ptr::null_mut(),
        }
    }
}

impl<'a> Iterator for DictionaryIterator<'a> {
    type Item = (&'a CStr, &'a CStr);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: av_dict_get is safe to call
        self.entry = unsafe {
            av_dict_get(
                self.dict.as_ptr(),
                // ffmpeg expects a null terminated string when iterating over the dictionary entries.
                c"".as_ptr() as *const _,
                self.entry,
                AVDictionaryFlags::IgnoreSuffix.into(),
            )
        };

        let mut entry = NonNull::new(self.entry)?;

        // Safety: The pointer here is valid.
        let entry_ref = unsafe { entry.as_mut() };

        // Safety: The pointer here is valid.
        let key = unsafe { CStr::from_ptr(entry_ref.key as *const _) };
        // Safety: The pointer here is valid.
        let value = unsafe { CStr::from_ptr(entry_ref.value as *const _) };

        Some((key, value))
    }
}

impl<'a> IntoIterator for &'a Dictionary {
    type IntoIter = DictionaryIterator<'a>;
    type Item = <DictionaryIterator<'a> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryIterator::new(self)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {

    use std::collections::HashMap;
    use std::ffi::CStr;

    use crate::dict::Dictionary;

    fn sort_hashmap<K: Ord, V>(map: std::collections::HashMap<K, V>) -> std::collections::BTreeMap<K, V> {
        map.into_iter().collect()
    }

    #[test]
    fn test_dict_default_and_items() {
        let mut dict = Dictionary::default();

        assert!(dict.is_empty(), "Default dictionary should be empty");
        assert!(dict.as_ptr().is_null(), "Default dictionary pointer should be null");

        dict.set(c"key1", c"value1").expect("Failed to set key1");
        dict.set(c"key2", c"value2").expect("Failed to set key2");
        dict.set(c"key3", c"value3").expect("Failed to set key3");

        let dict_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&dict);

        insta::assert_debug_snapshot!(sort_hashmap(dict_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
            "key3": "value3",
        }
        "#);
    }

    #[test]
    fn test_dict_set_empty_key() {
        let mut dict = Dictionary::new();
        assert!(dict.set(c"", c"value1").is_err());
    }

    #[test]
    fn test_dict_clone_empty() {
        let empty_dict = Dictionary::new();
        let cloned_dict = empty_dict.clone();

        assert!(cloned_dict.is_empty(), "Cloned dictionary should be empty");
        assert!(empty_dict.is_empty(), "Original dictionary should remain empty");
    }

    #[test]
    fn test_dict_clone_non_empty() {
        let mut dict = Dictionary::new();
        dict.set(c"key1", c"value1").expect("Failed to set key1");
        dict.set(c"key2", c"value2").expect("Failed to set key2");
        let mut clone = dict.clone();

        let dict_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&dict);
        let clone_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&clone);

        insta::assert_debug_snapshot!(sort_hashmap(dict_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
        }
        "#);
        insta::assert_debug_snapshot!(sort_hashmap(clone_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
        }
        "#);

        clone
            .set(c"key3", c"value3")
            .expect("Failed to set key3 in cloned dictionary");

        let dict_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&dict);
        let clone_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&clone);
        insta::assert_debug_snapshot!(sort_hashmap(dict_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
        }
        "#);
        insta::assert_debug_snapshot!(sort_hashmap(clone_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
            "key3": "value3",
        }
        "#);
    }

    #[test]
    fn test_dict_get() {
        let mut dict = Dictionary::new();
        assert!(
            dict.get(c"nonexistent_key").is_none(),
            "Getting a nonexistent key from an empty dictionary should return None"
        );

        dict.set(c"key1", c"value1").expect("Failed to set key1");
        dict.set(c"key2", c"value2").expect("Failed to set key2");
        assert_eq!(dict.get(c"key1"), Some(c"value1"), "The value for 'key1' should be 'value1'");
        assert_eq!(dict.get(c"key2"), Some(c"value2"), "The value for 'key2' should be 'value2'");

        assert!(dict.get(c"key3").is_none(), "Getting a nonexistent key should return None");

        dict.set(c"special_key!", c"special_value")
            .expect("Failed to set special_key!");
        assert_eq!(
            dict.get(c"special_key!"),
            Some(c"special_value"),
            "The value for 'special_key!' should be 'special_value'"
        );

        assert!(
            dict.get(c"").is_none(),
            "Getting an empty key should return None (empty keys are not allowed)"
        );
    }

    #[test]
    fn test_from_hashmap_for_dictionary() {
        let mut hash_map = std::collections::HashMap::new();
        hash_map.insert("key1".to_string(), "value1".to_string());
        hash_map.insert("key2".to_string(), "value2".to_string());
        hash_map.insert("key3".to_string(), "value3".to_string());
        let dict = Dictionary::try_from_iter(hash_map).expect("Failed to create dictionary from hashmap");

        let dict_hm: std::collections::HashMap<&CStr, &CStr> = HashMap::from_iter(&dict);
        insta::assert_debug_snapshot!(sort_hashmap(dict_hm), @r#"
        {
            "key1": "value1",
            "key2": "value2",
            "key3": "value3",
        }
        "#);
    }

    #[test]
    fn test_empty_string() {
        let mut dict = Dictionary::new();
        assert!(dict.set(c"", c"abc").is_err());
        assert!(dict.set(c"abc", c"").is_err());
        assert!(dict.get(c"").is_none());
        assert!(dict.set("".to_owned(), "abc".to_owned()).is_err());
        assert!(dict.set("abc".to_owned(), "".to_owned()).is_err());
        assert!(dict.get("").is_none());
        assert!(dict.set(c"".to_owned(), c"abc".to_owned()).is_err());
        assert!(dict.set(c"abc".to_owned(), c"".to_owned()).is_err());
        assert!(dict.get(c"").is_none());
    }
}
