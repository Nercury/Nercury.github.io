---
layout: post
title:  "Simple ownership and borrowing guide"
date:   2015-01-11
categories: rust guide
---

This guide assumes that you know basic syntax and building blocks of Rust
but still don't quite grasp how the __ownership__ and __borrowing__ works.

We will gradually increase complexity at a slow pace, explaining
and discussing every new bit of detail. This guide will assume
basic familiarity with `let`, `fn`, `struct`, `trait` and
`impl` constructs.

In the end, you should be able to design a new Rust program
and not hit any walls related to ownership or borrowing.

## Copyable data types

First, we need to remember the basics, and learn about copyable
types.

### What you already know

What happens to `i` at the end of the `main` function?

{% highlight rust %}
fn main() {
    let i = 5;
}
{% endhighlight %}

It simply ceases to be used, right? We will think of this outcome
as "getting destroyed".

> We are going skip talk about stack frames and obvious optimizations
> for a while.

If we pass this `i` to another function function `foo`, how
many times will it "get destroyed"?

{% highlight rust %}
fn main() {
    let i = 5;
    foo(i);
}

fn foo(i: i64) {

}
{% endhighlight %}

Well, it will get "destroyed" twice. First, at the end of `foo`,
and then at the end of `main`. If you modify it in `foo`,
__it will not affect__ the value in `main`.

The value __gets copied__ at the call of `foo`.

### How Rust does it

When you shape a new type into existence, you can define
many rules that should be followed when using it. One of them
is a `Copy` trait.

The `Copy` trait is used when the data can be copied automatically
by compiler without any modification. Implementing it allows your
data to be freely copied like a built-in integer.

The built-in machine type `i64` (one kind of `int`) implements this
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

Simply put, copyable types work like primitive types in other
languages and are not really different.

## Ownership

The [official guide][book-ownership] explains ownership well as a tool
for managing resources and memory. I won't repeat much of it here.

Instead, we will concentrate on learning how to __reason__ about
the owned data in our code.

Any type that is not copyable is required to follow these ownership
rules. They ensure that at any point, for a singe created instance,
there is only one owner that can __change__ this data.

Therefore, if a function is responsible for deleting this data,
it can be sure that there are no other users that will try to
access, change or delete it in future.

[book-ownership]: http://doc.rust-lang.org/book/ownership.html

### Say hello to Bob, our brave dummy structure

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
        println!("new bob {}", name); // announce
        Bob { name: name.to_string() }
    }
}
{% endhighlight %}

When Bob gets destroyed (sorry, Bob!), we will print its name
by implementing `Drop` trait:

{% highlight rust %}
impl Drop for Bob {
    fn drop(&mut self) {
        println!("del bob {}", self.name);
    }
}
{% endhighlight %}

And to print bob's at any time, we will make it printable
by implementing `Show` trait.

{% highlight rust %}
impl fmt::Show for Bob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bob {}", self.name)
    }
}
{% endhighlight %}

When we create Bob in the `main` function, we get a predictable
result:

{% highlight rust %}
fn main() {
    Bob::new("A");
}
{% endhighlight %}

    new bob A
    del bob A
