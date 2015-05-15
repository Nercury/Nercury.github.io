---
layout: post
title:  "Explore the ownership system in Rust"
date:   2015-01-19
categories: rust guide
---

> Updated for Rust 1.0.

This two-part guide is for a reader who knows basic syntax and
building blocks of __Rust__ but does not quite grasp how the
__ownership__ and __borrowing__ works.

We will start _very_ simple, and then will gradually increase
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
- [Reference counting](#reference-counting)
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
fn main() {
    let i = 5;
    foo(i); // call function foo
}

fn foo(i: i64) { // another function foo
    // something
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

The [`Copy` trait][copy-trait] makes your type to behave in a very familiar way:
the bits will be copied to another location when assigned, or when
used as a function argument. Exactly like a built-in integer.

[copy-trait]: http://doc.rust-lang.org/std/marker/trait.Copy.html

For example, this simple struct will be copy-able by default:

{% highlight rust %}
#[derive(Copy, Clone)]
struct Info {
    value: i64,
}
{% endhighlight %}

Note that we had to _tell_ the compiler that it is `Copy` - otherwise
it would __always be moved__ to another
location and would follow the _ownership_ rules.

But we are actually interested in ownership, so from now on we will
concentrate on non-`Copy` types!

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
we will implement the built-in `Debug::fmt` trait method:

{% highlight rust %}
use std::fmt;

impl fmt::Debug for Bob {
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

What if we _bind_ the returned value to a _variable_?

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
bound values at the end of their scope__.

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

[Try it yourself!](http://is.gd/fQ1Bzy)

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

    <anon>:33:16: 33:19 error: use of moved value: `bob`
    <anon>:33     black_hole(bob);
                             ^~~
    <anon>:32:16: 32:19 note: `bob` moved here because it has type `Bob`, which is non-copyable
    <anon>:32     black_hole(bob);
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
    bob.name = "mutant".to_string();
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
    bob.name = "mutant".to_string();
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
    value.name = "mutant".to_string();
}
{% endhighlight %}

### Replacing a value in mutable slot

What happens if we try to overwrite a value in some `mut` slot? Let's see:

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");
    println!(""); // skip line to make output nicer

    // First overwrite using name "B", and then "C"
    for &name in &["B", "C"] {
        println!("before overwrite");
        bob = Bob::new(name);
        println!("after overwrite");
        println!(""); // skip line
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

These ownership rules might seem a tad limiting at first, but
only because we are used to a different set of rules. They
do not limit what is actually possible, they simply give us a
different foundation for building higher-level constructions.

Some of these constructions are way harder to make safe in other
languages. Even if they are made safe, they do not necessarily
provide compile-time safety guarantees.

We will now overview some of them, available in the standard library.

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

    new bob "A"
    del bob "A"

The type of value `bob` returned from Box::new is `Box<Bob>`.
This _generic_ type makes the `Bob` lifecycle managed by this `Box<Bob>`
wrapper and deleted when the `Box` is deleted.

`Box` is not copyable, and follows the same ownership rules discussed
previously. When it reached the end of life on the stack, its destructor `drop`
was called, which subsequently called the `drop` on the `Bob`, as well
as cleaned up the memory on the heap.

The triviality of this implementation is a big deal. If we compare this
to the solutions in other languages, they mostly do one of the two things.
They either leave it up to you to clean up the memory (with some horrible
`delete` statement someone will forget or call twice), or rely on
garbage collection to track memory pointers and
clean up memory when those pointers are no longer referenced.

In Rust, ownership tracking has no runtime penalty and is ensured to be
correct at compile-time. This simple memory deallocation over `Box`
builds directly on ownership tracking, is small, safe and quite often
sufficient.

When it is not sufficient, there are other tools that can help with that.

### Reference Counting

Rust has enough low-level tools for reference counting to be implemented as
a library. It can be used in rare cases when the value has several owners,
therefore its end of life can not be determined statically at compile-time.

Rust has a better name for it: _shared ownership_.
The `std::rc` library provides a way to __share__ ownership of the
same value between different `Rc` _handles_. The value remains alive
as long as there is least one handle for it.

For example, we can make a bob instance managed by `Rc` handle this way:

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

[Try it here!](http://is.gd/5PGJCq)

We can change our `black_hole` function to accept `Rc<Bob>` and check if it is
destroyed by it. But instead it would be more convenient to make it
accept __any__ type `T` that implements `Debug` trait (so we can print it).
We are going to make it _generic_:

{% highlight rust %}
fn black_hole<T>(value: T) where T: fmt::Debug {
    println!("imminent shrinkage {:?}", value);
}
{% endhighlight %}

Works the same, and we will not need to change it for every new type change.

Now, back to sending `Rc<Bob>` to the black hole!

{% highlight rust %}
fn main() {
    let bob = Rc::new(Bob::new("A"));
    black_hole(bob.clone()); // clone call
    println!("{:?}", bob);
}
{% endhighlight %}

    new bob "A"
    imminent shrinkage bob "A"
    bob "A"
    del bob "A"

[Try it here!](http://is.gd/swTG3e)

Plot twist: happy ending! Bob survives the black hole!

Great! How does this work?

Once wrapped by `Rc` handle, bob will live as long as there is a live `Rc` __clone__
somewhere. `Rc` handle internally uses `Box` to place new value in heap memory,
together with reference count (RC).

Every time a new handle clone is created (by calling `clone` on `Rc`), the RC
is increased, and when it reaches end of life, decreased. When
RC reaches zero, the object itself is dropped and memory is deallocated.

Note, that `Rc` above is not mutable. If the contents of `Bob` need to be mutated,
it can be additionally wrapped in the `RefCell` type which allows a mutable
borrow of a reference to our single bob instance. In the following example
it will be mutated it in the `mutate` function.

{% highlight rust %}
fn mutate(bob: Rc<RefCell<Bob>>) {
    bob.borrow_mut().name = "mutant".to_string();
}

fn main() {
    let bob = Rc::new(RefCell::new(Bob::new("A")));
    mutate(bob.clone());
    println!("{:?}", bob);
}
{% endhighlight %}

    new bob "A"
    RefCell { value: bob "mutant" }
    del bob "mutant"

[Try it here!](http://is.gd/OBurg1)

The `RefCell` is used to provide what is called the _interior mutability_.
It is just one of the tools in Rust toolbox to solve a specific problem.

So, the point is: different low-level utilities in Rust can be combined
to achieve _precisely_ what is needed with minimal overhead.

For example, `Rc` can only be used in the same thread. But there is a
`Arc` type for _atomic_ RC usable between threads. A
mutable `Rc` might create cycles when multiple objects reference each other.
However, `Rc` can be cloned into a `Weak` reference which does not participate
in reference-counting. More information can be found in the
[official documentation](http://doc.rust-lang.org/std/rc/).

Most importantly, more advanced memory management mechanisms can (and will)
be implemented later, and they can be done as libraries.

### Concurrency

It is interesting to see how Rust changes the way we work with threads.
The default mode here is no data races. It is not because there are some
special safety walls around threads, no. With Rust you can build
your own threading library with similar safety properties, simply
because the ownership model is in itself thread-safe.

Consider what happens when we send two values into a new Rust thread, a
`Bob` (movable) and an integer (copyable):

{% highlight rust %}
use std::thread;

fn main() {
    let bob = Bob::new("A");
    let i : i64 = 12;

    let child = thread::spawn(move || {
        println!("From thread, {:?} and {:?}!", bob, i);
    });

    println!("waiting for thread to end");
    child.join();
}
{% endhighlight %}

    new bob "A"
    waiting for thread to end
    From thread, bob "A" and 12!
    del bob "A"

[Try it here!](http://is.gd/gOba5R)

What is happening there? First, we create two values:
`bob` and `i`. Then we create a new thread with `thread::spawn`
and pass a _closure_ for it to execute. This closure is going to
_capture_ our variables `bob` as `i`.

Capturing means different things for `Bob` and `i`. Because the `Bob` is
non-`Copy`, it will be _moved_ to the new thread. The `i` will be _copied_
there. When the theead is running, we can modify original copy of `i`
(if needed). It does not influence the copy that was passed to the thread.

`Bob`, however, is now owned by this new thread, and can not be modified unless
the thread returns it back somehow. If we wanted, we could
return it to the main thread over `child.join()` (the `join` waits for
the thread to finish).

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");

    let child = thread::spawn(move || {
        mutate(&mut bob);
        bob
    });

    println!("waiting for thread to end");

    if let Ok(bob) = child.join() {
        println!("{:?}", bob);
    }
}

fn mutate(bob: &mut Bob) {
    bob.name = "mutant".to_string();
}
{% endhighlight %}

    new bob "A"
    waiting for thread to end
    bob "mutant"
    del bob "mutant"

[Experiment more here!](http://is.gd/6EG4JL)

One could say that this does not change much the way we used to work
with threads - we know not to share same memory location
between threads without some kind of synchronisation. The difference
here is that Rust can enforce these rules at compile-time.

Of course, more things are possible in Rust, for example,
the `channels` can be used for sending and receiving data
between threads in more efficient ways. More is available in
[official threading documentation](http://doc.rust-lang.org/std/thread/),
[channel documentation](http://doc.rust-lang.org/std/sync/mpsc/), and
[the book](http://doc.rust-lang.org/1.0.0-beta.5/book/concurrency.html).

### What Else?

We got familiar with ownership system in Rust to the point where
we almost seem comfortable to jump in, browse the docs, and create
great and safe programs with it.

But the other side was glossed over completely: the _borrowing system_.

In the second part of this guide, we will learn why the borrowing
is needed and how best to use it.
