---
layout: post
title:  "Rust and OpenGL from scratch - Failure"
date:   2018-02-15
categories: rust opengl tutorial
---

> ["Rust and OpenGL from scratch"](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html) 
> series aims to introduce you 
> to various Rust features and patterns, from simple to advanced, 
> from common to obscure, while at the same
> time working towards a simple OpenGL renderer.

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/14/opengl-in-rust-from-scratch-07-basic-resources.html), 
we have changed the way we load shaders. We now use our new `resources` module.

We did not take much care to make our errors look nice. This is what we
are going to look into today.

## Current state

In our [`resources` mod](https://github.com/Nercury/rust-and-opengl-lessons/blob/master/lesson-07/src/resources.rs),
we've implemented the error handling in a simplest possible way. It's fast too, even on an error path:
the `Error` enum is zero-cost wrapper around an integer that discriminates the type:

(resources.rs, already exists)

```rust
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}
```

We also added the error handling to `Shader` and `Program` struct initializers,
so that we can retrieve structured errors with full information:

(render_gl.rs, already exists)

```rust
#[derive(Debug)]
pub enum Error {
    ResourceLoad { name: String, inner: resources::Error },
    CanNotDetermineShaderTypeForResource { name: String },
    CompileError { name: String, message: String },
    LinkError { name: String, message: String },
}
```

Now, we can (sort of) print the error using the `Debug` output (`Debug` is also used in
`unwrap()` panics):

(main.rs, example)

```rust
let shader_program = render_gl::Program::from_res(
    &gl, &res, "shaders/triangle"
);
if let Err(e) = shader_program {
    println!("{:?}", e);
}
```

```txt
ResourceLoad { name: "shaders/triangle.frag", 
inner: Io(Error { repr: Os { code: 2, message: "The system cannot find the file specified." } }) }
```

This is not bad, but we could do better if we [implemented the `Display` trait](https://doc.rust-lang.org/beta/std/fmt/trait.Display.html) for error: we could
customize the error message as needed.

Luckily, there is an easier way out: the `failure` crate can do that for us.

## Failure

Let's add `failure` to our dependencies:

(Cargo.toml, incomplete)

```toml
[dependencies]
failure = "0.1"
```

And then reference it in `main`:

(main.rs, at the top of main.rs)

```rust
#[macro_use] extern crate failure;
```

`#[macro_use]` means the crate brings some macros into scope. In case of `failure`, it also
brings in something called "procedural macros", which can read our source code and generate
additional code at the compile time.

Let's derive `Fail` trait for our `Error` types:

(resources.rs, modified)

```rust
#[derive(Debug, Fail)] // derive Fail, in addition to Debug
pub enum Error {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
}
```

(render_gl.rs, modified)

```rust
#[derive(Debug, Fail)] // derive Fail, in addition to Debug
pub enum Error {
    ResourceLoad { name: String, inner: resources::Error },
    CanNotDetermineShaderTypeForResource { name: String },
    CompileError { name: String, message: String },
    LinkError { name: String, message: String },
}
```

The `failure` crate will now try to implement `Fail` trait. If we looked at [`Fail` trait docs](https://docs.rs/failure/0.1.1/failure/trait.Fail.html),
we would find that to implement `Fail`, `Debug` and `Display` also need to be implemented: 
(`Fail: Display + Debug + ...`). We've already added the `Debug`, and `failure` will try
to implement `Display`, it just needs a little bit of help. We help it with additional
attributes with "display" messages for each case:

(resources.rs, modified)

```rust
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(io::Error),
    #[fail(display = "Failed to read CString from file that contains 0")]
    FileContainsNil,
    #[fail(display = "Failed get executable path")]
    FailedToGetExePath,
}
```

(render_gl.rs, modified)

```rust
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad { name: String, inner: resources::Error },
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}
```

And then we can try print the same error using `Display` (`{}`) formatter:

(main.rs, example)

```rust
println!("{}", e);
```

```txt
Failed to load resource shaders/triangle.frag
```

It works! But we've lost all the deeper information 
that was in the inner `Resource` and then `Io` error. We can use `#[cause]` attribute inside `Resource` case
to inform `failure` that this is not simply some additional data for `Resource`, but the cause of it:

(render_gl.rs, modified)

```rust
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad { name: String, #[cause] inner: resources::Error }, // added #[cause] attribute
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}
```

Same for `Io` case in `resources::Error`: 

(resources.rs, modified)

```rust
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error), // added #[cause] attribute
    #[fail(display = "Failed to read CString from file that contains 0")]
    FileContainsNil,
    #[fail(display = "Failed get executable path")]
    FailedToGetExePath,
}
```

If we look again at the API of `Fail` trait, we can see that it has handy method to
retrieve all chain of causes for the error!

I've created a function that takes any object that implements `failure::Fail` and
prints out the chain of all causes:

(main.rs, add to the end)

```rust
pub fn failure_to_string<E: failure::Fail>(e: E) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
```

Using it to print our error produces excellent result:

(example)

```rust
println!("{}", failure_to_string(e));
```

```txt
The system cannot find the file specified. (os error 2)
   Which caused the following issue:
I/O error
   Which caused the following issue:
Failed to load resource shaders/triangle.frag
```

## Generic Failure Error

`failure` crate has another type of `Error`, named, unsurprisingly, `failure::Error`, that
can hold any type of error inside, given it implements the `Fail` trait. `failure::Error` has
`From` conversion from anything that implements `Fail`.

The upside of this that we can convert any kind of error to `failure::Error`, and forget
about the exact error type! The `failure::Error` has all the same methods that can give
the same nice chain of causes.

It does not come without downsides, though:

- To have this kind of dynamic dispatch, `failure::Error` needs additional allocation inside.
So while simple `enum` is as fast as return code in C, `failure::Error` is not.
- If you need to inspect an error, it is much easier to do that with `match` statements.
We can't easily match agains an error that is inside `failure::Error` (although it is possible
to check if certain error exists inside).

Therefore the place we will use `failure::Error` is at the top level of our application,
where we need to print it, and avoid it in all the lower level modules.

We will finally be able to get rid of these `unwrap()` calls everywhere. For string errors,
we will use [`failure::err_msg` which can convert](https://boats.gitlab.io/failure/doc/failure/fn.err_msg.html) 
any displayable type to `failure::Error`.

Finally, we can move our program logic out of `main()` into a new `run()` function, 
that returns `Result<(), failure::Error>`. And then the only thing main is going to do
is invoke `run()` and print a nicely formatted message:

(main.rs, partial)

```rust
extern crate sdl2;
extern crate gl;
#[macro_use] extern crate failure;

pub mod render_gl;
pub mod resources;

use failure::err_msg;
use resources::Resources;

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_exe_path()?;

    let sdl = sdl2::init().map_err(err_msg)?;
    
    ...

    Ok(())
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
```

Make sure to check out the [failure's book](https://boats.gitlab.io/failure/intro.html), 
which explains these usage patterns in a detailed way.

[Next time](/rust/opengl/tutorial/2018/06/27/opengl-in-rust-from-scratch-09-vertex-attribute-format.html),
we will look into writing a nicer abstraction for vertex attribute
layout and format.

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-08).