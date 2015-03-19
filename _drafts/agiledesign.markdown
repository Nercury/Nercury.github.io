---
layout: post
title:  "Agile Software Design"
date:   2015-03-07
categories: presentation
---

# [Won't publish](http://slides.com/nercury/agiledesign/live)

# Agile Software Design

# The problem - The need to be agile

- Software changes
- If software does not change, it becomes useless
- Many areas: business situation
- Laws
- Money, taxes
- Operating systems

## Like a building

> Image of a building with green stable foundation, walls in less green,
> roof a bit yellow, door and windows in red.

We build software like a building. We intuitively understand that
we must have foundation, the walls depend on foundation, roof and windows
depend on walls.

## Except software is not a building

- Buildings rarely change. When they do, the change is carefully planned and
executed by great orchestration.
- In buildings, we know we can change the red parts easily, and changing the
green stuff is the most expensive. We know the cost. Cost is defined by
dependencies.
- However, in software we can create a mess. There is no gravity or laws
of physics to put everything in place. Nothing stops us to make foundation
depend on roof.

## Consider the cost

> Same house picture

- Intuitively, we know that when many things depend on something (i.e. the
foundation), then the cost and effort of changing it is high.
- However, we still create _circular_ dependencies, like, making foundation
depend on roof on windows. A building is wrong example for this, but
in software we can make foundation to be another roof for roof and window for
window.

> Crappy picture where foundation is the window and the roof at the same time.

Question: what is the cost of changing that?

There are two options: when changing it, first untangle the mess! Or...:

> Another picture where we decorate the mess with red parts so much that it
becomes green.

Extend the mess! Make more things depend on mess. What does this do?

It makes mess even _harder_ to change! Look, now our roof-foundation
masterpiece has so many dependencies that it is _stable_. If we need to
change it _now_, we must untangle _even bigger_ mess!

## Symptoms 1

- When we turn on car radio the gearbox falls out.
- We can only think of engineers as completely incompetent.

## Symptoms 2

- As the project development continues, the cost of simple change keeps
increasing.
- Tenfold, then again tenfold.
- Engineers say they can't keep up and project needs rewrite.
- But who wrote it? Them!
- We can only think of them as incompetent.

## Why it is not happening in real world?

Why don't real world engineers do similar crazy things as often as
software engineers?

- Software does not have gravity or other laws of physics.
- In real world there is a distance between things like gearbox and radio!

All laws can be broken in many cool and interesting ways.

> Picture of matrix.

## But what can we do?

Discipline.

- We need to follow some kind of rules.
- But it is boring to follow the rules!
- So they must be really, really good to be worth following!

It is possible to build our own useful laws of software physics!

# Definitions - Let's talk about dependencies

What is a _dependency_ exactly in software?










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

Also, consider what stability means: stable might be good,
but it is _hard to change_. If we want some part of our system
to be easy to modify and change, we should make sure that nothing
depends on it!

### Naming

The method `/api/products` is lying. If someone who does not know
better looks at it, he might get an idea to use it for any product export!
However, the output of it is designed for the _NeoShop_.

When we update this output based on new _NeoShop_ requirements,
we would break anyone else using it. For example, consider the case
where we trimmed the description - but other clients might need the
full description!

### Reusability

If we want to reuse ProductModule in other project that does not need
_NeoShop_ export, we would have to go in and remove this export
controller.

This inhibits the module reusability. Clearly, we must take the
_NeoShop_ stuff and separate it. But how to do it correctly?

## Dependent name experiment

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

## JustBearWithMe

Let's follow the rule and try to modify our project based on it.
We propagate the name prefix to anything that has a dependency on this
_NeoShop_ stuff.

    NeoShopProductModule <-- Explicitly declare this as "NeoShop"
        ExportController
            getList(Request) -> Response
                - Fetch from DB
                - Convert to output structure
                - Convert structure to XML

Ok, our module has received the _NeoShop_ prefix, because it depends on
_NeoShop_ stuff. However, that would likely mean that __anything__ inside
this module is related to _NeoShop_ in some way, which is wrong.

We need to move _NeoShop_ into its own subdir.

For example, a submodule named _NeoShop_!

    ProductModule
        NeoShop <-- Move prefix into "NeoShop" subdir
            ExportController
                getList(Request) -> Response
                    - Fetch from DB
                    - Convert to output structure
                    - Convert structure to XML

The point is, the _ExportController_ now has name _NeoShop_ in its
namespace!

These two cases are not as different as it might seem:

- __NeoShopProduct__ / ExportController
- Product / __NeoShop__ / ExportController

We might even choose to move ProductModule/__NeoShop__ to its own
separate module named __NeoShopProductModule__.

## Submodule naming

There are two core concerns: our __Product__ and __NeoShop__ product.
When we create something that involves both, the namespace
should contain __both__. We can choose where to put it physically
on the disk:

If we choose to keep it inside our own __Product__ module:

- __Product__ / __NeoShop__ / ...

If we choose to move this code into separate module, just for export:

- __NeoShopProduct__ / ...

or:

- __ProductNeoShop__ / ...

We can go further. We can choose to create a separate module that
contains anything that is related to __NeoShop__. However, the
same rule should follow: the namespace should have _Product_ in it,
because now the __NeoShop__ depends on our module!

- __NeoShop__ / __Product__ / ...

In this case the structure of this new module after inversion looks
like this:

    NeoShopModule
        Product <-- We specify that we depend on Product
            ExportController
                getList(Request) -> Response
                    - Fetch from DB      <-- Because we need to fetch it from DB!
                    - Convert to output structure
                    - Convert structure to XML

## Use by Symfony or Zend

Let's consider why we add _Module_ or _Bundle_ suffix to our "modules".

- __ProductBundle__ / ...

or:

- __ProductModule__ / ...

We do that to explicitly communicate that the code in our module
or bundle _will not work without Symfony or Zend_.

However, we are free to do the same trick and move framework-dependant
code into its own namespace, and free the space for framework-independant
component:

- __Product__ / __Bundle__ / ... (Symfony framewok-dependant code)
- __Product__ / __Module__ / ... (Zend framewok-dependant code)
- __Product__ / __Component__ / ... (framework independent lib)

## Layers

It is commonly known adage used for many parts of a system: a class,
object or function should to only __one__ thing.

But how is that possible?

When talking about __one__ thing, we are having in mind some level of
abstraction. Obviously, it gets higher when we go from function, to class,
and then to module.

However, functions or classes should always operate at dedicated abstraction
level and avoid crossing it.

If they do, it is a good sign that function or class needs to be
extracted into its own place, so it continues to do _one thing_.

## Single responsibility principle



### Examples - What are successful examples of separated concerns?

- Semantics and Presentation - HTML/CSS.
- Frontend and Backend - File operations (open, read, write).
- Immediate Representation - Languages, SQL, LLVM.
- Data Layers - Internet protocol suite.
