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
