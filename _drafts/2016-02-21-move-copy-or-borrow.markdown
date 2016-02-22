---
layout: post
title:  "Move, Copy, or Borrow?"
date:   2016-02-21
categories: rust lessons
---

Real world is intuitive, computers are not. Therefore we build abstractions
that let us model real world using computers.

Then we forget what we've just done, create systems that were not possible
without these abstractions, and also create new designs on top of these
abstractions to work with the data the way the _computers_ do.

When we are disapointed that these abstractions are not good enough.

What I am talking about?

![Adding T3 to (T1, T2)](/images/oop/oop-is.jpg)

No, I am not here to bash [OOP](https://en.wikipedia.org/wiki/Object-oriented_programming),
too late to that party. OOP simply fits well as an example here. It caused both
praise and disapointment, for various reasons.

To the outsiders, however, it might look like we don't know what the hell
we are doing.

So, before of jumping into some other Unicorn Driven Bandwagon, we might just
check our bearings for a moment.

## There is no Perfect Model for the World

Let's make one thing clear: when we write something like:

```java
cat.meow();
```

We do not make cat "meow".

__We run some code named `meow` which has full access to data called `cat`.__

We know this, because there is also this piece of code:

```java
cat.die();
cat.meow();
```

Other devs say that's legit, because `meow` produces different sound if
called when cat is dead.

It's good that's not a real cat, just a very poor representation of it.
