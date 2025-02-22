#![cfg(feature = "Foundation_NSMutableSet")]
use alloc::vec::Vec;

use objc2::rc::{DefaultId, SliceId};

use super::set::with_objects;
use crate::common::*;
use crate::Foundation::{self, NSMutableSet};

extern_methods!(
    unsafe impl<T: Message, O: Ownership> NSMutableSet<T, O> {
        /// Creates an empty [`NSMutableSet`].
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSString};
        ///
        /// let set = NSMutableSet::<NSString>::new();
        /// ```
        // SAFETY:
        // Same as `NSSet::new`, except mutable sets are always unique.
        #[method_id(new)]
        pub fn new() -> Id<Self, Owned>;

        /// Creates an [`NSMutableSet`] from a vector.
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSString};
        ///
        /// let strs = ["one", "two", "three"].map(NSString::from_str).to_vec();
        /// let set = NSMutableSet::from_vec(strs);
        /// ```
        pub fn from_vec(vec: Vec<Id<T, O>>) -> Id<Self, Owned> {
            // SAFETY:
            // We always return `Id<NSMutableSet<T, O>, Owned>` because mutable
            // sets are always unique.
            unsafe { with_objects(vec.as_slice_ref()) }
        }

        /// Returns a [`Vec`] containing the set's elements, consuming the set.
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSMutableString};
        ///
        /// let strs = vec![
        ///     NSMutableString::from_str("one"),
        ///     NSMutableString::from_str("two"),
        ///     NSMutableString::from_str("three"),
        /// ];
        /// let set = NSMutableSet::from_vec(strs);
        /// let vec = NSMutableSet::into_vec(set);
        /// assert_eq!(vec.len(), 3);
        /// ```
        pub fn into_vec(set: Id<Self, Owned>) -> Vec<Id<T, O>> {
            set.into_iter()
                .map(|obj| unsafe { Id::retain(obj as *const T as *mut T).unwrap_unchecked() })
                .collect()
        }
    }

    unsafe impl<T: Message> NSMutableSet<T, Shared> {
        /// Creates an [`NSMutableSet`] from a slice.
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSString};
        ///
        /// let strs = ["one", "two", "three"].map(NSString::from_str);
        /// let set = NSMutableSet::from_slice(&strs);
        /// ```
        pub fn from_slice(slice: &[Id<T, Shared>]) -> Id<Self, Owned> {
            // SAFETY:
            // Taking `&T` would not be sound, since the `&T` could come from
            // an `Id<T, Owned>` that would now no longer be owned!
            //
            // We always return `Id<NSMutableSet<T, Shared>, Owned>` because
            // the elements are shared and mutable sets are always unique.
            unsafe { with_objects(slice.as_slice_ref()) }
        }
    }

    // We're explicit about `T` being `PartialEq` for these methods because the
    // set compares the input value with elements in the set
    // For comparison: Rust's HashSet requires similar methods to be `Hash` + `Eq`
    unsafe impl<T: Message + PartialEq, O: Ownership> NSMutableSet<T, O> {
        #[method(addObject:)]
        fn add_object(&mut self, value: &T);

        /// Adds a value to the set. Returns whether the value was
        /// newly inserted.
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSString};
        ///
        /// let mut set = NSMutableSet::new();
        ///
        /// assert_eq!(set.insert(NSString::from_str("one")), true);
        /// assert_eq!(set.insert(NSString::from_str("one")), false);
        /// assert_eq!(set.len(), 1);
        /// ```
        #[doc(alias = "addObject:")]
        pub fn insert(&mut self, value: Id<T, O>) -> bool {
            // SAFETY:
            // We take `Id<T, O>` instead of `&T` because `&T` could be a
            // reference to an owned object which would cause us to have a copy
            // of an owned object in our set. By taking `Id<T, O>`, we force the
            // caller to transfer ownership of the value to us, making it safe
            // to insert the owned object into the set.
            let contains_value = self.contains(&value);
            self.add_object(&*value);
            !contains_value
        }

        #[method(removeObject:)]
        fn remove_object(&mut self, value: &T);

        /// Removes a value from the set. Returns whether the value was present
        /// in the set.
        ///
        /// # Examples
        ///
        /// ```
        /// use icrate::Foundation::{NSMutableSet, NSString};
        /// use icrate::ns_string;
        ///
        /// let mut set = NSMutableSet::new();
        ///
        /// set.insert(NSString::from_str("one"));
        /// assert_eq!(set.remove(ns_string!("one")), true);
        /// assert_eq!(set.remove(ns_string!("one")), false);
        /// ```
        #[doc(alias = "removeObject:")]
        pub fn remove(&mut self, value: &T) -> bool {
            let contains_value = self.contains(value);
            self.remove_object(value);
            contains_value
        }
    }
);

unsafe impl<T: Message, O: Ownership> Foundation::NSFastEnumeration2 for NSMutableSet<T, O> {
    type Item = T;
}

impl<'a, T: Message, O: Ownership> IntoIterator for &'a NSMutableSet<T, O> {
    type Item = &'a T;
    type IntoIter = Foundation::NSFastEnumerator2<'a, NSMutableSet<T, O>>;

    fn into_iter(self) -> Self::IntoIter {
        use Foundation::NSFastEnumeration2;
        self.iter_fast()
    }
}

impl<T: Message + PartialEq, O: Ownership> Extend<Id<T, O>> for NSMutableSet<T, O> {
    fn extend<I: IntoIterator<Item = Id<T, O>>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<T: Message, O: Ownership> DefaultId for NSMutableSet<T, O> {
    type Ownership = Owned;

    #[inline]
    fn default_id() -> Id<Self, Self::Ownership> {
        Self::new()
    }
}
