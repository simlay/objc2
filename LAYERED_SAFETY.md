# Layered Safety

Objective-C is different from Rust<sup>[citation needed]</sup>. In particular,
Rust has a concept of "safety" (see the [nomicon] for details), which
Objective-C completely lacks.

You will find when using `icrate` that basically everything (that has not been
manually audited) is `unsafe`. So you might rightfully ask: What's the point
then? Can't I just use `msg_send!`, and save the extra dependency?
Yes, you could, but in fact `icrate` is much safer than doing method calling
manually, even though you may end up writing `unsafe` just as many times. I
dub this "layered safety"<sup>1</sup> to capture the fact that _not all usage
of `unsafe` is created equally_!

Simply put, when using an `unsafe` method in `icrate`, you have to ensure the
compiler of much fewer things than when doing method calling manually.
To see why this is the case, let me guide you through the various abstraction
layers that `icrate` and `objc2` provide, and we'll see how each step makes
things safer!

`icrate` is not perfect, and there may be cases where you have to drop down
into lower-level details; luckily though, the fact that we have this layered
architecture with each step exposed along the way allows you to do exactly
that!

<sup>1: I haven't heard this concept named before, if you know of prior art on this please let me know.</sup>

[citation needed]: https://xkcd.com/285/
[nomicon]: https://doc.rust-lang.org/nomicon/intro.html


## Layer 1: `objc_msgSend`

Unlike C APIs where you define an `extern "C"` function that you want to call,
method calling is done in Objective-C using the "trampoline functions"
[`objc_msgSend`], `objc_msgSend_stret`, `objc_msgSend_fpret` and so on.
Which of these is correct depends on the target architecture and the calling
convention. Furthermore, to use these you first have to cast them to the
correct function signature using `mem::transmute`.

This is actually what's done [in the standard library][std-objc], since they
need to do it so rarely, and the extra dependency on a crate wouldn't be worth
the cost.

[`objc_msgSend`]: https://docs.rs/objc-sys/0.2.0-beta.3/objc_sys/fn.objc_msgSend.html
[std-objc]: https://github.com/rust-lang/rust/blob/aa0189170057a6b56f445f05b9840caf6f260212/library/std/src/sys/unix/args.rs#L196-L248


### Example

Doing the Rust equivalent of Objective-C's `NSUInteger hash_code = [obj hash];`.

```rust
let obj: *const c_void = ...;
let sel = unsafe { sel_registerName(b"hash\0".as_ptr() as *const c_char) };
let fnptr = unsafe {
    mem::transmute::<
        extern "C" fn(*const c_void, SEL) -> NSUInteger,
        extern "C" fn(),
    >(objc_msgSend)
};
let hash_code = unsafe { fnptr(obj, sel) };
```


## Layer 2: `MessageReceiver`

We can improve on this using [`MessageReceiver::send_message`], which
abstracts away the calling convention details, as well as adding an `Encode`
bound on all the involved types. This ensures that we don't accidentally try
to pass e.g. a `Vec<T>`, which does not have a stable memory layout. It also
handles details surrounding Objective-C's `BOOL` type.

Additionally, when `debug_assertions` are enabled, the types involved in the
message send are compared to the types exposed in the Objective-C runtime.
This cannot catch mistakes like passing `null` where a non-null object was
expected, but it helps a lot with accidentally passing a `&c_int` where `int`
was expected.

[`MessageReceiver::send_message`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/trait.MessageReceiver.html#method.send_message


### Example

We'll reuse the `hash` example from above again.

```rust
let obj: &NSObject = ...;
let sel = Sel::register("hash");
let hash_code: NSUInteger = unsafe {
    MessageReceiver::send_message(obj, sel, ())
};
```


## Layer 3a: `msg_send!`

Introducing macros: [`msg_send!`] can abstract away the tediousness of writing
the selector expression, as well as ensuring that the number of arguments to
the method is correct.

[`msg_send!`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/macro.msg_send.html


### Examples

The `hash` example again.

```rust
let obj: &NSObject = ...;
let hash_code: NSUInteger = unsafe { msg_send![obj, hash] };
```

Creating and using an instance of [`NSData`]:

```rust
let obj: *const NSObject = unsafe { msg_send![class!(NSData), new] };
let length: NSUInteger = unsafe { msg_send![obj, length] };
// We have to specify the return type here, see layer 4 below
let _: () = unsafe { msg_send![obj, release] };
```

[`NSData`]: https://developer.apple.com/documentation/foundation/nsdata?language=objc


## Layer 3b: `Id`

As you can see in the new example involving `NSData`, it can be quite tedious
to remember the `release` call when you're done with the object. Furthermore,
whether you need to `retain` and `release` the object involves subtle rules
that depend on the name of the method!

Objective-C solved this years ago with the introduction of "ARC". Similarly,
we can solve this with [`msg_send_id!`] and the smart pointer [`rc::Id`],
which work together to ensure that the memory management of the object is done
correctly.

[`msg_send_id!`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/macro.msg_send_id.html
[`rc::Id`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/rc/struct.Id.html


### Example

The `NSData` example again.

```rust
let obj: Id<NSObject, Shared> = unsafe { msg_send_id![class!(NSData), new] };
let length: NSUInteger = unsafe { msg_send![&obj, length] };
// `obj` goes out of scope, `release` is automatically sent to the object
```


## Layer 4: `extern_x` macros

There's still a problem with the above: we can't actually make a reusable
`hash` nor `length` function, since `NSObject` can refer to any object, and
all objects do not actually respond to that method.

To help with this, we have the [`extern_class!`] macro, which define a new
type resembling `NSObject`, but which represents the `NSData` class instead.

This allows us to make a completely safe API for downstream users!

Along with this, we can now use the [`extern_methods!`] macro to help with
defining our methods, which is also a big improvement over the `msg_send!` /
`msg_send_id!` macros, since it allows us to directly "see" the types, instead
of having them work by type-inference.

[`extern_class!`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/macro.extern_class.html
[`extern_methods!`]: https://docs.rs/objc2/0.3.0-beta.4/objc2/macro.extern_methods.html


### Example

The `NSData` example again.

```rust
extern_class!(
    #[derive(PartialEq, Eq, Hash)]
    pub struct NSData;

    unsafe impl ClassType for NSData {
        type Super = NSObject;
    }
);

extern_methods!(
    unsafe impl NSData {
        #[method_id(new)]
        fn new() -> Id<Self, Shared>;

        #[method(length)]
        fn length(&self) -> NSUInteger;
    }
);

let obj = NSData::new();
let length = obj.length();
```


## Layer 5: `icrate`

Apple has a _lot_ of Objective-C code, and manually defining an interface to
all of it would take a lifetime. Especially keeping track of which methods are
nullable, and which are not, is difficult.

Instead, we can autogenerate the above definition from the headers directly
using type information exposed by `libclang`, giving us a very high confidence
that it is correct!


### Example

The `NSData` example again.

```rust
use icrate::Foundation::NSData;

let obj = NSData::new();
let length = obj.length();
```
