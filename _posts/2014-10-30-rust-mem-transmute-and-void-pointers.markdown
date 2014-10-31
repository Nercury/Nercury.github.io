---
layout: post
title:  "Rust's mem::transmute and void pointers"
date:   2014-10-30 19:38:03
categories: rust pointers
---

Yesterday I was trying to produce something that resembles a dependency injection container in Rust.
I came to the point where I was sick and tired of managing lifetimes. I needed
to control the lifetime of object myself - a simple pointer!

Preferably, not too pointy, so I won't stab myself. Still very unsafe.
But probably it would be possible to wrap this logic into something
so it is safe to use externally.

The official intro [in writing unsafe code][unsafe-code] was a good eye-opener,
however it warned me just too many times to avoid [mem::transmute][doc-mem-transmute], so
I just had to. The [mem::transmute doc was not exactly helpful][doc-mem-transmute], but with a help of print statement
and other code references I think I figured it out.

Turns out that if you pass a value into mem::transmute, it will eat ownership of it.
If you transmute it into something stupid like _*mut u8_ (read: __void__ pointer), it is now your responsibility
to manage the result.

{% highlight rust %}
let val: i32 = 333;
let ptr: *mut u8 = unsafe { mem::transmute(&val) };
{% endhighlight %}

Well, the above example compiles and runs, but the result is not really visible.
Let's use something more debugable instead of int - a value that can print
itself when it is created and destroyed:

{% highlight rust %}
struct Val {
    value: i32
}
impl Val {
    fn new(value: i32) -> Val {
        println!("create {}", value);
        Val { value: value }
    }
}
impl Drop for Val {
    fn drop(&mut self) {
        println!("destroyed {}", self.value);
    }
}
{% endhighlight %}

And modify the code:

{% highlight rust %}
let val = Val::new(333);
let ptr: *mut u8 = unsafe { mem::transmute(&val) };
{% endhighlight %}

This prints:

{% highlight console %}
create 333
destroyed 333
{% endhighlight %}

Oh, wait... _We expected no destruction..._. Wel, duh, of course, because we transmuted
the reference instead of actual value. If instead we wrap the value in a _Box_...

{% highlight rust %}
let val = box Val::new(333);
let ptr: *mut u8 = unsafe { mem::transmute(val) };
{% endhighlight %}

We get:

{% highlight console %}
create 333
{% endhighlight %}

Excellent, not destroyed! The box got eaten by transmute - nothing owns the _val_ no more.
We have a memory leak! How nice.

Of course, the same is going to work backwards too:

{% highlight rust %}
let val = box Val::new(333);
let ptr: *mut u8 = unsafe { mem::transmute(val) };

let get_back_val: Box<Val> = unsafe { mem::transmute(ptr) };
{% endhighlight %}

Output:

{% highlight console %}
create 333
destroyed 333
{% endhighlight %}

Turns out that when we get back the value from transmute, it is reintroduced
in the current scope, and rust is generating a destructor for it as if
we did the following:

{% highlight rust %}
let val = box Val::new(333);
// to-pointer and from-pointer removed
let get_back_val: Box<Val> = val;
{% endhighlight %}

This is excellent for taking control of the pointer and accidentally shooting yourself in the foot,
because you can convert it back to anything and get varied garbage or crash horribly.

I will try to give an example (maybe a bit contrived) of something very simple that could use this
mechanism.

Let's make an object that might store a value of arbitrary type and call it MaybeValue.
It will have a mighty pointer inside, as well as the [TypeId][doc-type-id] of stored value:

{% highlight rust %}
use std::intrinsics::TypeId;

struct MaybeValue {
    ptr: *mut u8,
    type_id: TypeId,
}
{% endhighlight %}

We can use generics to create this value from any kind of type, box it and store it's pointer as
well as type.

{% highlight rust %}
impl MaybeValue {
    fn new<T: 'static>(value: T) -> MaybeValue {
        MaybeValue {
            ptr: unsafe { mem::transmute(box value) },
            type_id: TypeId::of::<T>(),
        }
    }
}
{% endhighlight %}

And we can also add method to take away the contained value, called _rob_ (to emphasize what it does):

{% highlight rust %}
impl MaybeValue {
    // ...

    fn rob<T: 'static>(&mut self) -> Option<T> {
        match self.ptr.is_null() {
            true => None, // When ptr is null return None
            false => match TypeId::of::<T>() == self.type_id {
                true => { // When types match

                    // Transmute into returned value and set internal pointer to
                    // null, so we avoid owning same value in several places.

                    let result: Box<T> = unsafe { mem::transmute(self.ptr) };
                    self.ptr = std::ptr::null_mut();

                    Some(*result) // Unbox and return Some
                },
                false => None, // When types do not match return None
            },
        }
    }
}
{% endhighlight %}

Run it:

{% highlight rust %}
fn main() {
    let mut maybe = MaybeValue::new(Val::new(333));
    let result = maybe.rob::<Val>();
    match result {
        Some(val) => println!("Has robbed value {}", val.value),
        None => println!("No robing!")
    }
}
{% endhighlight %}

Prints:

{% highlight console %}
create 333
Has robbed value 333
destroyed 333
{% endhighlight %}

Great! Should only rob once, even if we try to get value twice:

{% highlight rust %}
fn main() {
    let mut maybe = MaybeValue::new(Val::new(333));
    for _ in range(0i, 2i) { // Do it twice:
        let result = maybe.rob::<Val>();
        match result {
            Some(val) => println!("Has robbed value {}", val.value),
            None => println!("No robing!")
        }
    }
}
{% endhighlight %}

Prints:

{% highlight console %}
create 333
Has robbed value 333
destroyed 333
No robing!
{% endhighlight %}

So, it is safe now, right? Well, what happens to value if no one robs it?

{% highlight rust %}
fn main() {
    let mut maybe = MaybeValue::new(Val::new(333));
}
{% endhighlight %}

{% highlight console %}
create 333
{% endhighlight %}

Ooops! MaybeValue has no idea how to correctly destroy this pointer. We could
implement Drop, but it won't know anything about the type, because it was only
available in new...

What if instead we used a closure that knows how to drop it and is initialized
inside _new_?

{% highlight rust %}
struct MaybeValue {
    // ...
    destroy: |*mut u8|:'static + Send -> (),
}

impl MaybeValue {
    fn new<T: 'static>(value: T) -> MaybeValue {
        MaybeValue {
            // ...
            destroy: |ptr| {
                unsafe { mem::transmute::<*mut u8, Box<T>>(ptr) };
            }
        }
    }
    // ...
}

impl Drop for MaybeValue {
    fn drop(&mut self) {
        match self.ptr.is_null() {
            true => (),
            false => (self.destroy)(self.ptr),
        }
    }
}
{% endhighlight %}

To destroy it, we simply "untransmute" it back from pointer to real type
so that rust can automatically dispose of it.

And this indeed works out allright:

{% highlight console %}
create 333
destroyed 333
{% endhighlight %}

I like how Rust allows using powerful language constructs to wrap low level implementation,
and keep this implementation safely locked. I truly believe that Rust is going to
be amazing tool for building both high and low level applications.

Full code of this contrived container:

___Disclaimer: I am newbie in rust, use this as an example only and do not trust my code!___

{% gist Nercury/ccc99ec11f49043c14ed %}



[unsafe-code]:          http://doc.rust-lang.org/guide-unsafe.html
[doc-mem-transmute]:    http://doc.rust-lang.org/std/mem/fn.transmute.html
[doc-type-id]:          http://doc.rust-lang.org/std/intrinsics/struct.TypeId.html
