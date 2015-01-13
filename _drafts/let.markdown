---
layout: post
title:  "Simple guide to Let keyword"
date:   2015-01-11
categories: rust guide
---

## "Let" Slots

Well, `let` can do much more than an imperative language
assignment.

Think of `let` as a __slot__ for holding a __view__ of some
memory location.

### Mutable Slots

When written as `let x =`, the `x` holds an immutable view
of the exact thing that is on the right side.

When we prefix this slot with a `mut` keyword, we can change
the value here. For example, we can mutate the name of our `Bob`:

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");
    bob.name = String::from_str("mutant");
}
{% endhighlight %}

    new bob "A"
    del bob "mutant"

We created it with name "A", but deleted a "mutant".

What if we wanted to change a name of bob in another `mutate` function?
We can pass the bob to that function, bind it to a mutable
variable `bob`, and then mutate it:

{% highlight rust %}
fn mutate(value: Bob) {
    let mut bob = value;
    bob.name = String::from_str("mutant");
}

fn main() {
    mutate(Bob::new("A"));
}
{% endhighlight %}

    new bob "A"
    del bob "mutant"

But in Rust, function arguments work the same as `let` slots.
We can bind value as `mut` immediately in an argument definition:

{% highlight rust %}
fn mutate(mut value: Bob) {
    value.name = String::from_str("mutant");
}

fn main() {
    mutate(Bob::new("A"));
}
{% endhighlight %}

    new bob "A"
    del bob "mutant"
