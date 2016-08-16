---
layout: post
title:  "Rust Patterns - Constructor"
date:   2016-06-05
categories: rust guide
---

Rust has demonstrated that it is possible to create a safe, usable language
without hiding pointers and leaving memory cleanup to a garbage collector.

All that is required - a new language that encodes a little bit more of
user's intent into a set of rules, and a completely new standard library
that was designed for the first time ever around these rules.

Not surprisingly, newcomers to Rust face some learning curve.

First roadblock is not a big deal. The rules I am talking about are
ownership and borrowing, with ownership (you can think of it as uniquely
mutable data or resource for now) being the main force that everything else
dances around. They are quite simple, and I will include some links to
official and community tutorials at the end of this post.

The second roadblock is much more complicated. Over the years, we have found
many ways to use features in other languages to achieve what we want. It worked
even when switching from one language to another. In many cases, we
instinctively knew the patterns for implementing our ideas in code.

With Rust, many of those patterns can not be re-used verbatim. Instead,
we need to get familiar with the rules and features in Rust and find
how to do the same things again, in a way that fits the language.

However, there is a lot of Rust code written already, many ideas attempted,
and many patterns have emerged.

That's what this series will be focused on: getting familiar with Rust
patterns.

We will start from... the ways we create things.

## Initializer

Let's say we have a simple data structure:

```rust
struct Point {
    x: f32,
    y: f32,
}
```

In the same Rust module, we can initialize it simply by repeating the same structure, but with actual values:

```rust
let p = Point { x: 2.0, y: 3.0 };
```

### Use cases

We always use this format for private initialization of a structure.

## Public Initializer

To do that from another module, the `x` and `y` fields of point have to be public:

```rust
pub struct Point {
    pub x: f32,
    pub y: f32,
}
```

### Use cases

We use it when all members of a structure can be public and we know
that the fields are not going to change (like in this `Point` case).

Even if that's the case, the constructor might be more convenient.

## The Constructor

Rust language does not have special syntax for constructor. By convention,
we use a static method named "new" as the default way to construct a value.

```rust
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point {
            x: x,
            y: y,
        }
    }
}
```

Then use it as a simple method:

```rust
let p = Point::new(2.0, 3.0);
```

### Use cases



## Ownership and Borrowing tutorials

- [Fearless Concurrency with Rust (2015)][aaron-fearless] (Aaron Turon)
- [Rust ownership, the hard way (2015)][chris-ownership] (Chris Morgan)
- [Rust Book - Ownership (new version, work in progress) (2016)][new-rust-book-ownership] (Steve Klabnik)
- [Rust Book - Ownership (2015)][rust-book-ownership] (official)
- [Explore Ownership System in Rust (2015)][my] (my own guide)


[my]: /rust/guide/2015/01/19/ownership.html
[rust-book-ownership]: https://doc.rust-lang.org/book/ownership.html
[new-rust-book-ownership]: http://rust-lang.github.io/book/ownership.html
[chris-ownership]: https://chrismorgan.info/blog/rust-ownership-the-hard-way.html
[aaron-fearless]: http://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html
