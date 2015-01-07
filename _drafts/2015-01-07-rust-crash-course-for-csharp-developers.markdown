---
layout: post
title:  "Rust crash course for C# developers"
date:   2015-01-07
categories: rust csharp tutorial
---

## The agenda

We are going to implement a small simulation in both C# and Rust.
In the process, we will get familiar with a good chunk of Rust
language, as well as quickly see that Rust often requires
a different approach to solve similar problems.

## A farm of monsters

It will be a simple console application, consisting
of two parts: one of them a _Farm_ object, which will
have a rectangular grid of _Cells_. Another part will have a bunch
of _Monsters_ with different behaviours. We are going to throw
monsters into the farm and see which one of them survives.

We will start by implementing portion of farm as a class library
(called `crate` in Rust). We will then implement a simple monster
and will output something to console before running into a first
design issue.

### Monster interface

Monsters will live in cells. A cell can have any amount of
monsters in it. A monster can hurt other monster only of they
are in the same cell.

We will add different monsters by implementing a public `Monster`
interface.

{% highlight csharp %}
public interface Monster {
    bool IsAlive { get; }
    void ReceiveDamage(Monster instigator, int damage);
}
{% endhighlight %}

- `IsAllive` will be used by other monsters to check if it is alive.
- `ReceiveDamage` will be used by other monsters to deal damage.
- `instigator` will be a name for "damage dealer", for logging purposes.

In Rust, we can use a trait to implement the same interface:

{% highlight rust %}
pub trait Monster {
    fn is_alive(&self) -> bool;
    fn receive_damage(&mut self, instigator: &Monster, damage: i32);
}
{% endhighlight %}

- Member functions require a reference to the object itself, specified as
`&self`, which is analogous to `this`, but is always required as first parameter.
- If the function will need to modify the object, we use `mut` (mutable)
reference for `&self` - named `&mut self`.
- Rust does not have property getters and setters - we are using simple `is_alive` function.

### Cell

As I mentioned, our `Cell` class will contain monsters. We will also
give a name to it, so that we can differentiate it in logs.

{% highlight csharp %}
using System.Collections.Generic;

public class Cell {
    private readonly string name;
    private List<Monster> monsters = new List<Monster>();
}
{% endhighlight %}

- We will initialize `name` in constructor and it will not change.

Rust does not have "classes" as such. We will use `struct` to do
exactly the same:

{% highlight rust %}
pub struct Cell {
    name: String,
    monsters: Vec<Box<Monster + 'static>>,
}
{% endhighlight %}

- Fields are private by default unless declared as `pub`.
- `Vec` is very similar to `List`.
- Generics work the same: `Vec<T>` will have all methods correctly
typed for the thing it contains.
- A `Monster` trait can not be used directly in Vec as `Vec<Monster>`,
because the trait has a meaning only at compile time. Remember: Rust
has no runtime, it needs to know concrete size at compile-time. To
do that, we use a pointer to allocated memory by wrapping our trait
in a `Box<Monster>`. The size of pointer is known, therefore it
can be used in the `Vec`.
- In C# all class objects are pointers to memory, in Rust
they are in memory only when `Box`'ed.
- We need a `+ 'static` marker to disallow any non-static references
in `Monster` implementation. The alternative is not necessary in this tutorial. _[Read about trait objects here][trait-objects]_.

While these two implementations are almost the same, there is one
key difference in memory management. Rust does not have garbage
collection. However, it will automatically clean-up all `Monster`
implementations when the `Cell` is destroyed. When will the `Cell`
get destroyed? In short - when it goes out of scope. _[Read longer
explanation about ownership here][ownership]_.

Let's add a constructor for `Cell`:

{% highlight csharp %}
public class Cell {
    // ...

    public Cell(string name) {
        this.name = name;
    }
}
{% endhighlight %}

In Rust:

{% highlight rust %}
pub struct Cell {
    // ...
}

impl Cell {
    pub fn new(name: &str) -> Cell {
        Cell {
            name: name.to_string(),
            monsters: Vec::new(),
        }
    }
}
{% endhighlight %}

- An `impl` for `Cell` is needed to declare methods for struct. We add it just bellow the `struct` in file.
- Any `static` method (without `self` as first argument) can be a
constructor. By convention, we use a name `new`. Construction method
returns a new `Cell` for us.
- When constructing a cell, we can specify all non-pub members of it
because the non-pub members are visible in the same module,
like `internal` in C#.

We are going to use a __string slice__ `&str` for constructor argument,
and convert it to a new `String` with `to_string` method.
_[Read more about `String` and `&str` types][strings-and-slices]_.

More often than not, strings in method arguments are `&str` slices, and
strings inside structures like `Cell` are owned `String`s. Like the
`Vec<Monster>`, our `name` in `Cell` will be cleaned up when
parent `Cell` cleans up.

In C#, we can make `Cell` printable by overriding a `ToString` method:

{% highlight csharp %}
public class Cell {
    // ...

    public override string ToString() {
        return name;
    }
}
{% endhighlight %}

And then print it in console with `WriteLine` statement:

{% highlight csharp %}
public class Program
{
    public static void Main()
    {
        var cell = new Cell("Hello");
        Console.WriteLine("Cell is {0}", cell);
    }
}
{% endhighlight %}

In Rust, we can achieve the same by implementing `Show`
trait for `Cell`:

{% highlight rust %}
use std::fmt;

impl fmt::Show for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{0}", self.name);
    }
}
{% endhighlight %}

- The `Show` trait exists in standard library module `fmt`, which
is imported into current scope with `use std::fmt` statement. Anything
in `fmt` can then be used with a `fmt` prefix.
_[Read more about modules][modules]_.
- To implement a trait, we define a new `impl` for it with the trait
methods only. Trait methods can access any other `Cell` methods in
other `impl`s. _[Read more about implementing traits][implementing-traits]_.
- In implementation body, we return formatted result using the
`write!` macro. In addition to formatter, we need to pass
format string, which initially may be very similar to C# format
string and the number of parameters that match parameters in format
string. [Read more about formatting][string-formatting].

And then print it in console with `println!` macro:

{% highlight rust %}
fn main() {
    let cell = Cell::new("Hello");
    println!("Cell is {0}", cell);
}
{% endhighlight %}

- The entry point of rust executable is a function named `main`.
- The `let` keyword initialises a slot for a new variable, the type of which is automatically inferred based on the right side expression,
similarly to `var`.
- We use `println!` macro to format output string.

I wrote several things C#-way to make them more recognisable at
the first glance. We are going to rewrite them in more idiomatic
Rust now.

The last statement in a function is special: if it is missing
a semicolon, it will automatically return the result. So,
return statements are usually written like this:

{% highlight rust %}
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{0}", self.name) // no "return" + no semicolon
}
{% endhighlight %}

Then, the format arguments do not need to contain numbers in them.
If they come in sequence `{0}`, `{1}`, etc., they can be written
simply as `{}` and `{}`. So, we can change our `{0}` in both
`write!` and `println!` to `{}`:

{% highlight rust %}
write!(f, "{}", self.name)
{% endhighlight %}

{% highlight rust %}
println!("Cell is {}", cell);
{% endhighlight %}

- Interesting: in case of argument mismatch, Rust will produce a
compilation error, while C# will fail with a run-time exception.

[reddit-post-about-abstract-class]: http://www.reddit.com/r/rust/comments/29ywdu/what_you_dont_love_about_rust/ciq4m20
[string-formatting]: http://doc.rust-lang.org/std/fmt/
[modules]: /todo
[implementing-traits]: /todo
[strings-and-slices]: /rust/csharp/tutorial/2015/01/06/rust-reference-for-csharp-developers.html#strings-and-string-slices
[trait-objects]: /todo
[ownership]: /todo
