---
layout: post
title:  "Simple ownership and borrowing guide"
date:   2015-01-11
categories: rust guide
---

This guide assumes that reader knows basic syntax and building blocks
of Rust but still don't quite grasp how the __ownership__ and
__borrowing__ works.

It will start _very_ simple, and then gradually increase
complexity at a slow pace, explaining and discussing every new bit
of detail. This guide will assume a _very_
basic familiarity with `let`, `fn`, `struct`, `trait` and
`impl` constructs.

The goal is to learn how to design a new Rust program
and not hit any walls related to ownership or borrowing.

#### Contents

- After short [Introduction](#prerequisites---what-you-already-know)
- we will learn about [Copy Trait](#copy-trait)s, and then
- about [Immutable](#ownership)
- and [Mutable](#mutable-ownership) ownership rules.

### Prerequisites - What you already know

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
    foo(i);
}

fn foo(i: i64) {
    // something
}
{% endhighlight %}

Well, it will "die" twice. First, at the end of `foo`,
and then at the end of `main`. If you modify it in `foo`,
__it will not affect__ the value in `main`.

The value __gets copied__ at the call of `foo(i)`.

## Copy Trait

When you shape a new type into existence, you can define
many rules that should be followed when using it. One of them
is a `Copy` trait.

The `Copy` trait is used when the data can be copied automatically
by compiler without any modification. Implementing it allows your
data to be freely copied like a built-in integer.

The built-in machine type `i64` (one kind of integer) implements this
trait, like [many others][copy-trait].

[copy-trait]: http://doc.rust-lang.org/std/marker/trait.Copy.html

If we have a struct `Info`, we can make it copy-able by implementing
`Copy` this way:

{% highlight rust %}
struct Info {
    value: i64,
}
impl Copy for Info {} // no member function
{% endhighlight %}

Or, equivalently, add `#[derive(Copy)]` attribute for it:

{% highlight rust %}
#[derive(Copy)]
struct Info {
    value: i64,
}
{% endhighlight %}

### Why did we need to talk about it?

We needed to get the copyable types out-of-the-way because
by their own nature they are not very useful when demonstrating how
ownership works in Rust.

For example, the [official ownership guide][book-ownership], uses
`i64` type in a `Box` to demonstrate ownership, because `Box` is
not copyable.

> We will talk about `Box` much, much later.

Simply put, copyable types work like primitive types in other
languages and are not really different.

## Ownership

The [official guide][book-ownership] explains ownership _well_ as a tool
for managing resources and memory. I won't repeat much of it here.

Instead, we will concentrate on learning ownership __rules__.

Any type that is not copyable is required to follow these
rules. They ensure that at any point, for a single created instance,
there is only one owner that can __change__ this data.

Therefore, if a function is responsible for deleting this data,
it can be sure that there are no other users that will try to
access, change or delete it in future.

But enough of this abstract stuff, let's get into some real examples!

[book-ownership]: http://doc.rust-lang.org/book/ownership.html

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
we will implement the built-in `Show::fmt` trait.

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

Simple! Compiler makes sure that we can not move moved values,
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

Also, a bit later we will look into big __borrowing__ topic - because
often we do __not__ want to _move_ value to another place just to
temporarily read or modify it.

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

Just a small side note: the function arguments can also be `mut`
slots, because they behave the same way as `let`. So function from
previous example can be shortened:

{% highlight rust %}
fn mutate(mut value: Bob) { // use mut directly before arg name
    value.name = String::from_str("mutant");
}
{% endhighlight %}

### Replacing a value in mutable slot

What happens if we try to overwrite value in `mut` slot? Let's see:

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

The old value gets deleted. A new value will be deleted
at the end of scope, unless it is moved or overwritten, again.

### Mutable Ownership rules

So, there is one additional rule, for mutable slots:

- Unused return values are destroyed.
- All values bound with `let` are destroyed at the end of
function, unless they are moved.
- __Replaced values are destroyed__.

Kind of obvious. The point is, in Rust, we are __sure__
nothing else owns or references them - so it is possible to
do that.

All the rest of the language bends over backwards to make
these ownership rules valid and safe at all times.

## Borrowing
