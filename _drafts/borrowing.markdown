---
layout: post
title:  "Explore the borrowing system in Rust"
date:   2015-01-20
categories: rust guide
---

## Borrowing

Borrowing is needed to access or change a memory location
without moving or copying it.

## Reference poisoning

References are harder than ownership because they must bend over backwards to
adapt to ownership rules and avoid breaking them in the process.

Without references, every access to data would need a move.

With references, we can conveniently access objects "remotely".

References themselves are subject to ownership rules, however,
they follow a yo-yo approach. If they are used somewhere and there is no
hook to keep them there they will return back.
