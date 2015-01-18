---
layout: post
title:  "Explore ownership system in Rust"
date:   2015-01-19
categories: rust guide
---

This two-part guide is for a reader who knows basic syntax and
building blocks of __Rust__ but does not quite grasp how the
__ownership__ and __borrowing__ works.

It will start _very_ simple, and then gradually increase
complexity at a slow pace, exploring and discussing every new bit
of detail. This guide will assume a _very_
basic familiarity with `let`, `fn`, `struct`, `trait` and
`impl` constructs.

Our goal is to learn how to write a new Rust program
and not hit any walls related to ownership or borrowing.

#### In this first, _ownership_ part:

- After short [Introduction](#prerequisites---what-you-already-know)
- we will learn about the [Copy Traits](#copy-trait), and then
- about the [Immutable](#ownership)
- and [Mutable](#mutable-ownership) ownership rules.
- Then we will look at the [Power of Ownership system](#the-power-of-ownership-system)
- in [Memory management](#memory-allocation),
- [Garbage Collection](#garbage-collection)
- and [Concurrency](#concurrency).

### Prerequisites - What you already know

Scope/stack based memory management is quite intuitive, because we are very
familiar with it.

What happens to `i` at the end of the `main` function?

{% highlight rust %}
fn main() {
    let i = 5;
}
{% endhighlight %}

It goes out of scope and dies, right?

If we pass this `i` to another function `foo`, how
many times will it "die"?

{% highlight rust %}
fn foo(i: i64) { // another function foo
    // something
}

fn main() {
    let i = 5;
    foo(i); // call function foo
}
{% endhighlight %}

Well, it will "die" twice. First, at the end of `foo`,
and then at the end of `main`. If you modify it in `foo`,
__it will not affect__ the value in `main`.

The value __gets copied__ at the call of `foo(i)`.

In Rust, like in C++ (and some other languages), it is possible to use
your own type instead of integer. The value will be allocated on current stack
and it will be destroyed (the destructor will be called) when it goes
out of scope.

However, the Rust compiler follows different _ownership_ rules, unless
type implements a `Copy` trait. Therefore we need to talk about the `Copy`
trait first, and get it out of the way.

## Copy Trait

The `Copy` trait makes the type to behave in a
very familiar way: its bits will be copied to another
location for every assignment or use as function argument.
Implementing it allows your data to be used like a built-in integer.

The built-in machine type `i64` (one kind of integer) implements this
trait, like [many others][copy-trait].

[copy-trait]: http://doc.rust-lang.org/std/marker/trait.Copy.html

If we have a struct `Info`, we can make it copy-able by implementing
`Copy` this way:

{% highlight rust %}
struct Info {
    value: i64,
}
impl Copy for Info {}
{% endhighlight %}

Or, equivalently, add `#[derive(Copy)]` attribute for it:

{% highlight rust %}
#[derive(Copy)]
struct Info {
    value: i64,
}
{% endhighlight %}

The types __without__ this trait will be __always moved__ to another
location and will follow the _ownership_ rules.

## Ownership

Ownership rules ensure, that at any point, for a single non-copyable
value, there is only one owner that can __change__ it.

Therefore, if a function is responsible for deleting this value,
it can be sure that there are no other users that will try to
access, change or delete it in future.

But enough of this abstract stuff, let's get into some real examples!

### Say hello to Bob, our brave new dummy structure

To demonstrate how the data is moving around, we will create
a new struct and call it `Bob`.

{% highlight rust %}
struct Bob {
    name: String,
}
{% endhighlight %}

In Bob constructor `new`, we will announce that it was created:

{% highlight rust %}
impl Bob {
    fn new(name: &str) -> Bob {
        println!("new bob {:?}", name); // announce
        Bob { name: name.to_string() }
    }
}
{% endhighlight %}

When Bob gets destroyed (sorry, Bob!), we will print its name
by implementing built-in `Drop::drop` trait method:

{% highlight rust %}
impl Drop for Bob {
    fn drop(&mut self) {
        println!("del bob {:?}", self.name);
    }
}
{% endhighlight %}

And to make bob value format-able when outputing to console,
we will implement the built-in `Show::fmt` trait method:

{% highlight rust %}
impl fmt::Show for Bob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bob {:?}", self.name)
    }
}
{% endhighlight %}

### Let's put it to the Test!

When we create Bob in the `main` function, we get a predictable
result:

{% highlight rust %}
fn main() {
    Bob::new("A");
}
{% endhighlight %}

    new bob "A"
    del bob "A"

OK, it got deleted somehow - but when exactly?

Let's insert a "print" statement at the end of function:

{% highlight rust %}
fn main() {
    Bob::new("A");
    println!("end is near");
}
{% endhighlight %}

    new bob "A"
    del bob "A"
    end is near

It was deleted __before__ the end of function. The return
value was not assigned to anything, so the compiler called our
`drop` and destroyed the value right there.

What if we bind the returned value to a variable?

{% highlight rust %}
fn main() {
    let bob = Bob::new("A");
    println!("end is near");
}
{% endhighlight %}

    new bob "A"
    end is near
    del bob "A"

With `let`, it was deleted __at the end__ of function - at the
end of variable scope. So the compiler simply __destroys
bound values at the end of scope__.

### Destroyed Unless Moved

There is a catch though - the values can be __moved__
somewhere else - and if they get moved, they won't get destroyed!

How to move them? Well, simply pass them __as values__ to
another function.

Let's pass our bob value to a function named `black_hole`:

{% highlight rust %}
fn black_hole(bob: Bob) {
    println!("imminent shrinkage {:?}", bob);
}

fn main() {
    let bob = Bob::new("A");
    black_hole(bob);
    println!("end is near");
}
{% endhighlight %}

    new bob "A"
    imminent shrinkage bob "A"
    del bob "A"
    end is near

[Try it yourself!](http://is.gd/hOd53m)

It got destroyed in the black hole, and not at the end of `main`!

But wait... What happens if we try to send `Bob` to the black hole
twice?

{% highlight rust %}
fn main() {
    let bob = Bob::new("A");
    black_hole(bob);
    black_hole(bob);
}
{% endhighlight %}

    <anon>:35:16: 35:19 error: use of moved value: `bob`
    <anon>:35     black_hole(bob);
                             ^~~
    <anon>:34:16: 34:19 note: `bob` moved here because it has type `Bob`, which is non-copyable
    <anon>:34     black_hole(bob);
                             ^~~

Simple! Compiler makes sure that we can not use moved values,
and explains nicely what happened.

### There is no Magic - just some rules

To implement "memory safety without garbage collection", compiler
does not need to go chasing your values around the code. It can
decide what is destroyed in a function simply by _looking_
at the function body.

You can easily do that too, if you know the rules. So far, we saw
a __few__ of them:

- __Unused return values are destroyed__.
- __All values bound with `let` are destroyed at the end of
function, unless they are moved__.

Here you go, memory safety based on the fact that there can only be
a single owner of a value.

However, so far we talked only about __immutable__ `let` binding -
the rules get slightly more complicated when the value
can be _changed_.

## Mutable Ownership

All the owned values can be mutated: we just need to put them to
__mut__ slot with __let__. For example, we can mutate some
part of bob, like a `name`:

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");
    bob.name = String::from_str("mutant");
}
{% endhighlight %}

    new bob "A"
    del bob "mutant"

We created it with name "A", but deleted a "mutant".

If we give this value to another function `mutate`, we can also
assign it to `mut` slot there:

{% highlight rust %}
fn mutate(value: Bob) {
    let mut bob = value;
    bob.name = String::from_str("mutant");
}

fn main() {
    mutate(Bob::new("A"));
}
{% endhighlight %}

    new bob "A"
    del bob "mutant"

So, it is possible to make an owned value mutable at any time.

Useful to know: the function arguments can also be upgraded to _mutable_,
because they are also bindable slots that work the same way as a `let` slot.
So function from previous example can be shortened:

{% highlight rust %}
fn mutate(mut value: Bob) { // use mut directly before the arg name
    value.name = String::from_str("mutant");
}
{% endhighlight %}

### Replacing value in a mutable slot

What happens if we try to overwrite value in a `mut` slot? Let's see:

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");
    println!("");
    for &name in ["B", "C"].iter() {
        println!("before overwrite");
        bob = Bob::new(name);
        println!("after overwrite");
        println!("");
    }
}
{% endhighlight %}

    new bob "A"

    before overwrite
    new bob "B"
    del bob "A"
    after overwrite

    before overwrite
    new bob "C"
    del bob "B"
    after overwrite

    del bob "C"

The old value gets deleted. The newly assigned value will be deleted
at the end of scope - unless it is moved or overwritten again.

### Mutable Ownership rules

So, there is one additional rule, for the mutable slots:

- Unused return values are destroyed.
- All values bound with `let` are destroyed at the end of
function, unless they are moved.
- __Replaced values are destroyed__.

Kind of obvious. The point is, in Rust, we are __sure__
nothing else owns or references them - so it is possible to
do that.

## The power of Ownership system

These ownership rules might seem a bit limiting at first, but
only because we are used to the different set of rules. They
do not limit what is actually possible, they simply give us
different foundation for building higher-level constructions.

Some of these constructions are way harder to make safe in other
languages, and even if built, are far from providing compile-time
safety guarantees.

We will now overview some of the tools available in standard library.

### Memory Allocation

So far we talked about integer-like values, that live on a __stack__.
Our test dummy `Bob` was such a value. While some popular languages can _also_
keep values only on a stack (`struct` in C#, or
value instantiation without `new` in C++), many do not.

Instead, a newly constructed object instance (in many languages - with a `new`
operator) is created in what is called the __heap__ memory.

The heap memory has some advantages. First, it is not limited by a stack size.
Placing a huge structure on the a stack might simply overflow it.
Second, its memory location does not change, unlike the location of a stack
value. Every time a stack-allocated value is moved or copied, the actual
bits need to be copied from one place of the stack to another.
While it is very efficient for a small structure
(the values are always "nearby"), it can become slower if the structure
grows bigger.

__Box__ solves this by moving our created value to the __heap__, while
wrapping a small pointer to the heap location on the __stack__.

For example, we can create our `Bob` in the heap memory like this:

{% highlight rust %}
fn main() {
    let bob = Box::new(Bob::new("A"));
}
{% endhighlight %}

    new bob A
    del bob A

The type of value `bob` returned from Box::new is `Box<Bob>`.
This _generic_ type makes the `Bob` lifecycle managed by this `Box<Bob>`
wrapper and deleted when the `Box` is deleted.

`Box` is not copyable, and follows the same ownership rules discussed
previously. When it reached the end of life on the stack, its destructor `drop`
was called, which subsequently called the `drop` on the `Bob`, as well
as cleaned up the memory on the heap.

The triviality of this implementation is a big deal. If we compare this
to the solutions in other languages, they do one of the two things.
They either leave it up to you to clean up the memory (with some horrible
`delete` statement someone will forget or call twice), or rely on
garbage collection to track memory pointers and
clean up memory when those pointers are no longer referenced.

Instead, Rust provides a very simple memory deallocation mechanism
which is safe and quite often sufficient.

When it is not sufficient, there are other tools that can help with that.

### Garbage Collection

Rust has enough low-level tools for garbage collection (GC) to be implemented as
a library. The simplest kind of it already exists in Rust: the
reference-counted GC.

For example, we can make a bob instance managed by `Rc` wrapper this way:

{% highlight rust %}
use std::rc::Rc;

fn main() {
    let bob = Rc::new(Bob::new("A"));
    println!("{:?}", bob);
}
{% endhighlight %}

    new bob A
    Rc(bob A)
    del bob A

[Try it here!](http://is.gd/LFKS2A)

We can change our `black_hole` function to accept `Rc<Bob>` and check if it is
destroyed by it. But instead it would be more convenient to make it
accept __any__ type `T` that implements `Show` trait (so we can print it).
We are going to make it _generic_:

{% highlight rust %}
fn black_hole<T>(value: T) where T: fmt::Show {
    println!("imminent shrinkage {:?}", value);
}
{% endhighlight %}

Works the same, and we will not need to change it for every new type change.

Now, back to sending `Rc<Bob>` to black hole!

{% highlight rust %}
fn main() {
    let bob = Rc::new(Bob::new("A"));
    black_hole(bob.clone()); // clone call
    println!("{:?}", bob);
}
{% endhighlight %}

    new bob A
    imminent shrinkage Rc(bob A)
    Rc(bob A)
    del bob A

[Try it here!](http://is.gd/A5YrX9)

It survived the black hole! Great! How does this work?

Once wrapped by `Rc`, bob will live as long as there is a live `Rc` __clone__
somewhere. `Rc` internally uses `Box` to place new value in heap memory,
together with reference count (RC).

Every time a new clone is created (by calling `clone` on `Rc`), the RC
is increased, and when it reaches end of life, decreased. When
RC reaches zero, the object itself is dropped and memory is deallocated.

Note, that `Rc` above is not mutable. If the contents of `Bob` need to be mutated,
it can be additionally wrapped in the `RefCell` type which allows a mutable
borrow of a reference to our single bob instance. In the following example
it will be mutated it in the `mutate` function.

{% highlight rust %}
fn mutate(bob: Rc<RefCell<Bob>>) {
    bob.borrow_mut().name = String::from_str("mutant");
}

fn main() {
    let bob = Rc::new(RefCell::new(Bob::new("A")));
    mutate(bob.clone());
    println!("{:?}", bob);
}
{% endhighlight %}

    new bob A
    Rc(RefCell { value: bob mutant })
    del bob mutant

This demonstrates how different low-level utilities can be combined to
achieve precisely what is needed with minimal overhead.

For example, `Rc` can only be used in the same thread. But there is a
`Arc` type for _atomic_ RC usable between threads. A mutable `Rc` might
create cycles, in cases where multiple objects reference each other. However,
`Rc` can be cloned into a `Weak` reference which does not participate
in reference-counting. More information can be found in the
[official documentation](http://doc.rust-lang.org/std/rc/).

Most importantly, more advanced garbage collection mechanisms can (and will)
be implemented later, and they can be done as libraries.

### Concurrency

It is interesting to see how Rust changes the way we work with threads.
The default mode here is no data races. It is not because there are some
special safety walls around threads, no. In principle, you could build
your own threading library and with similar safety properties, simply
because the ownership model is in itself thread-safe.

Consider what happens when we send two values into a new Rust thread, a
`Bob` (movable) and an integer (copyable):

{% highlight rust %}
use std::thread::Thread;

fn main() {
    let bob = Bob::new("A");
    let i : i64 = 12;
    let guard = Thread::scoped(move || {
        println!("From thread, {:?} and {:?}!", bob, i);
    });
    println!("waiting for thread to end");
}
{% endhighlight %}

    new bob A
    waiting for thread to end
    From thread, bob A and 12i64!
    del bob A

[Try it here!](http://is.gd/bxhDVZ)

What is happening there? First, we create two values:
`bob` and `i`. Then we create a new thread with `Thread::scoped`
and pass a _closure_ for it to execute. This closure is going to
_capture_ our variables `bob` as `i` simply because it _uses_ them.

_Capturing_ means different thing for `bob` and `i`. The `bob` will get
__moved__ into closure (and will not be usable outside the thread),
while `i` will be __copied__ into it, and will remain usable outside
the thread and in the thread itself.

Our current, main thread will stop and wait for spawned thread
to finish when the returned `guard` will reach the end of its life
(in this case - at the end of our `main` function).

One could say that this does not change much the way we used to work
with threads - we know not to share same memory location
between threads without some kind of synchronisation. The difference
here is that Rust can enforce these rules at compile-time.

Of course, it is possible to get returned result for `guard`,
as well as create `channels` for sending and receiving data
between threads in safe and synchronised way. More is available in
[official threading documentation](http://doc.rust-lang.org/std/thread/),
[channel documentation](http://doc.rust-lang.org/std/sync/mpsc/), and
[the book](http://doc.rust-lang.org/book/threads.html).

### What Else?

We got familiar with ownership system in Rust to the point where
we almost seem comfortable to jump in, browse the docs, and create
great and safe programs with it.

But the other side was glossed over completely: the _borrowing system_.

In the second part of this guide, we will learn why the borrowing
is needed and how best to use it.
