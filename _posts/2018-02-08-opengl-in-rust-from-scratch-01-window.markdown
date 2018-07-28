---
layout: post
title:  "Rust and OpenGL from scratch - Window"
date:   2018-02-08
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html), 
we got some motivation to use Rust and configured our 
development environment.

In this lesson, we will create a window! If that does not sound too exciting, we will also learn
about Rust libraries. Very exciting.

## A window

To create a window that works across multiple platforms, as well as provides such niceties as
OpenGL context or multi-platform input, we will use SDL2.

## Crates

Rust libraries are called `crates`, and the Rust crate manager is `cargo` command-line tool.
Crates can be found by searching central Rust crate repository at [crates.io](https://crates.io).

A new Rust library can be created either manually or using `cargo new library-name` command.
The directory structure for a new library would look quite similar to our first hello-world project:

```haskell
library-name
    src
        lib.rs
    Cargo.toml
```

The difference from executable project, is that instead of `main.rs` there is `lib.rs` in `src` directory.
This is configuration by convention, and it could be changed if needed.

`Cargo.toml` file describes the crate. It contains crate identifier, list of dependencies, links to 
documentation, and [many other things explained in cargo documentation book](https://doc.rust-lang.org/cargo/).

## SDL2 crate

As you may know, [libsdl is, in fact, a C project](https://www.libsdl.org/). 
The `sdl2` crate, however, is a safe Rust wrapper around
SDL2 C API.

Let's go to [crates.io](https://crates.io) and search for `sdl2`.

On [sdl2 crate page](https://crates.io/crates/sdl2), you can find links to
documentation, github repository.

### A note about SDL choice

For better or worse, I have picked SDL for this tutorial. However, you should
be aware that there are Rust alternatives. For example [winit](https://crates.io/crates/winit)
crate has an API that is very similar to Rust-SDL.

So if the SDL2 does not work, or you want to go with more Rusty solution,
try `winit`! The next lessons won't use anything else but window events from
SDL2 anyways.

### Documentation

Library authors can provide link to crate documentation in their `Cargo.toml` file. This is that link.
It contains API reference.

### Github repository

This links to github `rust-sdl2` project. This page may also contain documentation, in the root
`README.md` file rendered by github. In case of `sdl2` crate, it contains information of how to set-up
SDL2.

## Dependencies

Let's create a new project named `game`.

```txt
> cargo new --bin game
```

Add `[dependencies]` section to `Cargo.toml` file, with `sdl2` crate as dependency:

(Cargo.toml, incomplete)

```toml
[dependencies]
sdl2 = "0.31.0"
```

At the time of writing this, the latest `sdl2` crate version was `0.31.0`. You can input the exact same
version to make sure everything compiles.

Specifying a dependency allows the crate to be downloaded.

To use it, we then need to reference it.

## Crate reference

At the top of `main.rs` file, reference `sdl2` by writing

```rust
extern crate sdl2;
```

This "mounts" sdl2 crate at our crate root module. Sdl functions can then be referenced
like `sdl2::init()`.

## Initializing SDL

Replace `println!("Hello, world!");` with the code to initialize sdl:

(full main.rs is provided)

```rust
extern crate sdl2;

fn main() {
    let _sdl = sdl2::init().unwrap();
}
```

We will go over every detail in the code soon. But first, run it!

...

It is unlikely that worked, unless you already had SDL2 installed and was not using Windows!

I got this error:

```txt
error: linking with `link.exe` failed: exit code: 1181
  |
  = note: "C:\\Program Files (x86)\\Microsoft Visual Studio\\2017\\Community\\VC\...
  = note: LINK : fatal error LNK1181: cannot open input file 'SDL2.lib'
          

error: aborting due to previous error

error: Could not compile `lesson-01-window`.
```

This is easily fixed on OSX or Linux by installing libsdl2-dev package (sdl2 on homebrew).
Everything is explained in detail in [rust-sdl2/README.md](https://github.com/Rust-SDL2/rust-sdl2).

For windows, we may learn what build scripts are, create a rust "script" that runs
at compile time, tells `cargo` how to link lib files as well as copies dll files to executable
directory. Details are in the beforementioned README.

Or, we may use quite recent "Bundled" feature, which makes `sdl2` do all this work itself.
From the README:

> Since 0.31, this crate supports a feature named "bundled" which downloads SDL2 from source, 
compiles it and links it automatically. While this should work for any architecture, you will 
need a C compiler (like gcc, clang, or MS's own compiler) to use this feature properly.

## Crate features

Crates have a powerful conditional compilation mechanism available, that allows them to conditionally
include/exclude functionality based on architecture, OS, and you guessed it, features.

This can be even used to include additional platform-dependant dependencies, invoke compilers
or even crazy things like downloading sdl sources and compiling them.

## Linking SDL

In our case, we just want to tell `sdl2` to use this "bundled" feature.

For that, `sdl2` dependency string can be replaced with map `{}` to contain additional information:

(Cargo.toml, incomplete)

```toml
[dependencies]
sdl2 = { version = "0.31.0", features = ["bundled", "static-link"] }
```

We also throw-in "static-link" feature to avoid copying `dll` files around. Without "static-link", we would
need to fish out the dll out of build directory `target/debug/build/sdl2-sys-???/out/bin` and copy
it to `target/debug` location manually (where our main executable is placed after it is compiled).
However, if that's the case, it may be better to fall back to build-script method of setting up sdl2.

There is an alternative way to specify sdl2 dependency:

(Cargo.toml, incomplete)

```toml
[dependencies.sdl2]
version = "0.31.0"
features = ["bundled", "static-link"]
```

This way, we can use multiple lines to list sdl2 properties. Make sure previous `sdl2 = "0.31.0"` entry is
removed.

Run the project!

Cargo will now fetch a gazilion of other dependencies (don't worry, they are build-time dependencies),
compile SDL2 with CMake (using MSVC compiler), make sure it is linked into `sdl2-sys` crate, which in turn
will be linked into `sdl2` crate.

The executable should run and return 0:

```txt
    ...
    
    Finished dev [unoptimized + debuginfo] target(s) in 136.27 secs
     Running `target\debug\lesson-01-window.exe`

Process finished with exit code 0
```

## SDL initialization and shutdown

Let's go back to the code.

```rust
fn main() {
    let _sdl = sdl2::init().unwrap();
}
```

Let's unpack it bit by bit.

`sdl2::init()` initializes SDL. It is educational to [go to `sdl2` documentation](https://rust-sdl2.github.io/rust-sdl2/sdl2/),
scroll down and find `init` function, click on it, [read it](https://rust-sdl2.github.io/rust-sdl2/sdl2/fn.init.html).

It is should be quite clear how it works. What may be not clear, is Rust-specific stuff.

The `init` function signature is `pub fn init() -> Result<Sdl, String>`, it returns
`Result<Sdl, String>`. If initialization was successful, the `Result` contains `Sdl` struct as an Ok value,
otherwise it contains the `String` as Err value.

[Rust standard libary reference](https://doc.rust-lang.org/std/) contains all information
about [the Result type](https://doc.rust-lang.org/std/result/enum.Result.html) and the methods
available on it. One of the methods is [unwrap](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap).

Calling `unwrap` on `Result<Sdl, String>` yields `Sdl` Ok value, or terminates the program
with an error message that contains `String` Err.

I recommend to [get familiar with error handling in Rust by reading the book](https://doc.rust-lang.org/book/second-edition/ch09-00-error-handling.html).

The Result type is very thin: if you are coming from C/C++, you can think of it as a union of two
types, wrapped in a struct with a discriminator flag.

We assign the returned `Sdl` to `_sdl` value with `let _sdl = ...` statement.
The `_` before variable name `_sdl` is useful to silence a warning about unused `sdl` value.

Notice that there is no explicit `SDL_Quit()`, but it is still executed automatically at the end of the 
`fn main`.

Rust language does not have garbage collector, instead variables are cleaned up as they leave scopes.
`Sdl` struct has a `drop` function that is executed before the clean up, and that's where the `SDL_Quit()` 
is invoked.

The story is not complete here however, because `Sdl` is secretly internally reference-counted. Therefore 
you need not to concern yourself of keeping it around, it will keep SDL alive as long as any other sdl object
has a clone of it.

## Video subsystem

In addition to SDL, let's initialize video subsystem and the window:

```rust
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("Game", 900, 700)
        .resizable()
        .build()
        .unwrap();
```

The `sdl.video()` will create a video subsystem that internally will contain clone of `Sdl`, so that
when `video_subsystem` is dropped, the `Sdl` reference-count is decreased, and it is dropped if 
reference-count reaches zero.

This pattern continues all over the API.

The `video_subsystem.window(...)` call does not return window itself. Instead, it creates a builder.

In [sdl api reference](https://rust-sdl2.github.io/rust-sdl2/sdl2/), find `VideoSubsystem`, and find
`window` function in it, with this signature:

```rust
impl VideoSubsystem {
    pub fn window(&self, title: &str, width: u32, height: u32) -> WindowBuilder
}
```

This is a method implemented for `VideoSubsystem`, therefore the first special parameter
`&self` refers to `VideoSubsystem` itself. It is akin to `this` in other languages.

When we call this method, we pass only the subsequent three parameters: title, width, height.

In this example, we set `resizable` flag from [all the other possible flags](https://rust-sdl2.github.io/rust-sdl2/sdl2/video/struct.WindowBuilder.html).

Call to `build` creates the window, and the `unwrap` handles possible error by terminating the program.

If you try to run this program, you may notice window flashing briefly before performing a graceful 
exit with correct resource cleanup. We are getting there.

## Loop

To keep window open, we add an infinite loop:

```rust
loop {

}
```

You will notice that this window is "not responding". This happens because when you do
any action related to window, such as move a mouse around, click, or try to resize it, 
OS is sending messages to window, but no one is receiving them.

## Receive window events

Before we start the loop, we initialize `EventPump` to receive application events 
(we may even have multiple windows).

Inside the loop, we iterate over all the received events, and then continue to loop:

```rust
    let mut event_pump = sdl.event_pump().unwrap();
    loop {
        for _event in event_pump.poll_iter() {
            // handle user input here
        }

        // render window contents here
    }
```

We again use prefix `_event` variable with underscore to silence unused event warning.

If we run the program now, the window comes to life! Albeit a bit empty inside. We will remedy
that soon.

When we try to close the window, OS sends `Quit` event, but our window is ignorant even of that.
Let's handle it:

```rust
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        // render window contents here
    }
```

We use `match` statement to match Quit event. There will be lots of pattern matching
when handling SDL events, so [I suggest you learn all about it](https://doc.rust-lang.org/book/second-edition/ch06-02-match.html).
No that's not all, [here is more about patterns](https://doc.rust-lang.org/book/second-edition/ch18-03-pattern-syntax.html).

We have also annotated the outer loop with `'main` label, so that `break 'main` breaks out of the `loop` loop
instead of inner `for` loop.

The code can be
 [found here](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-01).
 
With that, our window can finally be closed, and 
[we can start loading some color inside](/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html)!