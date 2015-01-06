---
layout: post
title:  "Rust and C# crash course"
date:   2015-01-06
categories: rust csharp tutorial
---

# The agenda

We are going to implement a small simulation in both C# and Rust.
In the process, we will get familiar with a good chunk of Rust
language, as well as quickly see that Rust often requires
a different approach to solve similar problems.



Let's start with overly simplistic example. We have a library `farm`
that can model life of bunch of monsters in it.

When an hour passes some monsters are going to hurt other monsters.

An implementation in C# would look like this:

{% highlight csharp %}
class Farm {
    private List<Monster> monsters;

    void add(Monster monster) {
        monsters.add(monster);
    }

    void
}
{% endhighlight %}

[reddit-post-about-abstract-class]: http://www.reddit.com/r/rust/comments/29ywdu/what_you_dont_love_about_rust/ciq4m20
