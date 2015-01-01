---
layout: post
title:  "Dependency injection container - Learning the ropes in Rust"
date:   2014-11-02
categories: rust di
---

- [Part 1][part-1] - Figuring out the design - A naive start;
- __Part 2__ - Learning the ropes in Rust.

## Background

While I dabbled a bit in modern C++ before, it was 2 years ago. Besides that,
the amount of code I wrote in any systems language pales down to the likes
of Javascript or PHP.

[If you look at my initial design of DI container (part 1)][part-1], it looks
like something one would write in Javascript. Indeed, my first mistake was
to approach Rust like it was some kind of [CoffeeScript][coffee-script] clone.

I figured out CoffeeScript in a day, so, I thought, How Hard Can This Be?

## Harder than I expected

In this post I will try to recollect some major walls I ran into. Back then,
I had only a slight idea how the ownership and borrowing works, so, I thought
this little DI project would be great for some practice.

It might be useful for readers who are trying to implement something similar or
want to know how a Rust newbie with scripting-language background thinks.

## Coming up with initial working prototype

From the tutorial and some experimentation before, I imagined traits to
be akin to something like interfaces in C#, Java or PHP. I also found
[`AnyMap`][anymap], so I knew I could cast my boxed value into `Box<Any>`
type and keep items of different types in the same map.

### Sneaky silent reference

I started making basic abstraction: the registry was going to contain all the
getters for all defined items of different types:

{% highlight rust %}
struct Registry {
    factories: HashMap<String, Box<Any>>,
}
{% endhighlight %}

I knew I could implement trait for any value. My idea was to make a trait that
converts any compatible value to a getter of appropriate type:

{% highlight rust %}
trait ToFactory<T> {
    fn to_factory<'r>(self) -> ||:'r -> T;
}
{% endhighlight %}

I figured I could try to use `|| -> T` as my getter, why not? I would then add
it to registry like this:

{% highlight rust %}
impl Registry {
    fn one<T: ToFactory<T>>(&mut self, id: &str, value: T) {
        self.factories.insert(
            id.to_string(),
            box value.to_factory() as Box<Any>
        );
    }
}
{% endhighlight %}

Then, for a simple start, I attempted to implement `ToFactory` for an `i32`
value. Returning a fixed value worked:

{% highlight rust %}
impl ToFactory<i32> for i32 {
    fn to_factory<'r>(self) -> ||:'r -> i32 {
        || 5i32 // always return "5" just to try it...
    }
}
{% endhighlight %}

Wow, I thought. It is going to work, I thought. And then I tried to return
cloned self:

{% highlight rust %}
impl ToFactory<i32> for i32 {
    fn to_factory<'r>(self) -> ||:'r -> i32 {
        || self.clone()
    }
}
{% endhighlight %}

Success? Nope:

{% highlight text %}
<anon>:30:12: 30:16 error: captured variable `self` does not outlive the enclosing closure
<anon>:30         || self.clone()
                     ^~~~
<anon>:29:45: 31:6 note: captured variable is valid for the block at 29:44
<anon>:29     fn to_factory<'r>(self) -> ||:'r -> i32 {
<anon>:30         || self.clone()
<anon>:31     }
<anon>:29:45: 31:6 note: closure is valid for the lifetime 'r as defined on the block at 29:44
<anon>:29     fn to_factory<'r>(self) -> ||:'r -> i32 {
<anon>:30         || self.clone()
<anon>:31     }
{% endhighlight %}

[Playpen link for full code](http://is.gd/jQxIHr).

Being new to this, I though that closure would use `self` value,
not a silent `&self` reference! The error message in this case was completely
cryptic to me. I thought: "well, closure would capture `self` into its
environment, so, no matter where I moved the closure, the `self` would
follow". Why does it complain about `self` not outliving scope? Why should it?

It took me quite a while to figure it out. Sadly, the unboxed closures were
very unstable back then (I got bunch of ICEs when attempting the `move ||`
syntax), so, I thought, I can do it without closures.

Full Java Ahead!

And then I had my next lesson.

### Traits are not like interfaces in other languages

Even though it did not work with a closure, I knew it was certainly possible
to manage without it. I just needed an interface... I mean... _trait_ that
returns my value:

{% highlight rust %}
trait Getter<T> {
    fn take(&self) -> T;
}
{% endhighlight %}

And, just for testing, I implemented `Getter<i32>` for `i32`, so I could
return `i32` where `Getter<i32>` interface was required:

{% highlight rust %}
impl Getter<i32> for i32 {
    fn take(&self) -> i32 {
        self.clone()
    }
}
{% endhighlight %}

Then, instead of a closure, `ToFactory` should return this trait:

{% highlight rust %}
trait ToFactory<T> {
    fn to_factory<'r>(self) -> (Getter<T> + 'r);
}
{% endhighlight %}

Was it going to work? Well:

{% highlight text %}
<anon>:18:13: 18:47 error: the trait `core::kinds::Sized` is not implemented for the type `Getter<T>`
<anon>:18             box value.to_factory() as Box<Any>
                      ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
{% endhighlight %}

Why? What is this `Sized`, I thought. After few shameful attempts to
transmute and store trait as an unsafe pointer instead of a `Box<Any>` blew up
into my face, I tried to gently communicate to compiler that my getter
should always be sized.

I changed my factory to return an unknown getter:

{% highlight rust %}
trait ToFactory<G> {
    fn to_factory<'r>(self) -> G;
}
{% endhighlight %}

I was surprised that I needed no `Sized` when I did this:

{% highlight rust %}
impl Registry {
    fn one<
        T, // for type T
        G: Getter<T> + 'static, // and static Getter<T> as G
        V: ToFactory<G>
    >
        (&mut self, id: &str, value: V)
    {
        self.factories.insert(
            id.to_string(),
            box value.to_factory() as Box<Any>
        );
    }
}
{% endhighlight %}

Little did I know that this kind of signature, when used with an `i32` as an
argument, will essentially be equivalent to this:

{% highlight rust %}
impl Registry {
    fn one_i32(&mut self, id: &str, value: i32)
    {
        self.factories.insert(
            id.to_string(),
            box value as Box<Any>
        );
    }
}
{% endhighlight %}

Notice that no actual `Getter<i32>` is getting boxed? And I thought
I have just managed to shove the trait into the box!

So, imagine my surprise when I saw this when I tested if I could get the
value back:

{% highlight rust %}
use std::any::{Any, AnyRefExt};

impl Registry {
    // <...>

    fn get_val<T>(&self, id: &str) -> T
    {
        // Find the value in map.
        let item = self.factories.get(id).unwrap();
        // Try to downcast Box<Any> back to Getter.
        let getter = item.downcast_ref::<Getter<T>>().unwrap();

        getter.take()
    }
}
{% endhighlight %}
{% highlight text %}
<anon>:32:27: 32:54 error: the trait `core::kinds::Sized` is not implemented for the type `Getter<T>`
<anon>:32         let getter = item.downcast_ref::<Getter<T>>().unwrap();
{% endhighlight %}

Here we go... it needs `Sized`, again.

What did I do? I tried to workaround it again.

{% highlight rust %}
impl Registry {
    // <...>

    fn get_val<T, G: Getter<T> + 'static>(&self, id: &str) -> T
    {
        let item = self.factories.get(id).unwrap();
        let getter = item.downcast_ref::<G>().unwrap();
        getter.take()
    }
}
{% endhighlight %}

What? It doesn't work without the underlying type for getter getting exposed?

The lesson which I learned was this: the traits are simply a collection
of methods, like a vtable for virtual class in C++, but separate from the
struct. And it can not be boxed or used as a trait again if the actual struct
that is backing the trait can not be somehow resolved at compile time.

However, while it worked for this quick test, the real use of getter in DI
would have to be abstracted from the actual type behind `Getter`, because
my getter would be constructed from varied underlying types.

So I ended up implementing a wrapper struct for my `Getter<T>` trait that
looked like this:

{% highlight rust %}
pub struct Factory<'a, T> {
    getter: Box<Getter<T> + 'a>,
}
{% endhighlight %}

Then both `Registry` methods were greatly simplified:

{% highlight rust %}
impl Registry {
    fn one<T: ToFactory<T> + 'static>(&mut self, id: &str, value: T) {
        self.factories.insert(
            id.to_string(),
            box value.to_factory() as Box<Any>
        );
    }

    fn get_val<T: 'static>(&self, id: &str) -> T {
        let item = self.factories.get(id).unwrap();
        let factory = item.downcast_ref::<Factory<T>>().unwrap();
        factory.take()
    }
}
{% endhighlight %}

[Playpen example with complete code](http://is.gd/bCLHVU).

I am still wondering about the exact meaning of `'static` here :)

### The rest

I moved the value construction code [into a new `metafactory` crate][metafactory],
and [named actual dependency injection crate `di`][di].

The biggest chunk of code in these crates is not this (now simple) DI
mechanism, but code required for validation and error collection.
I did not hit any major obstacles while implementing it, it was simply tedious.

## Conclusion

Ownership and borrowing system is great. However, there are some cases
where it may not be intuitive, and some understanding is needed about the
things compiler does behind the scenes - like in the first case where I did
not know that the reference of `self` was used.

Also, Rust does not initially look like systems language. Therefore, one may
be tempted to skip thinking about things like stack or memory allocation.
I did that when I tried to use the trait like an interface. I could only
move on when I finally realised what is actually happening behind the scenes,
and had a better mental model of the way Rust compiles generic methods, traits
and structs, and how they are laid down in memory.

That's all for this DI story for now, but I might return back with part 3
if I find this `di` library useful and decide to improve it further.

[part-1]: /rust/di/2014/11/02/building-dependency-injection-container-in-rust.html
[coffee-script]: http://coffeescript.org/
[anymap]: https://github.com/chris-morgan/anymap
[metafactory]: http://nercury.github.io/di-rs/metafactory/index.html
[di]: http://nercury.github.io/di-rs/di/index.html
