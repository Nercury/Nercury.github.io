---
layout: post
title:  "Agile Software Design"
date:   2015-03-07
categories: presentation
---

# [Won't publish](http://slides.com/nercury/agiledesign/live)

## Spaghetti code

You know how it goes. It starts simple. Some module __A__ needs to ask module
__B__ to do some work. Module __B__, however, can only do it by calling __C__.
__C__ in turn calls __D__ and then __D__ finds that only __B__ can do what is
needed. It can go on. The point is, I probably forgot to add
some background to this slide.

Spaghetti code? No one loves that.

## We know we should separate concerns

In this presentation, we will discuss how we can rationalize
our feelings about good or bad design decisions and convert them
into some guidelines anyone could follow.

# Product Export Study

Someone called _NeoShop_ wants to receive products in XML format
by accessing site over some url, like _/api/products_.

The action list is quite clear.

- Receive Request
- Fetch from DB
- Transform to output
- Send Response

If we are using some kind of web framework,
a typical structure our application might look like this:

    ProductModule
        ExportController
            getList(Request) -> Response

Our framework has already took care of first concern: receiving a request and
sending the response.

- Receive Request [framework]
- Fetch from DB
- Transform to output
- Send Response [framework]

We now have to simply "fill the gap". When first implementing it,
we can write everything in the controller method.

    ProductModule
        ExportController
            getList(Request) -> Response
                - Fetch from DB
                - Transform to output

Let's say it has happened. A programmer wrote the implementation in an hour
and business has reaped the benefits quickly.

_This solution probably has no tests, but we will discuss the tests later._

## New requirement: Description too long!

Which part of our mechanism needs a change?

- Receive Request [framework]
- Fetch from DB
- Transform to output __[problem is here]__
- Send Response [framework]

Requirement DOC is a good indication that anything related to
it must be in its own function/file/module/etc.


    ProductModule
        ExportController
            getList(Request) -> Response
                - Fetch from DB
                - Convert to output structure <-- NeoShop
                - Convert structure to XML


But this structure is lying! The `getList` method works __only__ in
case the exported products are in _NeoShop_ format!

## Let's put the feeling why "this is bad" into words

### Dependencies

Consider this assertion: if module __A__ has a dependency on __B__,
module __A__ is _less_ stable than __B__.

You might complain that no, __A__ is super stable, it is just that
__B__ is flaky! However, the fact that your code needs __B__ to
work correctly _by definition_ means that you can not be _less_
flaky than __B__!

### Naming

The method `/api/products` is lying. If someone who does not know
better looks at it, he might get an idea to use it for something else!
However, the output of it is designed for the _NeoShop_.

There is a similar problem with `getList` method - nowhere does it
mention the caveat that exported products might not be quite what you
expect.

### Reusability

If we want to reuse ProductModule in other project that does not need
_NeoShop_ export, we would have to go in and remove this export
controller.

## Correct name experiment

I suggest a simple rule: include in your module's name all the names it
depends on. For example, let's say __P__ depends on __N__. In that case,
we should call it not a __P__, but __NP__.

    P -> N

Actual name:

    NP -> N

In similar way, if __N__ depends on __A__ and __B__, we add all the names to
its name too, making it __ABN__.

    N -> A
    N -> B

Actual name:

    ABN -> A
    ABN -> B

And that name should be updated for __P__ too, making it __ABNP__.

So if our module __Product__ depends on __NeoShop__, it should be called
__NeoShopProduct__ module!

But wait, this is crazy, right? No one is going to do that, the names
would become too long!


### Examples - What are successful examples of separated concerns?

- Semantics and Presentation - HTML/CSS.
- Frontend and Backend - File operations (open, read, write).
- Immediate Representation - Languages, SQL, LLVM.
- Data Layers - Internet protocol suite.
