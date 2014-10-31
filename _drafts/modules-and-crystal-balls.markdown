---
layout: post
title:  "Little talk about modules and crystal balls"
date:   2014-10-31
categories: rust module
---

## A module concept

What is a __module__? Google gives me this definition:

> each of a set of __standardized__ parts or __independent__ units that can
> be used to construct a __more complex__ structure, such as an item of furniture or a building.

More often than not, what we may call "module" in our code fails
to achieve this goal by a large margin. Usual approach goes like this:

   - Name possible parts of the system;
   - Put them into separate directories and call them "modules";
   - Watch them grow into unmaintainable coupled mess.

Clearly, those are not modules: we expected so much more when creating them.

Ideally, a good module:

   - Should be independently developable;
   - Should be easily replaceable.

Of course, we can compose a system from a hierarchical
component structure, where component __A__ depends on __B__ and __B__ depends
on __C__.

   __A  →    B    →    C__.

However, can we really develop __B__ independently? The more __C__'s we have,
the more they have chances to screw us by abandoning their project, introducing
backwards-incompatible change or maybe not releasing a security patch for
some older version that we use.

On the other hand, the more __A__'s start to depend on us, the more we are
locked in the current state. __A__ maintainers want us to remain stable,
but release fixes for bugs as long as they use our module.

If we are tightly coupled to __C__ and all our implementation details are
locked in place by __A__, __B__ is no longer easily replaceable. Ease of
development and experimentation is constrained by all the modules that depend
on us, and at the same time our stability is affected by every module we
depend on.

So, what are we talking about when we discuss a good module structure?

_We are talking about_ ___dependency management.___

Most languages now have package managers where __A  →    B    →    C__
dependency chain is easy. But to call a package a module, we would expect
it:

   - To be plugable into a bigger system;
   - To be easily replaceable with another implementation;
   - The implementation could be reused without this "bigger system".

Some frameworks encourage module design that forces your module to
use whole framework as a dependency. This makes module code completely
unusable without a huge dependency tree.

## It is all about having a crystal ball(s)

A module comes with a maintenance cost of a factory glue and additional
dependency management. Clearly, we must weight the cost of introducing
a "module" as such in component chain.

The reasons follow nicely from previous definition:

# 1) Independently developable

Do you actually need to develop it independently? Will you need
to do that in the future? How many people are working or will be working
on the related modules, can you achieve stable-enough glue so
that independent development could actually work?

# 2) Replacable

Will you need to replace it, if ever? Is the future unpredictable
enough for something to be a module with nothing depending on it so that you
can replace it easily? Is future clear enough for something to become
a stable interface for many things to depend upon?

Module is about having some insight into the future. It depends
on the quality of crystal ball, too. And never trust client's
crystal ball. [Instead, create something, put in front of client, and
then use your own crystal ball to predict what the client will do next][agile-manifesto].

# Therefore, a module structure should be dictated by project

Well, maybe the project is so simple that there is no need for
no crazy plugable things. But then, you notice a switch statement
in one area of a project that keeps growing. You never thought that
it might ever would need modularity, but in reality the project "decided"
otherwise!

[nickel-fw]:        http://nickel.rs/
[iron-fw]:          http://ironframework.io/
[agile-manifesto]:  http://agilemanifesto.org/