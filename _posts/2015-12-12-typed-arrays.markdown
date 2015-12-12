---
layout: post
title:  "Zero-Runtime-Cost Mixed Array in Rust"
date:   2015-12-12
categories: rust interesting
---

> Well, either that, or just another way to learn associated types.

Type system in Rust is interesting.

Pushing adds a new element to an array. Let's say we define such trait `Push`:

```rust
trait Push<X> {
    type Next;
    fn push(self, other: X) -> Self::Next;
}
```

Let's go over this quickly.

The trait `Push` is generic over the type `X`. The `X` type will be provided at the location
where the trait `Push` is _used_.

The `type Next` is another generic type. However, the difference from `X` is that `Next` will
be provided by trait _implementation_.

An implementation is also required to provide a `fn push` method. We know it is a _member method_ of some
instance from the keyword argument `self`. And because it is bare `self` (not `&self`),
it is going to _consume_ or _take ownership_ of the instance when called.

The `fn push` requires one argument of the type of `X` and returns whatever type is associated over
`Self::Next` as a result.

### Intent

Calling push is going to combine _input_ of `Self` with another type of `X` and return output
type of `Self::Next` as a result.

## Array? Or what?

So, where is our array? Well, this blog post is titled Zero-Cost, so there isn't any.

But we can implement `Push` for _any other type_, and it will _look like_ we are working with array,
while in fact everything is known at compile-time and is kept in stack (or optimized away).

Besides `Push`, we will see how to implement `Pop` and iterate over the thing.

You may have already guessed that the _type_ of this "array" is going to be obnoxious. But, we
are brave, right?

## Implementing Push

We can think of `()` type (also known as _unit type_ or _empty tuple_) as array of no size.
Implementing `Push` for it is quite easy:

```rust
impl<X> Push<X> for () {
    type Next = (X,);

    fn push(self, other: X) -> Self::Next {
        (other,)
    }
}
```

Here, we provide the concrete types for everything, except for `X`.

The `X` we are going to push will remain unknown. In other words, implementation remains generic for
any `X`, and that's what `impl<X>` is saying.

We fix the target type to _empty tuple_ with `impl .. for ()`.

We specify that the `Next` type is going to be another tuple of single element `X`. We need to use
funny comma in `(X,)`, to disambiguate it from type in parenthesis `(X)`, which otherwise
would be resolved to the plain `X`.

In `push` implementation we put `other` value into a single-element tuple `(other,)` and return it.

### Tuple of Two

Similarly, we can implement `Push` for a tuple of one element. Predictably, the result is going to
be a tuple of two elements:

```rust
impl<T1, X> Push<X> for (T1,) {
    type Next = (T1, X);

    fn push(self, other: X) -> Self::Next {
        (self.0, other)
    }
}
```

Here the `self.0` is a syntax for taking first argument (which is zero) from tuple.

### How does it look?

We can now write this little code (the `Push` trait has to be in scope):

```rust
fn main() {
    let items = ()
        .push(5)
        .push(true);

    println!("{:#?}", items);
}
```

The `{:#?}` is funny little placeholder for pretty-printing the value for debugging.

We get this output:

```text
$ cargo run
   Compiling types v0.1.0 (file:///Users/nercury/fun/types)
     Running `target/debug/types`
(
    5,
    true
)
```

That's a tuple allright. Let's call `push` _one more time_:

```rust
    let items = ()
        .push(5)
        .push(true)
        .push(2);

    println!("{:#?}", items);
```

Output:

```text
26:17 error: no method named `push` found for type `(_, bool)` in the current
scope
    .push(2);
     ^~~~~~~
26:17 help: items from traits can only be used if the trait is implemented and
in scope; the following trait defines an item `push`, perhaps you need to
implement it:
26:17 help: candidate #1: `Push`
```

Rust is telling us that we can keep on going by implementing `Push` for `(T1, T2)`,
then for `(T1, T2, T3)`, and then for `(T1, T2, T3, T4)` and so on.

Indeed, we can do it. We can write a macro that expands into these implementations for
us.

Or we can wait, maybe Rust will have variadic generics one day.

What else can we do?

## Unlimited Size "Array"

Well, we can represent list as "an element" called _Head_ and "all other elements", called _Tail_.
So, when we add new item, it becomes new _Head_, and previous head becomes part of _Tail_. This
is common sequence representation in functional languages.

However, here we will continue extending previously defined implementations, just to show
that we don't need to follow exact rules.

### The chains

First, let's imagine what should happen when we push _one more_ element for `(T1, T2)`:

![Adding T3 to (T1, T2)](/images/typed-arrays/push_1.png)

And then another `T4` to `((T1, T2), (T3,))`:

![Adding T4 to ((T1, T2), (T3,))](/images/typed-arrays/push_2.png)

And then another `T5` to `((T1, T2), (T3, T4))`:

![Adding T4 to ((T1, T2), (T3, T4))](/images/typed-arrays/push_3.png)

Aaaaaaaand... we are done! Because the implementation for these:

```haskell
((T1, T2), (T3,)).push(T4)
(((T1, T2), (T3, T4)), (T5,)).push(T6)
```

Will be generic no matter what _Tail_ there is:

```haskell
(Tail, (T3,)).push(T4)
(Tail, (T5,)).push(T6)
```

### Chain Tree Implementation

However, we can't continue to use tuples for representing the chain. We already used them to mean "a sequence of items", and
here we want to build a tree of them.

But we can define a new type for this chain, so that compiler treats it as a new type and lets us
provide different `Push` for it:

```rust
#[derive(Debug)] // makes this pretty printable
struct Chain<T>(T);
```

Then, the addition of `T3` to `(T1, T2)` will look thus:

```rust
impl<T1, T2, X> Push<X> for (T1, T2) {
    type Next = Chain<((T1, T2), (X,))>;

    fn push(self, other: X) -> Self::Next {
        Chain((
            self,
            (other,),
        ))
    }
}
```

Addition of `T4` to `Chain((T1, T2), (T3,))` will look like this:

```rust
impl<TailT, H1, X> Push<X> for Chain<(TailT, (H1,))> {
    type Next = Chain<(TailT, Chain<(H1, X)>)>;

    fn push(self, other: X) -> Self::Next {
        let Chain(value) = self;
        let head = value.1;
        Chain((
            value.0,
            Chain((head.0, other)),
        ))
    }
}
```

Note how here we no longer care what's in the Tail. We just move it from the previous chain to
the new one.

Also we wrap the second element in the `Chain`, because it is full and we need to pick it up
in next implementation based on this.

And then, finally, let's implement addition of `T5` to `Chain(_, Chain(_))`, where `_` means "whatever":

```rust
impl<C1, C2, X> Push<X> for Chain<(C1, Chain<C2>)> {
    type Next = Chain<(Chain<(C1, Chain<C2>)>, (X,))>;

    fn push(self, other: X) -> Self::Next {
        Chain(
            (
                self,
                (
                    other,
                )
            )
        )
    }
}
```

Here, when we have two formed chains, _everything_ inside `self` becomes a new _Tail_, and
we return a chain of this new _Tail_ and a new _Head_ `X`.

### Using This new Chain

Let's see how adding _one more element_ works now:

```rust
    let items = ()
        .push(5)
        .push(true)
        .push(2);

    println!("{:#?}", items);
```

Output:

```rust
Chain(
    (
        (
            5,
            true
        ),
        (
            2,
        )
    )
)
```

And another:

```rust
    let items = ()
        .push(5)
        .push(true)
        .push(2)
        .push("Hello,");

    println!("{:#?}", items);
```

Output:

```rust
Chain(
    (
        (
            5,
            true
        ),
        Chain(
            (
                2,
                "Hello,"
            )
        )
    )
)
```

Is it unlimited?

```rust
    let items = ()
        .push(5)
        .push(true)
        .push(2)
        .push("Hello,")
        .push("I")
        .push("don't")
        .push("even")
        .push("care")
        .push(2)
        .push("stop");

    println!("{:#?}", items);
```

Indeed it is:

```rust
Chain(
    (
        Chain(
            (
                Chain(
                    (
                        Chain(
                            (
                                (
                                    5,
                                    true
                                ),
                                Chain(
                                    (
                                        2,
                                        "Hello,"
                                    )
                                )
                            )
                        ),
                        Chain(
                            (
                                "I",
                                "don\'t"
                            )
                        )
                    )
                ),
                Chain(
                    (
                        "even",
                        "care"
                    )
                )
            )
        ),
        Chain(
            (
                2,
                "stop"
            )
        )
    )
)
```

Some readers may know or have noticed what _everything_ could be implemented as simple _Head_ - _Tail_
tree. The problem with that, however, is that debugging or printing such type would be even more awkward.
Here, we could make the chain contents bigger (let's say tuple of 12), and resort to `Chain` tree only when contents exceed certain size.

## Implementing Pop

Let's first define a trait for it:

```rust
trait Pop<X> {
    type Next;
    fn pop(self) -> (X, Self::Next);
}
```

Removing element from this "array" requires modification of array type, so the simple `&mut self`
won't work here. So, we return a tuple with the removed element and the remaining _Tail_ of a
new type.

Now, let's implement it for all the types we have. Work is similar to `Push`, but
backwards:

```rust
impl<Tail, H1, X> Pop<X> for Chain<(Tail, Chain<(H1, X)>)> {
    type Next = Chain<(Tail, (H1,))>;

    fn pop(self) -> (X, Self::Next) {
        let Chain(value) = self;
        let Chain(head) = value.1;
        (head.1, Chain((value.0, (head.0,))))
    }
}

impl<Tail, X> Pop<X> for Chain<(Tail, (X,))> {
    type Next = Tail;

    fn pop(self) -> (X, Self::Next) {
        let Chain(value) = self;
        let tail = value.0;
        let head = value.1;
        (head.0, tail)
    }
}

impl<T1, X> Pop<X> for (T1, X) {
    type Next = (T1,);

    fn pop(self) -> (X, Self::Next) {
        (self.1, (self.0,))
    }
}

impl<X> Pop<X> for (X,) {
    type Next = ();

    fn pop(self) -> (X, Self::Next) {
        (self.0, ())
    }
}
```

Let's check it out:

```rust
fn main() {
    let items = ()
        .push(5)
        .push(true)
        .push(2)
        .push("Hello,")
        .push("I")
        .push("don't")
        .push("even")
        .push("care")
        .push(2)
        .push("stop");

    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
    let (value, items) = items.pop(); println!("{:?}", value);
}
```

Output:

```console
"stop"
2
"care"
"even"
"don\'t"
"I"
"Hello,"
2
true
5
```

However, if we call `pop` just _one more time_:

```text
$ cargo run
   Compiling types v0.1.0 (file:///Users/nercury/fun/types)
129:37 error: no method named `pop` found for type `()` in the current scope
let (value, items) = items.pop(); println!("{:?}", value);
                           ^~~~~
```

Which brings home the fact that all of this is just type system trick.

## Passing Around this Scary Type

Let's say, for some reason, somewhere, this is worth the effort.

However, no one is going to pass around such types. Well, at least not without some help from
type system. Question is, can it provide enough help?

For example, if we have a some other methods that add items to our "array", we don't even want to
know the input type. We just want something that can be `Push`ed to,
modify input object, and return resulting output. However, the output type depends directly
on input. It would be great if we could abstract over it somehow, so that we could
provide only added types.

Well, let's do exactly that. First, create a trait that we will abuse to figure out the output
based on input:

```rust
trait AddTo<Input> {
    type Output;

    fn add_to(self, array: Input) -> Self::Output;
}
```

Let's say we have some structure that contains the items that we want to add to our "array":

```rust
#[derive(Copy, Clone)] // tells rust this type is primitive and can be copied
struct Data {
    a: bool,
    b: i32,
}
```

Then we can implement addition like this:

```rust
impl<Input, T2, Final> AddTo<Input> for Data
    where
        Input: Push<bool, Next=T2>,
        T2: Push<i32, Next=Final>,
{
    type Output = Final;

    fn add_to(self, array: Input) -> Self::Output {
        array
            .push(self.a)
            .push(self.b)
    }
}
```

Here, we say that input should be `Push` that has `Next` element equal to `T2`, and `T2` should
be `Push` that has the next element equal to `Final`. Then we say that the `Output = Final`, and
everything clicks into place.

Let's see how it works:

```rust
fn main() {
    let items = ();

    let mut data = Data {
        a: true,
        b: 42,
    };

    // add items from data
    let items = data.add_to(items);

    // add another unrelated item
    let items = items.push("and");

    // modify data and add items from data once more
    data.a = false;
    let items = data.add_to(items);

    println!("{:#?}", items);
}
```

When we print the result, we get the expected sequence:

```text
Chain(
    (
        Chain(
            (
                (
                    true,
                    42
                ),
                Chain(
                    (
                        "and",
                        false
                    )
                )
            )
        ),
        (
            42,
        )
    )
)
```

## Iterating over the Items

Iterators usually require all items to be of the same type. Here, the types are different,
so the only way to iterate is to do some work while iterating, so that end result erases
all the type differences.

For that, we will implement a simple reduce with an abstract visitor.

In other words, for every type we will provide a bit of code (visitor), that does some work
with the value of that type and accumulates the result into the output that is the same type
for all elements.

Here is the trait for visitor:

```rust
trait Visit<A> {
    fn visit(self, acc: A) -> A;
}
```

Let's implement it for all our "array" types. Here, we require that contained types
also implement `Visit`, and forward work to them:

```rust
impl<A, T1> Visit<A> for (T1,)
    where T1: Visit<A>
{
    fn visit(self, acc: A) -> A {
        self.0.visit(acc)
    }
}

impl<A, T1, T2> Visit<A> for (T1, T2)
    where
        T1: Visit<A>,
        T2: Visit<A>
{
    fn visit(self, acc: A) -> A {
        let acc = self.0.visit(acc);
        self.1.visit(acc)
    }
}

impl<A, T1, T2> Visit<A> for Chain<(T1, T2)>
    where
        T1: Visit<A>,
        T2: Visit<A>
{
    fn visit(self, acc: A) -> A {
        let Chain(value) = self;
        let acc = value.0.visit(acc);
        value.1.visit(acc)
    }
}
```

### Collecting the Accumulated Result

Suppose we want every value to append a textual representation of itself to a single output string.

Let's define a type for it, which will be our accumulator:

```rust
#[derive(Debug)]
struct Text(String);
```

Then, we need to implement `Visit` for every type that we ever added to our pseudo array:

```rust
impl Visit<Text> for bool {
    fn visit(self, Text (mut s): Text) -> Text {
        match self {
            true => s.push_str("<true>"),
            false => s.push_str("<false>"),
        }
        Text(s)
    }
}

impl Visit<Text> for i32 {
    fn visit(self, Text (mut s): Text) -> Text {
        s.push_str(&format!("<i32 value {}>", self));
        Text(s)
    }
}

impl<'a> Visit<Text> for &'a str {
    fn visit(self, Text (mut s): Text) -> Text {
        s.push_str(&format!("<{}>", self));
        Text(s)
    }
}
```

And call the `visit`:

```rust
println!("{:#?}", items.visit(Text(String::new())));
```

Output:

```text
Text(
    "<true><i32 value 42><and><false><i32 value 42>"
)
```

How cool is that? This actually looks reasonable!

So, what's not reasonable?

### Type Errors

Let's say, in above example we forget to implement `Visit` for `str`. We get this error:

```text
204:55 error: no method named `visit` found for type
`Chain<(Chain<((bool, i32), Chain<(&str, bool)>)>, (i32,))>`
in the current scope
println!("{:#?}", items.visit(Text(String::new())));
                        ^~~~~~~~~~~~~~~~~~~~~~~~~~
```

This was not really helpful. However, it was kind of expected...

## Conclusion

It is a bit doubtful that this kind of generic array would be actually worth it, simply
because the type of it is going to remain horrendous.

However, in some cases this can provide a type-checked mixed-type execution sequences. And
I am guessing that they may be as fast as writing these sequence actions by hand.

Rust is quite interesting _Systems_ Language.
