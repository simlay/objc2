use core::cmp;
use core::fmt;
use core::ops::AddAssign;
use core::str;

use objc2::rc::{DefaultId, Id, Owned, Shared};
use objc2::{extern_methods, ClassType};

use crate::Foundation::{NSCopying, NSMutableCopying, NSMutableString, NSString};

extern_methods!(
    /// Creating mutable strings.
    unsafe impl NSMutableString {
        /// Construct an empty [`NSMutableString`].
        #[method_id(new)]
        pub fn new() -> Id<Self, Owned>;

        /// Creates a new [`NSMutableString`] by copying the given string slice.
        #[doc(alias = "initWithBytes:length:encoding:")]
        #[allow(clippy::should_implement_trait)] // Not really sure of a better name
        pub fn from_str(string: &str) -> Id<Self, Owned> {
            unsafe {
                let obj = super::string::from_str(Self::class(), string);
                Id::new(obj.cast()).unwrap()
            }
        }
    }
);

impl DefaultId for NSMutableString {
    type Ownership = Owned;

    #[inline]
    fn default_id() -> Id<Self, Self::Ownership> {
        Self::new()
    }
}

unsafe impl NSCopying for NSMutableString {
    type Ownership = Shared;
    type Output = NSString;
}

unsafe impl NSMutableCopying for NSMutableString {
    type Output = NSMutableString;
}

impl alloc::borrow::ToOwned for NSMutableString {
    type Owned = Id<NSMutableString, Owned>;
    fn to_owned(&self) -> Self::Owned {
        self.mutable_copy()
    }
}

impl AddAssign<&NSString> for NSMutableString {
    #[inline]
    fn add_assign(&mut self, other: &NSString) {
        self.appendString(other)
    }
}

impl PartialEq<NSString> for NSMutableString {
    #[inline]
    fn eq(&self, other: &NSString) -> bool {
        PartialEq::eq(&**self, other)
    }
}

impl PartialEq<NSMutableString> for NSString {
    #[inline]
    fn eq(&self, other: &NSMutableString) -> bool {
        PartialEq::eq(self, &**other)
    }
}

impl PartialOrd for NSMutableString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl PartialOrd<NSString> for NSMutableString {
    #[inline]
    fn partial_cmp(&self, other: &NSString) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, other)
    }
}

impl PartialOrd<NSMutableString> for NSString {
    #[inline]
    fn partial_cmp(&self, other: &NSMutableString) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self, &**other)
    }
}

impl Ord for NSMutableString {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl fmt::Write for NSMutableString {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let nsstring = NSString::from_str(s);
        self.appendString(&nsstring);
        Ok(())
    }
}

impl fmt::Display for NSMutableString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn display_debug() {
        let s = NSMutableString::from_str("test\"123");
        assert_eq!(format!("{s}"), "test\"123");
        assert_eq!(format!("{s:?}"), r#""test\"123""#);
    }

    #[test]
    fn test_from_nsstring() {
        let s = NSString::from_str("abc");
        let s = NSMutableString::from_nsstring(&s);
        assert_eq!(&s.to_string(), "abc");
    }

    #[test]
    fn test_append() {
        let mut s = NSMutableString::from_str("abc");
        s.appendString(&NSString::from_str("def"));
        *s += &NSString::from_str("ghi");
        assert_eq!(&s.to_string(), "abcdefghi");
    }

    #[test]
    fn test_set() {
        let mut s = NSMutableString::from_str("abc");
        s.setString(&NSString::from_str("def"));
        assert_eq!(&s.to_string(), "def");
    }

    #[test]
    fn test_with_capacity() {
        let mut s = NSMutableString::with_capacity(3);
        *s += &NSString::from_str("abc");
        *s += &NSString::from_str("def");
        assert_eq!(&s.to_string(), "abcdef");
    }

    #[test]
    fn test_copy() {
        let s1 = NSMutableString::from_str("abc");
        let s2 = s1.copy();
        assert_ne!(Id::as_ptr(&s1), Id::as_ptr(&s2).cast());
        assert!(s2.is_kind_of::<NSString>());

        let s3 = s1.mutable_copy();
        assert_ne!(Id::as_ptr(&s1), Id::as_ptr(&s3));
        assert!(s3.is_kind_of::<NSMutableString>());
    }
}
