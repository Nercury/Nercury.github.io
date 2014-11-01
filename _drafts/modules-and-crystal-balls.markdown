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

Ideally, a good module (at least):

   - Should be independently developable;
   - Should be easily replaceable.

Of course, we can compose a system from a hierarchical
component structure, where component __A__ depends on __B__ and __B__ depends
on __C__.

   __A ➙ B ➙ C__.

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

Most languages now have package managers where __A ➙ B ➙ C__
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
a "module" as such in the component chain.

The weighting criteria follow nicely from previous definition:

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

# Therefore, a modular structure should be dictated by your project

Well, maybe the project is so simple that there is no need for
no crazy plugable things. But then, you notice a switch statement
in one area of a project that keeps growing. You never thought that
this area would ever need modularity, but the project has "decided"
otherwise!

## Building modular structure

It is time for exercise. Let's look at some __A ➙ B ➙ C__ example and make all
the parts completely reusable as modules, yet functioning together when
plugged into the system.

Our example project is going to sort incoming emails based on some rules and
generate some reports for them. We can make three modules for that:

   - _Reader_ - fetches emails from the server;
   - _Sorter_ - groups and sorts them;
   - _Reporter_ - displays the report.

> Hey, but what would be the reasons of splitting it __this__ way?

I am going to invent some story. You can skip it if you want.

1) We need a _Reader_ because the way we fetch emails might change in
the future. Right now we have to deal with IMAP, because, well, the
peculiarities of this particular hosting provider, but _we know for a
fact_ that we will need to integrate with ActiveSync on Microsoft
Exchange, because .. features! And only when client pays for that,
of course.

2) We need a _Reporter_ because the damn report count keeps growing
in the spec, and we also keep hearing these talks about sending
reports directly to manager's smartphone... We kind of suspect that
the _Report_ module might spawn a bunch of new modules, and we would
like to deal with them separately.

3) We need a _Sorter_ because the CEO of that company keeps changing
the sorting rules every few hours, and all our attempts at telepathy
are fruitless. We might even implement a GUI for building sorting
rules instead of dealing with it. In any case, this module might
need to be modified separately too.

OK, story time is over. What we have learned? Well, we need 3 modules,
because we have 3 major sources of change. And we want to prevent
any unrelated system breakage when these changes happen.

So, how do we connect them together?

It would be no surprise for initial implementation to look like this:

    Reporter ➙ Reader ➙ Sorter

Or maybe:

    Reporter ➙ Reader
             ➘ Sorter

However, for _Reporter_ to be really easily reusable, the _reporting_ part
should not depend on anything! We should move the actual report
generation into independent component.

Same with _Reader_ and _Sorter_ - refactor the reading and sorting,
so that they are completely self-contained.

What we get:

           ----------------  MainModule -----------------
           ↓                      ↓                     ↓
    Reporter Component     Reader Component      Sorter Component

Whoa! Where did this "MainModule" came from? Let's look at the code:

{% highlight rust %}
fn main() {
   let reporter = Reporter::new();
   reporter.setEmailSource(EmailReader::new());
   reporter.addFilter(EmailSorter::new());
}
{% endhighlight %}

Well, this was disappointing. The components are tied together over these
"EmailSource" and "EmailFilter" interfaces. Or maybe they are just
using Sorter and Reader structures directly? It might work for smaller
version of this system, but we are trying to imagine this as an example
for a bigger project.

For

[agile-manifesto]:  http://agilemanifesto.org/