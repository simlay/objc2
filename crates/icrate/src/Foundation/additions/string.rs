#![cfg(feature = "Foundation_NSString")]
use core::cmp;
use core::ffi::c_void;
use core::fmt;
use core::panic::RefUnwindSafe;
use core::panic::UnwindSafe;
#[cfg(feature = "apple")]
use core::ptr::NonNull;
#[cfg(feature = "apple")]
use core::slice;
use core::str;
#[cfg(feature = "apple")]
use std::os::raw::c_char;

use objc2::msg_send;
use objc2::rc::{autoreleasepool_leaking, AutoreleasePool, DefaultId, Id, Shared};
use objc2::runtime::__nsstring::{nsstring_len, nsstring_to_str, UTF8_ENCODING};

use crate::common::*;
use crate::Foundation::{self, NSString};

// SAFETY: `NSString` is immutable and `NSMutableString` can only be mutated
// from `&mut` methods.
unsafe impl Sync for NSString {}
unsafe impl Send for NSString {}

// Even if an exception occurs inside a string method, the state of the string
// (should) still be perfectly safe to access.
impl UnwindSafe for NSString {}
impl RefUnwindSafe for NSString {}

impl NSString {
    /// Construct an empty NSString.
    pub fn new() -> Id<Self> {
        Self::init(Self::alloc())
    }

    /// The number of UTF-8 code units in `self`.
    #[doc(alias = "lengthOfBytesUsingEncoding")]
    #[doc(alias = "lengthOfBytesUsingEncoding:")]
    pub fn len(&self) -> usize {
        // SAFETY: This is an instance of `NSString`
        unsafe { nsstring_len(self) }
    }

    /// The number of UTF-16 code units in the string.
    ///
    /// See also [`NSString::len`].
    #[doc(alias = "length")]
    pub fn len_utf16(&self) -> usize {
        self.length()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: lengthOfBytesUsingEncoding: might sometimes return 0 for
        // other reasons, so this is not really correct!
        self.len() == 0
    }

    /// Get the [`str`](`prim@str`) representation of this string if it can be
    /// done efficiently.
    ///
    /// Returns [`None`] if the internal storage does not allow this to be
    /// done efficiently. Use [`NSString::as_str`] or `NSString::to_string`
    /// if performance is not an issue.
    #[doc(alias = "CFStringGetCStringPtr")]
    #[allow(unused)]
    #[cfg(feature = "apple")]
    // TODO: Finish this
    fn as_str_wip(&self) -> Option<&str> {
        type CFStringEncoding = u32;
        #[allow(non_upper_case_globals)]
        // https://developer.apple.com/documentation/corefoundation/cfstringbuiltinencodings/kcfstringencodingutf8?language=objc
        const kCFStringEncodingUTF8: CFStringEncoding = 0x08000100;
        extern "C" {
            // https://developer.apple.com/documentation/corefoundation/1542133-cfstringgetcstringptr?language=objc
            fn CFStringGetCStringPtr(s: &NSString, encoding: CFStringEncoding) -> *const c_char;
        }
        let bytes = unsafe { CFStringGetCStringPtr(self, kCFStringEncodingUTF8) };
        NonNull::new(bytes as *mut u8).map(|bytes| {
            let len = self.len();
            let bytes: &[u8] = unsafe { slice::from_raw_parts(bytes.as_ptr(), len) };
            str::from_utf8(bytes).unwrap()
        })
    }

    /// Get an [UTF-16] string slice if it can be done efficiently.
    ///
    /// Returns [`None`] if the internal storage of `self` does not allow this
    /// to be returned efficiently.
    ///
    /// See [`as_str`](Self::as_str) for the UTF-8 equivalent.
    ///
    /// [UTF-16]: https://en.wikipedia.org/wiki/UTF-16
    #[allow(unused)]
    #[cfg(feature = "apple")]
    // TODO: Finish this
    fn as_utf16(&self) -> Option<&[u16]> {
        extern "C" {
            // https://developer.apple.com/documentation/corefoundation/1542939-cfstringgetcharactersptr?language=objc
            fn CFStringGetCharactersPtr(s: &NSString) -> *const u16;
        }
        let ptr = unsafe { CFStringGetCharactersPtr(self) };
        NonNull::new(ptr as *mut u16)
            .map(|ptr| unsafe { slice::from_raw_parts(ptr.as_ptr(), self.len_utf16()) })
    }

    /// Get the [`str`](`prim@str`) representation of this.
    ///
    /// TODO: Further explain this.
    #[doc(alias = "UTF8String")]
    pub fn as_str<'r, 's: 'r, 'p: 'r>(&'s self, pool: AutoreleasePool<'p>) -> &'r str {
        // SAFETY: This is an instance of `NSString`
        unsafe { nsstring_to_str(self, pool) }
    }

    // TODO: Allow usecases where the NUL byte from `UTF8String` is kept?

    /// Creates an immutable `NSString` by copying the given string slice.
    ///
    /// Prefer using the [`ns_string!`] macro when possible.
    ///
    /// [`ns_string!`]: crate::ns_string
    #[doc(alias = "initWithBytes")]
    #[doc(alias = "initWithBytes:length:encoding:")]
    #[allow(clippy::should_implement_trait)] // Not really sure of a better name
    pub fn from_str(string: &str) -> Id<Self> {
        unsafe {
            let obj = from_str(Self::class(), string);
            Id::new(obj.cast()).unwrap()
        }
    }

    // TODO: initWithBytesNoCopy:, maybe add lifetime parameter to NSString?
    // See https://github.com/nvzqz/fruity/blob/320efcf715c2c5fbd2f3084f671f2be2e03a6f2b/src/foundation/ns_string/mod.rs#L350-L381
    // Might be quite difficult, as Objective-C code might assume the NSString
    // is always alive?
    // See https://github.com/drewcrawford/foundationr/blob/b27683417a35510e8e5d78a821f081905b803de6/src/nsstring.rs
}

pub(crate) fn from_str(cls: &Class, string: &str) -> *mut Object {
    let bytes: *const c_void = string.as_ptr().cast();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        msg_send![
            obj,
            initWithBytes: bytes,
            length: string.len(),
            encoding: UTF8_ENCODING,
        ]
    }
}

impl PartialOrd for NSString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NSString {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.compare(other).into()
    }
}

// TODO: PartialEq and PartialOrd against &str
// See `fruity`'s implementation:
// https://github.com/nvzqz/fruity/blob/320efcf715c2c5fbd2f3084f671f2be2e03a6f2b/src/foundation/ns_string/mod.rs#L69-L163

impl DefaultId for NSString {
    type Ownership = Shared;

    #[inline]
    fn default_id() -> Id<Self, Self::Ownership> {
        Self::new()
    }
}

impl fmt::Display for NSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        autoreleasepool_leaking(|pool| fmt::Display::fmt(self.as_str(pool), f))
    }
}

impl fmt::Debug for NSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        autoreleasepool_leaking(|pool| fmt::Debug::fmt(self.as_str(pool), f))
    }
}
