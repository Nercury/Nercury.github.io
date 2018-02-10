---
layout: post
title:  "Rust and OpenGL from scratch - Setup"
date:   2018-02-08
categories: rust opengl tutorial
---

Hello! Let's learn how to work with OpenGL in Rust.

I titled this post "from scratch", because I am going to assume little
knowledge of Rust and basic knowledge of 3D graphics and OpenGL.

Therefore, this tutorial may teach you basic Rust and how to get Rust working
with OpenGL, however for in-depth OpenGL learning you will need another tutorial or book.

"From Scratch" also means that we will try to build abstractions ourselves,
so that we get better knowledge of Rust. In addition to that, we will
able to follow existing OpenGL tutorials, because we will know exactly what
OpenGL functions we are calling.

## Why Rust

> Wrapping carefully written unsafe code in a safe API is the underappreciated cornerstone of Rust

— Jason Orendorff, [Building on an unsafe foundation (video)](https://youtu.be/rTo2u13lVcQ)

Rust allows us to wrap whatever unsafe functionality we
need in a safe layer, and then truly forget about it. C++ can do the same, of course. 
There are three major differences:

- Given a safe abstraction in Rust, we need to explicitly write `unsafe` keyword to wreak havoc.
- Safe abstractions in Rust require less language gymnastics.
- It is easier to write fast safe abstractions.

We won't delve much deeper here in discussion of the last two assertions. Feel
free to examine their correctness yourself while learning Rust.

What does the "unsafe" mean in the context of Rust? [In short - no segfaults](https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html).
In Rust it is unsafe to dereference a raw pointer or call a non-rust function.
OpenGL is non-rust, so we will have lots of unsafe fun.

### Why "unsafe" exists

It may seem strange that "unsafe" exists at all. The reason for it is quite simple:
it allows us to deal with complicated stuff once, inside a function with a safe API,
and then completely forget about it when we become the users of that API. In other
words, it moves the responsibility of correct API usage to API implementer.

## Setup for Rust development

There are many ways to set up development environment for Rust. You can select
the setup that is the most comfortable for you from [this web page](https://forge.rust-lang.org/platform-support.html).
I will explain my setup, which you may choose to follow.

Because Rust is designed to
be cross platform from the ground-up, pure Rust code will compile and run on
 [many platforms](https://forge.rust-lang.org/platform-support.html). However, we
will use packages that link to C libraries, and we historically had more issues with that on Windows.
I will write this tutorial with Windows as a primary platform, and if
you are following on Linux or OSX, I hope use of c-libraries there will require little to no effort.

First, any setup will require [rustup](https://www.rustup.rs/), a Rust toolchain installer,
that takes care of updating Rust and much more. If you have installed Rust using your OS package
manager or Homebrew, I recommend to remove it and reinstall via Rustup.

When completed, the rustup should be available from command prompt:

```txt
> rustup --version
rustup 1.9.0 (57fc3c087 2018-01-04)
```

On windows Rust is available with two toolchains: GNU (compatible with mingw C libraries), and 
MSVC (compatible with Microsoft C++ C libraries). We will use MSVC toolchain.

Install the MSVC toolchain using rustup (may be already installed):

```txt
> rustup install stable-x86_64-pc-windows-msvc
``` 

Make it default:

```txt
> rustup default stable-msvc
```

`rustc` and `cargo` should both work (set up the required environment paths and log in again if they don't):

```txt
> rustc --version
rustc 1.23.0 (766bd11c8 2018-01-01)
```

```txt
> cargo --version
cargo 0.24.0 (45043115c 2017-12-05)
```

MSVC environment will require Microsoft's linker. The easiest way to get it is by installing 
[Visual C++ 2015 Build Tools](http://landinghub.visualstudio.com/visual-cpp-build-tools).

Alternatively, you may install [Visual Studio Community 2017](https://www.visualstudio.com/vs/community/)
with "Desktop development with C++" feature set.
This installs VC++ 2017 toolset that also contains the linker.

We will compile SDL2 from sources. For that, [install cmake](https://cmake.org/download/), 
and make sure to add it to your PATH. Re-login
if necessary.

```txt
> cmake --version
cmake version 3.10.2
```

I use free [IntelliJ IDEA Community Edition](https://www.jetbrains.com/idea/download/), because
I am familiar with IntelliJ products.

[Rust plugin for IntelliJ IDEA](https://intellij-rust.github.io/)
provides good autocompletion, however the debugging story is bad, so we will pay
more attention to good logging.

## Hello world
 
From the command line, create a new Rust project:

```txt
> cargo new --bin new-project
     Created binary (application) `new-project` project
```

You may run it from command line:

```txt
> cd new-project
new-project> cargo run
   Compiling new-project v0.1.0 (file:///C:/Users/Nerijus/dev/new-project)
    Finished dev [unoptimized + debuginfo] target(s) in 1.8 secs
     Running `target\debug\new-project.exe`
Hello, world!
```

Looks like "Hello world" is already written, how boring is that?

Start IntelliJ IDEA and open the same Rust project. It will contain
`src` directory with a `main.rs` file. In the `main.rs` file you will find 
the `main` function:

```rust
fn main() {
    println!("Hello, world!");
}
```

Click near the ▶ arrow near the `fn main`. IntelliJ Rust plugin should also
compile and run your function, as well as add "Run new-project" configuration.

Congrats! We are ready to begin 
[creating a window](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html).