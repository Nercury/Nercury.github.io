---
layout: post
title:  "Multiple Inheritance of Structure Fields in Rust"
date:   2017-09-01 11:00:00 UTC
categories: rust multiple inheritance
---

As of 1.20 version, Rust does not have inheritance, let alone multiple inheritance. There is some [experimentation
with some of its aspects][specialization-rfc], but it is likely that traditional classes and polymorphism
is not a good match for Rust.

[specialization-rfc]: https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md

However, I don't see OOP as all-or-nothing thing: some aspects of it might be useful for solving specific
problems. Here, we are going to experiment with _field_ inheritance. In OOP terms, we will try to 
use fields from base class. Such fields in traditional OOP have this nice property of being available in a child
class as long as they are *somewhere* in the parent class hierarchy. Likewise, the base class can
observe the changes made by the child class.

For a sneak-peek example, let's say there is "base" `Explosion` "class", which is "inherited" by `BigExplosion`
and the `Shockwave`. This can be written as:

```rust 
struct Explosion
{
    location: (f64, f64),
}

impl Explosion {
    pub fn new(x: f32, y: f32) -> Explosion {
        Explosion { location: (x, y) }
    }
}

#[derive(Inherit)]
#[inherit(Explosion)]
struct BigExplosion;

impl New for BigExplosion {
    
}

#[derive(Inherit)]
#[inherit(Explosion)]
struct Shockwave;
```

To create `Shockwave`, we will need to specify the parent:

```rust
let sw = Shockwave::construct(
    Explosion::new(12.3, 13.4)
);
```

However, we should be able to swap parent with `BigExplosion`, because it inherits `Explosion` required
by the `Shockwave`:

```rust
let sw = Shockwave::construct(
    BigExplosion::construct(
        Explosion::new(12.3, 13.4)
    )
);
```

This should continue to work even though `Explosion` is somewhere deeper in the parent hierarchy.

And then we can get references to the "base" `Explosion` "class", modify the location, which will
modify exactly one place in memory:

```rust
let e: &mut Explosion = &mut sw;
e.location = (54.6, 25.2);
```

To make this implementation really "Rusty", we will avoid heap allocations, so that whole class hierarchy 
can be represented in a single blob in memory. This requirement will present some fun little problems that we will try
to solve.

Lets get started!

