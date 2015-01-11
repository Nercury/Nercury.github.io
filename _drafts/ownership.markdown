---
layout: post
title:  "Simple ownership and borrowing guide"
date:   2015-01-11
categories: rust guide
---

This guide assumes that you know basic syntax and building blocks of Rust
but still don't quite grasp how the __ownership__ and __borrowing__ works.

Here I will suggest another way to imagine data flow, which fits
nicely into the way compiler "thinks".

We will gradually increase complexity at slow pace, explaining
and discussing every new bit of detail. This guide will not try to
be clever.

In the end, you should be able to design a new Rust program
and not hit any walls related to ownership and borrowing when
implementing it.
