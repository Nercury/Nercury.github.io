---
layout: post
title:  "Mutable Shared Object and a little Flyweight"
date:   2019-01-01 11:00:00 UTC
categories: rust pattern mutable shared
---

Let's talk about a Rust pattern that puts a flyweight facade on a 
reference-counted mutable shared object.

Sometimes we want to create APIs that do not map well into the idiomatic Rust code.
For example, the design of a channel that transmits data between
the sending and receiving ends requires a hidden state that ties
them together. Or we may want to create many different mesh objects 
for our game, and update mesh positions all around the codebase,
without touching the list used by the loop that renders them sequentially 
and efficiently.

Enter the Mutable Shared Object pattern.

## This blog post was very hard to write

Rust strives for zero-cost abstractions without a compromise.
A use of interior mutability such as `Rc<RefCell<T>>` or `Arc<Mutex<T>>`
is often frowned upon, followed up with questions such as "what are you
trying to do?" and suggestions to learn to do it "the Rust way".

Long story short, the borrow checker that Rust brings along sometimes
makes "the Rust way" a very inconvenient way. It is especially true
if the language user wants to achieve the same design they did in
another language.

I admit that Rust has a point here. It is a different language and
it can't really be learned by mindlessly wrapping every object in a
reference-counted pointer (such as before mentioned `Rc` or `Arc`).

However, "the Rust way" that completely excludes interior mutability
as a possible solution would greatly limit the possibilities.

That's what this blog post is about: a pattern quite often used by
quite advanced Rust users to hide implementation of a quite convenient
API, impossible without interior mutability.

Now that we got the mixed-feelings out of the way, let's look into it. 

## Structure of this blog post

We will call Rust solutions without interior mutability (those that
do not use reference-counted objects) _"pure" Rust_.

First, we we look at how borrow checker together with ownership
limits the solutions we can develop in this "pure" Rust.

Then, we will see how we can actually implement things in "pure"
Rust, and will get familiar with some inconvenient things these
solutions bring.

Lastly, we will look at how to use shared mutability to put a convenience
layer on top of our solution. This is the "pattern" this blog post
is about.



