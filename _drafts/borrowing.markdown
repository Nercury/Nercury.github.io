---
layout: post
title:  "Explore the borrowing system in Rust"
date:   2015-05-14
categories: rust guide
---

This is the second part of the Ownership-Borrowing introduction:

- [Part 1][part-1] - Ownership;
- __Part 2__ - Borrowing.

In this second part, we will continue our practical exploration of Rust's
safety mechanisms, this time - the _borrowing system_. As before, we will
start _very_ simple, and will expand the scope gradually to fit the narrative.

# Ownership Recap

We are going to reuse our brave dummy structure, `Bob`. Thanks, `Bob`!

__Unused return values are destroyed__. The value returned from `Bob::new()`
constructor will be destroyed right there, because it is not used.

{% highlight rust %}
fn main() {
    Bob::new("A"); // Bob is destroyed here
}
{% endhighlight %}

__All values bound with `let` are destroyed at the end of the
scope__, unless they are moved. If we bind returned value to `bob`, we
are going to extend its lifetime up to the end of scope.

{% highlight rust %}
fn main() {
    let bob = Bob::new("A");
    println!("I am alive!");
} // Bob is destroyed here.
{% endhighlight %}

__Replaced values are destroyed__. If we replace bob in mutable slot with
another value, the old value will be destroyed right after.

{% highlight rust %}
fn main() {
    let mut bob = Bob::new("A");
    bob = Bob::new("B"); // Bob "A" is destroyed here.
} // Bob "B" is destroyed here.
{% endhighlight %}

__Destroyed at the end of the scope, unless it is moved__. If we send
`bob` value to another function, we pass its ownership to it. That means
out function does not run `Bob` destruction. `Bob` is most likely
destroyed in the function that took its ownership, but _we don't need to
care_.

{% highlight rust %}
fn main() {
    let bob = Bob::new("A");
    black_hole(bob); // Bob is most likely destroyed somewhere inside.
}
{% endhighlight %}

# Passing value around

Consider this scenario: we wish to modify `Bob` in another function, but
still retain its ownership. Can we simply return it? Well, yes:

{% highlight rust %}
fn modify(bob: Bob) -> Bob {
    bob.name = "modified";
    bob // return the value
}

fn main() {
    let bob = Bob::new("A");
    bob = modify(bob);
}
{% endhighlight %}

__Question:__ How does the Rust know that we have returned the same `Bob` after
modification?

__Answer:__ It doesn't. The ownership is evaluated per-function.

The `modify` could as well swap the bob with a new value, it would all work
the same. Rust functions are responsible only for what they promise
in their definitions, and nothing more.

The definition of `modify(bob: Bob) -> Bob` means that it is going to
_consume a value_ and then _return a value_ of the type of `Bob`.

So you can imagine the values moving in and out of function as the function
runs the statements in it, cleaning all the owned values at the end of each
scope.

It works precisely because because function knows that nothing else owns those
values.

# Scopes









But first - a brief interlude.

# No Garbage Collector, But Safe, eh?

In Rust, we are responsible for collecting our garbage. We do it
by writing scopes and shoving values from one location to another.
The compiler does the rest.

As we saw in the previous part, the ownership rules are quite simple,
and cognitive load is low. In fact, we could write many usable programs
using ownership alone.

But then we crave to be clever. Instead of moving or copying data, we
want to have references to it. Then, we want to have mutable references.
Then, we want to get rid of `Box`ed values and keep things in stack.
Then, we want many mutable and immutable references to our own stack as well
as stack closures, and any combination of these. In multiple threads,
of course.

Then we might as well complain that Rust borrow checker is too restrictive.
But, I suggest we put on our good old `C` hats for a minute and read the
previous paragraph again, replacing every mention of "reference" to "pointer".

That's crazy! We would not do these things in C.

In C, any such use of stack pointer would be well thought out. We would follow
some well established pattern to know who is responsible of clearing each piece
of data, and when. We would plan every deviation from this very carefully,
and isolate such cases from one another. That is, we would need to manage
memory _manually_.

__Rust does not save you from manual memory management. It just makes sure
you haven't made a mistake, if it can understand what you were doing.__

Therefore, to work with Rust, you will have to learn patterns that Rust will
understand, and avoid patterns that will give Rust trouble.

So, with our expectations in check, let us resume.

# Playing Catch Up

The Rust Book
[already has quite an elegant explanation for borrowing][book-borrowing],
so we won't go into it much deeper than a brief summary, and then expand
on it more.

> You might ask: if it is in the book, why bother? Well, first of all,
> I promised it [in the fist part][part-1]. Second, when I was learning
> these concepts, I remember reading every possible bit about them, so,
> maybe my write-up might provide you a different perspective on things.
> But this does not come with any certain guarantee, of course.

[part-1]: /rust/guide/2015/01/19/ownership.html
[book-borrowing]: http://doc.rust-lang.org/nightly/book/references-and-borrowing.html
