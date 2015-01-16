---
layout: post
title:  "Rust reference for C# developers"
date:   2015-01-06
categories: rust csharp tutorial
---

# Functions

Functions can live outside of classes. A function that takes an
integer and returns a boolean looks like this:

{% highlight rust %}
fn is_big(value: i32) -> bool {
    return value > 30;
}
{% endhighlight %}

# Strings and string slices

`String` is a dynamically allocated string that is owned. Its small cousin,
a `&str` is called a __string slice__, and refer to all or some
portion of original `String`.

In rust, when we write a string literal, i.e. `"hello"`, it is a
__string slice__ referring to a static memory location that is
valid for the lifetime of a program.

To get a `String` that can be owned and moved, we explicitly
use `to_string` method. For example:

{% highlight rust %}
let s = "hello".to_string();
// s will be a "String"
{% endhighlight %}

To get a `&str` slice to our new `String` in memory, we use
`as_slice`:

{% highlight rust %}
let s = "hello".to_string().as_slice();
// s will be a "&str"
{% endhighlight %}

# Interface -> trait

# Member functions, `&self`

# Struct is Struct, Class is struct with impl

# Generics

# Collections

# Ownership

# Garbage collection

# References

# Boxing

# Trait objects
