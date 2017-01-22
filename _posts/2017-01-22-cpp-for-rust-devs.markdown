---
layout: post
title:  "Short intro to C++ for Rust developers: Ownership and Borrowing"
date:   2017-01-22
categories: C++ intro
---

Today, there was a reddit post that asked what one needs to know when
[Going after C++ with Rust basics][reddit-post]. I thought this was an
interesting question to answer in a blog post and revive my blog.

Since I got C++ job after learning Rust, I thought it would be interesting
to write a summary how one would adapt to C++ with some prior Rust experience.

I would assume the reader already knows C++ syntax and features, and
would be interested in how one would fit concepts to C++ from Rust world.

In this post, however, I could not fit everything I wanted to write, so
I will focus on Ownership, Borrowing and Lifetimes.

## Ownership and Moves

The big feature in Rust is Ownership, which means that non-primitive values are
moved by default instead of being copied.

As an example, if we create a `String` in Rust and pass it to another function,
it will be moved into that function and destroyed there.

```rust
fn foo(val: String) {
    // val destroyed here
}

fn main() {
    let val = String::from("Hello");
    foo(val);
    // accessing val here is compile-time error
}
```

Let's look at the same code in C++:

```c++
#include <string>

using std::string;

void foo(string val) {
    // val is destroyed here
}

int main() {
    string val("Hello");
    foo(val);
    // accessing val here is fine, because we passed a copy to function
    // original val is destroyed here
}
```

You may be tempted to reduce copying in C++ too.

The C++ has this notion of `lvalues` versus `rvalues`.

In C++, `lvalues` are copied, while `rvalues` can be moved, if the type
actually implements move operations (and I am glossing over a lot of details
here).

There is a function in C++ `std` library that allows us to transform any
`lvalue` to `rvalue`, called `std::move`.

So, we can modify our previous C++ program to behave similarly to Rust program
and avoid unnecessary copy by wrapping `val` with `std::move`:

```c++
#include <string>

using std::string;

void foo(string val) {
    // val is destroyed here
}

int main() {
    string val("Hello");
    foo(std::move(val));
    // warning: accessing val here is NOT fine!
    // original val is also destroyed here, but contains no value so it's fine
}
```

Note that `std::move` does not actually move anything, it just changes how
the compiler treats the value at this particular place. In this case, move works
because `std::string` implements move operations.

In C++, it is possible to accidentally _use_ moved value. Therefore, the move
operations usually set the original container size to zero.

Therefore, a good practice in C++ is to __avoid__ using move in the case like
this, even if this means unnecessary deep copy of the value, to avoid the
accidental usage of the moved value.

If the copy of value is actually costly and should not be copied, it is worth
wrapping it into `unique_ptr` (like `Box`) or `shared_ptr` (like `Arc`),
which will keep a single instance of the value on the heap. Relying on `move`
in such case is very fragile and incurs a maintenance cost to keep the program
correct.

## Functions and Methods

### Const references

In Rust, you can create a function that immutably borrows a value:

```rust
fn foo(value: &String) {
    println!("value: {}", value);
}
```

The Rust compiler will not allow calling methods or operations on String that
modify contents of that String. In Rust-talk it would not allow to call methods
that mutably borrow a string or need to take ownership of a string.

In C++, you can do the same:

```cpp
#include <string>
#include <iostream>

using std::string;
using std::cout;
using std::endl;

void foo(const string& value) {
    cout << "value: " << value << endl;
}
```

The `const T&` idiom is similar to `&T` in Rust. C++ compiler will
not allow modifying the contents of `const T&` object. In C++-talk, the C++
would not allow to call methods on the string that are non-const.

### Const methods

Let's say we have structure `Person` in Rust, and use it as parameter for function
`print_full_name`:

```rust
struct Person {
    first_name: String,
    last_name: String,
}

fn print_full_name(person: &Person) {
    println!("{} {}", person.first_name, person.last_name);
}
```

This function could be made into a method on Person:

```rust
struct Person {
    first_name: String,
    last_name: String,
}

impl Person {
    pub fn print_full_name(&self) {
        println!("{} {}", self.first_name, self.last_name);
    }
}
```

Note that `print_full_name` can only access `&self` reference immutably.
In C++, this is achieved with `const` modifier on the method:

```c++
#include <string>
#include <iostream>

class Person {
private:
    std::string first_name;
    std::string last_name;
public:
    void print_full_name() const {
        std::cout << first_name << " " << last_name << std::endl;
    }
};
```

In Rust, we would be able to use `print_full_name` method in places where
`Person` can be borrowed immutably.

```rust
fn foo(person: &Person) {
    person.print_full_name();
}
```

In C++, we will be able to use `print_full_name` in places where `Person`
can be `const`.

```cpp
void foo(const Person& person) {
    person.print_full_name();
}
```

## Methods that Mutably Borrow in C++

In Rust, methods that modify the reference must use `&mut` reference. For
example, a method implemented on `Person`:

```rust
struct Person {
    first_name: String,
    last_name: String,
}

impl Person {
    pub fn clear_name(&mut self) {
        self.first_name.clear();
        self.last_name.clear();
    }
}
```

Or a standalone method:

```rust
fn foo(person: &mut Person) {
    person.clear_name(); // "clear_name" mutably re-borrows Person
}
```

In C++, this is simply any method without `const` qualifier:

```c++
#include <string>

class Person {
private:
    std::string first_name;
    std::string last_name;
public:
    void clear_name() {
        first_name.clear();
        last_name.clear();
    }
};
```

And any method that takes non-const reference:

```cpp
void foo(Person& person) {
    person.clear_name();
}
```

## Methods that Take Ownership in C++

As discussed previously, it is possible in C++, but is considered a bad
practice, and you should leave moves up to the compiler.

However, there is a few cases where manual `std::move` might be ok. One of them
is a setter function.

Consider a Rust method that changes the name:

```rust
struct Person {
    name: String,
}

impl Person {
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
```

We can call it in some function `foo` that had the ownership of the name:

```rust
fn foo(person: &mut Person, name: String) {
    person.set_name(name); // requires explicit clone
}
```

In Rust, the `set_name` will take the ownership of name be default. However,
C++ it would copy by default.

Same method in C++:

```c++
#include <string>

class Person {
private:
    std::string name;
public:
    void set_name(std::string name) {
        this->name = std::move(name); // we can safely move
    }
};
```

We can safely move inside the setter, because we have a parameter that is
already a copy. However, we did not avoid the copying at the call site:

```cpp
void foo(Person& person, std::string name) {
    person.set_name(name); // copy
}
```

We can use `std::move` here:

```cpp
void foo(Person& person, std::string name) {
    person.set_name(std::move(name)); // move
}
```

However, the caller of foo must do the same to ensure the move, and this cycle
continues.

One thing to look for when using `std::move` is mutable
references! Let's say we had a mutable reference in function `foo`, and moved
the value:

```cpp
void foo(Person& person, std::string& name) {
    person.set_name(std::move(name)); // move clears the original name
}
```

Now the caller of foo will suddenly find the name gone.

In this particular case, the better practice is to use `const T&` reference
all the way down to the setter. This will create a copy of name inside
the setter, with a minimal overhead.

However, if the `name` was a very big string, i.e. something like file contents,
and it would be necessary to ensure no copies for performance reasons, the
`unique_ptr` or `shared_ptr` would come to the rescue:

```c++
#include <string>
#include <memory>

class Person {
private:
    std::shared_ptr<std::string> personal_page;
public:
    void set_personal_page(const std::shared_ptr<std::string>& personal_page) {
        this->personal_page = personal_page; // note that we copy here
    }
};
```

Note that we leave the copy in, but what we copy now is only a `Arc` pointer
that points to the same memory contents.

## Lifetimes

One idiomatic thing in Rust is exposing value's contents for external mutation.
All iterators in Rust are built on this concept, as well as many standard
library functions.

For example, we may add a method for `Person` that allows someone else to change
the first and the last names:

```rust
#[derive(Debug)]
struct Person {
    first_name: String,
    last_name: String,
}

impl Person {
    pub fn get_first_name_mut(&mut self) -> &mut String {
        &mut self.first_name
    }

    pub fn get_last_name_mut(&mut self) -> &mut String {
        &mut self.last_name
    }
}
```

Then we can have a function that appends "foo" to a string reference:

```rust
fn append_foo(value: &mut String) {
    value.push_str(" foo");
}
```

Then we can write some code that allows some external function to modify
contents of a `String` inside the `Person`:

```rust
fn main() {
    let mut p = Person {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
    };

    append_foo(p.get_first_name_mut());
    append_foo(p.get_last_name_mut());

    println!("{:?}", p);

    // output:
    // Person { first_name: "John foo", last_name: "Smith foo" }
}
```

As you may know, the Rust compiler understands lifetime elision. That means you
usually do not need to annotate any references with lifetimes, but they are still there.

For example, `impl` of `Person` has these lifetime annotations:

```rust
impl Person {
    pub fn get_first_name_mut(&'a mut self) -> &'a mut String {
        &mut self.first_name
    }
}
```

References are basically pointers. The lifetime syntax `&'a mut` communicates to the
compiler that the returned value must point to the same or narrower memory location `'a` as the function
argument.

If we tried to return a reference to the value which is outside of `'a`, the compiler would complain:

```rust
impl Person {
    pub fn get_first_name_mut(&'a mut self) -> &'a mut String {
        &mut String::from("Other") // error: borrowed value does not live long enough
        //   ^^^^^^^^^^^^^^^^^^^^^ temporary value created here
    }
}
```

Therefore, at the call site, the compiler knows that the `Person` is borrowed
for every call to `append_foo` and would not allow us to do anything funky:

```rust
fn main() {
    let mut p = Person {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
    };

    {
        let name: &mut String = p.get_first_name_mut();
        p.first_name = String::from("Crash");
        // error: cannot assign to `p.first_name` because it is borrowed
        append_foo(name);
    }
}
```

The C++, however, has no machinery to understand where the pointers or references
point to, and does not help. However, we can still implement the same in C++.

First, the `Person`:

```c++
class Person {
public:
    std::string first_name;
    std::string last_name;

    Person(std::string first_name, std::string last_name)
    : first_name(std::move(first_name))
    , last_name(std::move(last_name))
    {}

    std::string& get_first_name_mut() {
        return this->first_name;
    }

    std::string& get_last_name_mut() {
        return this->last_name;
    }
};
```

Similar to setters, we used `std::move` trick in constructor to avoid copies.
This is a usual practice in C++.

Then we create `append_foo`, which is nothing surprising:

```c++
void append_foo(std::string& value) {
    value += " foo";
}
```

And finally, the main function:

```c++
int main() {
    Person p("John", "Smith");

    append_foo(p.get_first_name_mut());
    append_foo(p.get_last_name_mut());

    std::cout << "first name: " << p.first_name << std::endl;
    std::cout << "last name: " << p.last_name << std::endl;

    // output:
    // first name: John foo
    // last name: Smith foo
}
```

However, the C++ compiler is not able to track lifetimes and ensure memory
safety.

This is a problem is when you get used to these things being verified by the compiler.
The objects we have just written might become more complex, and it would
become much harder to track runaway modifications
to `Person`:

```c++
int main() {
    Person p("John", "Smith");

    std::string& name = p.get_first_name_mut();
    p = Person("Crash", "Bob");
    append_foo(name);

    // Output:
    // first name: Crash foo
    // last name: Bob
}
```

It worked, even when we have overwritten the memory location of
`Person`. This actually may continue working. Or it may fail in release build.
Or it may fail when other developer wraps `Person` in shared_ptr:

```c++
int main() {
    auto p = std::make_shared<Person>("John", "Smith");

    std::string& name = p->get_first_name_mut();
    p = std::make_shared<Person>("Crash", "Bob");
    append_foo(name);

    std::cout << "first name: " << p->first_name << std::endl;
    std::cout << "last name: " << p->last_name << std::endl;

    // Output:
    // first name: Crash
    // last name: Bob
}
```

Now, we modified freed memory, which worked, but may not work
if something else was written in that previous memory location.

The better practice in C++ is to avoid methods that return mutable references.
Instead, we could access the fields directly (but trade away privacy):

```c++
int main() {
  Person p("John", "Smith");

  append_foo(p.first_name);
  append_foo(p.last_name);
}
```

Or create the additional copy, which is not really a big deal:

```c++
std::string append_foo(const std::string& value) {
    // set capacity and avoid multiple allocations
    std::string ret;
    ret.reserve(value.size() + 4);
    ret += value;
    ret += " foo";
    return ret;
}

int main() {
    Person p("John", "Smith");

    p.first_name = append_foo(p.first_name);
    p.last_name = append_foo(p.last_name);
}
```

## Conclusion

The big hurdle when moving back to C++ from Rust was the missing move-by-default
feature. This required learning other idiomatic patterns in C++ land, and in some
cases admitting that not all the code needs to be both efficient and easy to maintain.

In most cases maintainability wins, and avoiding "premature optimization" is
very much a necessity in C++.

[reddit-post]: https://www.reddit.com/r/rust/comments/5pfytf/going_after_c_with_rust_basics/
