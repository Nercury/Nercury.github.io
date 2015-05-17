---
layout: post
title:  "Explore the borrowing system in Rust"
date:   2015-05-14
categories: rust guide
---

This is the second part of the Ownership-Borrowing introduction:

- [Part 1][part-1] - Ownership;
- __Part 2__ - Borrowing.

In this second part, we will continue our practical exploration of Rust's
safety mechanisms, this time - the _borrowing system_. As before, we will
start _very_ simple, and will expand the scope gradually to fit the narrative.

But first - a brief interlude.

# No Garbage Collector, But Safe, eh?

In Rust, we are responsible for collecting our garbage. We do it
by writing scopes and shoving values from one location to another.
The compiler does the rest.

As we saw in the previous part, the ownership rules are quite simple,
and cognitive load is low. In fact, we could write many usable programs
using ownership alone.

But then we crave to be clever. Instead of moving or copying data, we
want to have references to it. Then, we want to have mutable references.
Then, we want to get rid of `Box`ed values and keep things in stack.
Then, we want many mutable and immutable references to our own stack as well
as stack closures, and any combination of these. In multiple threads,
of course.

Then we might as well complain that Rust borrow checker is too restrictive.
But, I suggest we put on our good old `C` hats for a minute and read the
previous paragraph again, replacing every mention of "reference" to "pointer".

That's crazy! We would not do these things in C.

In C, any such use of stack pointer would be well thought out. We would follow
some well established pattern to know who is responsible of clearing each piece
of data, and when. We would plan every deviation from this very carefully,
and isolate such cases from one another. That is, we would need to manage
memory _manually_.

__Rust does not save you from manual memory management. It just makes sure
you haven't made a mistake, if it can understand what you were doing.__

Therefore, to work with Rust, you will have to learn patterns that Rust will
understand, and avoid patterns that will give Rust trouble.

So, with our expectations in check, let us resume.

# Playing Catch Up

The Rust Book
[already has quite an elegant explanation for borrowing][book-borrowing],
so we won't go into it much deeper than a brief summary, and then expand
on it more.

> You might ask: if it is in the book, why bother? Well, first of all,
> I promised it [in the fist part][part-1]. Second, when I was learning
> these concepts, I remember reading every possible bit about them, so,
> maybe my write-up might provide you a different perspective on things.
> But this does not come with any certain guarantee, of course.

[part-1]: /rust/guide/2015/01/19/ownership.html
[book-borrowing]: http://doc.rust-lang.org/nightly/book/references-and-borrowing.html
