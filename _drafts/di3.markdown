---
layout: post
title:  "The Ownership-driven Dependency Injection"
date:   2015-10-20
categories: rust di
---

- [Part 1][part-1] - Figuring out the design - A naive start;
- [Part 2][part-2] - Learning the ropes in Rust;
- __Part 3__ - Ownership-driven dependencies.

[part-1]: /rust/di/2014/11/02/building-dependency-injection-container-in-rust.html
[part-2]: /rust/di/2015/01/02/dependency-injection-learning-rust.html

> The more you know, the less you need.

Almost a year has passed since [my last attempt at dependency injection library][last-attempt]
for Rust. At that pre-1.0 time, I implemented it using (and a bit abusing)
closure type inference.

In the meantime,
[Rust 1.0 happened][rust-1],
[closures became traits][closures-traits],
[and my hack no longer worked][closure-issue].

[last-attempt]: https://github.com/Nercury/di-rs/tree/c9f0522a4beb845ae54770e9cad312c7145f66b3
[rust-1]: http://blog.rust-lang.org/2015/05/15/Rust-1.0.html
[closures-traits]: https://github.com/rust-lang/rfcs/blob/master/text/0114-closures.md
[closure-issue]: https://github.com/rust-lang/rust/issues/20770

__In this blog post, I will describe how Rust forced me to discover a superior design
I did not know existed.__

But first, let's take a quick look at the most common DI implementation.

## The Container-driven DI

In the center of everything, there is all-knowing and all-powerful container.
When one asks, it gives. When one asks again, and it gives again.

```rust
trait Container {
    fn get(id) -> Something;
}
```

The idea is clear, but the details - not so much. Some aspects may vary
between different implementations:

- The container may or may not be thread safe,
- It may always return new objects, or share singleton instances,
- The `id` can be either a type, or a string identifier,
- The dependency configuration mechanisms are wildly different.

Quite often the container implementations will try to adopt all possible use cases,
and in the process become much more complicated to use.

## The Elephant in the Container

The idea that we need to discuss some kind of _container_ when talking about
_dependency injection_ hints us that there might be some kind of elephant in
the room we are not seeing. Shouldn't the _injection of dependency_ be a
separate thing from _keeping that dependency_?

If you looked at my older blog posts linked at the beginning of this one,
you would see my struggle to shoe-horn container-driven DI framework into
a Rust library. That did not go too well, because I did not realize how
fine-grained and precise Rust's resource management is.

Rust cares a lot about the _correct_ resource management. And when there is a
container that can contain the resources, Rust wants to know all the details
about them, so it can provide the most efficient implementation. But if
the implementation can contain _everything_, Rust forces the design to cater
for _everything_.

And if you are not familiar with Rust by now, "forcing the design" means
"failing to compile".

If one wanted usual-kind of DI, in Rust, something like _this_ would be a result:

```rust
trait Container {
    fn get(id: &str) -> Option<Arc<Mutex<Box<Any>>>>;
}
```

It _is_ possible to hide this partially under type inference [if one (me) wanted hard enough][wanting-hard-enough],
but if we try to keep it simple, Rust exposes all the truth about this container over the
type system.

[wanting-hard-enough]: https://github.com/Nercury/di-rs/tree/c9f0522a4beb845ae54770e9cad312c7145f66b3#example

#### Option - it may be `Some(value)` or `None`

It tells that the thing you requested - you might not get it. In some other languages,
make sure to check if it is not null before calling methods on it.

#### Arc - atomically reference counted

Tells that you are not alone. _Good to know._ Or, in other words, this container design
requires some kind of garbage collection.

#### Mutex - a mutual exclusion primitive

Ah, you want to _mutate_ it? It is obviously a _Singleton Service_. Good disguise for
[global mutable shared state][global-state].

[global-state]: http://programmers.stackexchange.com/questions/148108/why-is-global-state-so-evil

#### Box\<Any\> - a pointer (`Box`) to unknown type (`Any`)

It's like ordering from China. The thing you are going to get might not be the thing you requested.
Make sure to _cast_ it to correct type. _Everyone_ loves casting.

Wow, what a list.

__No wonder so many people hate dependency injection__.

To be fair, dependency injection containers work hard to make sure these things never happen.
They do type-checking and compile-time (or pre-warm) code generation to convert tons of
XML configuration into safe getter list.

However, Rust is a language that successfully does away without garbage collector.
Maybe it is possible to do a DI library without it? Maybe even without reference-counting?

Well, it is obvious that _something_ has to be different here.

## What Problem Are We Solving Here, Again?

Getting things!

No.

I described it [in the first part][part-1]. This is our definition:

> Making the creation of other objects on which your object depends someone else's problem. (1)

The problem here is actually quite simple: the object has to be created on demand, because
someone has requested it. However, this requester _can not_ start owning of the object
it requested, because any other requester might ask for this object later. So this object
will have to sit in memory forever, because anyone getting it would expect the _same instance_.

Assuming we are avoiding shared ownership over reference counting, we need to have a more
specific owner here. My actual "aha" moment happened when I inverted the definition above:

> Making the creation of other objects that depend on your object someone else's problem. (2)

Here! It's inverted. Almost impossible to spot it.

The first definition assumes that all _parents_ are magically injected into _child_ object when
its creation is requested. In other words, it creates everything that is required from
nothing (type id ommited).

```haskell
create = () => Parent + Child
```

We can look at the same definition from other point of view. Assume that your object is
_parent_ and it does not know how its _children_ are created. However, it creates children
_just for the parent scope_, so the lifetime of it is clear.

```haskell
create = (Parent) => Parent + Child
```

In other words, the _parent_ defines the start and end of a lifetime of every _child_ that depends
on it.

However, the obvious questions come to mind:

- This kind of DI, can it be usable?
- What about _depending on multiple parents_?

Well, let's try it out and see!

## The Ownership-based DI

So, we stop doing injection and instead call a list of child constructors when a parent type
is created. The registered constructors are free to create other
types the same way, achieving a process similar to usual DI, but initiated by the root instead
of a leaf. Code:
