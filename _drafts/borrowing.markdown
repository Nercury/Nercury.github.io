---
layout: post
title:  "Explore the borrowing system in Rust"
date:   2015-01-20
categories: rust guide
---

### WARNING! DRAFT AND RAMBLINGS!

When we create a reference, the value can't be moved.

Explain why.

References poison struct stacks. Investigate how deeply.
(Workarounds? Example?)

Reference must not reach out of stack of any "poisoned" object related to it.
(Example?)

Imagine you drive a car. This is a thing you own. Then someone asks
to borrow the wheels from you. You are not moving anywhere until you get
your wheels back! Getting your wheels stolen is a compile-time error.
So it is always safe to lend your wheels in Rust!

We can move values into better positions on stack to make them live longer
than references. (Example? Relations)

References are temporary. From the point of view of value they
reference.

Pitfalls of putting references in structs. Then using them from the
member functions.

It may initially feel like reference is your usual object that you can
pass around. Not really - it must always get back to its originating
owner - otherwise you are screwed. (Screwiness Examples?)

Maybe pattern: move all values in position, do the work (by borrowing
as neccesary), produce new values as artefacts of work, repeat.

Rust makes you think more about the data.
