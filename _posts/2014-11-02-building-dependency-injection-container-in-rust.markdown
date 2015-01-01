---
layout: post
title:  "Building dependency injection container in Rust - A naive start"
date:   2014-11-02
categories: rust di
---

- __Part 1__ - A naive start;
- [Part 2][part-2] - Learning the ropes.

### What is dependency injection?

*(Skip this you are familiar with the concept)*

I find that this sentence neatly summarizes the use case for dependency injection (__DI__):

> Making the creation of other objects on which your object depends someone else's problem.

It is used to move the responsibility of object creation away from
object user. For example, consider the creation of this simple object:

{% highlight rust %}
let house = House::new(Bricks::new());
{% endhighlight %}

Suddenly the house user has to care about Bricks. It would be
neat if we could just write:

{% highlight rust %}
let house = House::create();
{% endhighlight %}

And someone else would take care of injecting the right dependency. Now,
should we add that responsibility to House? Probably not - House already
has a very simple "new" method.

Instead, we should have something that can serve as a House _Factory_,
most likely specific to our project that we are making. After all, required dependencies
themselves might depend on many different project-specific parameters that
we should not expose to House or Bricks. Keep them reusable and replaceable!

For example, we might want these dependant objects to be easily replaceable
with fakes when running tests. Or maybe there are different deployment
environments where we have to use different implementations.

Or maybe we simply want it to keep the responsibilities separated.

In the end, we can look at the _Factory_, and the DI container that
contains many factories as the _project-specific_ part of the system.
The more _generic objects_ like House or Bricks
should be _easily reusable_ without this DI mechanism.

### Figuring out the design of DI container

I imagine this DI component to be usable at the project layer, for plugging-in
existing libraries. There would be some kind of container for all object
factory methods:

{% highlight rust %}
let mut container = di::Container::new();
container.define("bricks", |container| {
    Bricks::new()
});
container.define("house", |container| {
    House::new(container.get("bricks"))
});

let house: House = container.get("house");
{% endhighlight %}

Yuck, a string hashmap based solution. I would prefer to avoid
going over hashmap for every object instantiation. But we might
be able to fix this in a moment.

Also, passing the "container", or even simply using the "container" in
factory function looks no more better than using a global. Factory
should receive only those objects it can use.

First, let's get rid of "container", and use "args" array:

{% highlight rust %}
let mut container = di::Container::new();
container.define("bricks", |args| {
    Bricks::new()
});
container.define("house", |args| {
    House::new(args.get(0))
}).with_param<Bricks>("bricks");

let house: House = container.get("house");
{% endhighlight %}

At the first glance this looks a bit uglier, but it is actually
can be much more faster: we can have some magic inside that
plugs `args.get(0)` directly into `"bricks"` closure. Well, _almost_
directly - we need to forward correctly plugged args into `"bricks"`
too.

This change will also allow us to separate this factory mapping
from the runtime container. I will call the mapping part
_Registry_, it is going to contain the definitions
of named factory methods and factory dependencies. The actual _Container_
will be built by inspecting and validating the _Registry_.
It will have all the dependent factories mapped as direct
calls, as well as all types and argument counts
already validated. The result might look like this:

{% highlight rust %}
// First, define the dependencies and factories.
let mut registry = di::Registry::new();
registry.define("bricks", |args| {
    Bricks::new()
});
registry.define("house", |args| {
    House::new(args.get(0))
}).with_param<Bricks>("bricks");

// Create (preferably immutable) container for runtime and forget the registry.
let container = di::Container::new(registry);

// We should be able to get house factory once.
let house_factory = container.get<House>("house");

// Create as many houses as needed.
let house = house_factory.create();
{% endhighlight %}

We only need one hashmap lookup to get `house_factory`, and then
we can use it to create our stream of objects without any lookup.

Also, whatever `house_factory` happens to be internally, it can have
any required parent factory calls locally, so also no additional lookup.

However, that `args.get(0)` irks me a bit... Variadic functions anyone?
Well, no variadic functions exist. Also there seems to be no way to
invoke a closure with a parameter set. Of course, it might be possible
with some memory hack, but I can't figure it out right now.

But.. Maybe we could use real types for factory arguments instead of
"args" and infer them somehow?

For example, if I do this:

{% highlight rust %}
registry.define("house", |bricks: Bricks| {
    House::new(bricks)
}).with(["bricks"]);
{% endhighlight %}

Can I find out the argument type and count and _do the right thing_?
Quick search lends me some nice hits on
[overloading based on argument type][stack-overflow-rust-overloading] and
[unboxed closure trait objects][closure-type]. I _think_ it _might_ be possible.

### Two months later

Well, I wrote the above, went to implement it, and then... 2 months passed.
Read the follow up in [part 2][part-2].

[part-2]: /part-2
[stack-overflow-what-is-dependency-injection]: http://stackoverflow.com/a/130862/1187538
[tuple-crazyness]: http://doc.rust-lang.org/std/tuple/trait.Tuple12.html
[stack-overflow-rust-overloading]: http://stackoverflow.com/questions/24857831/is-there-any-downside-to-overloading-functions-in-rust-using-a-trait-generic-f
[closure-type]: http://stackoverflow.com/questions/24224811/rust-standard-library-closure-parameters-run-time-or-compile-time
