---
layout: post
title:  "Rust and OpenGL from scratch - GL Generator"
date:   2018-02-12
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/11/opengl-in-rust-from-scratch-05-triangle-colors.html), 
we have rendered a colored triangle to the window.

Our program was a mixed bag of decent shader compiler code and the error-prone VBO buffer and VAO
layout handling mess.

Before getting deep into improving this situation, we are going to step back a bit and improve
the way we load and use OpenGL. We will create our own project-specific `gl` crate.

This will allow us:

- To ensure gl context functions invoked from the thread they were created from;
- Enable debug checks for all gl function calls;
- Have control of enabled OpenGL extensions;
- Have control of supported API level, i.e. remove new OpenGL functions that we are not using.

## Build script

[The entire implementation of `gl` crate is this](https://github.com/brendanzab/gl-rs/blob/master/gl/src/lib.rs):

(src/lib.rs in gl crate)

```rust
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

Crate code is included from this mysterious `bindings.rs` file.

The code in `bindings.rs` is auto-generated in a build script, which is in the crate's root
directory. We have [discussed that this was happening previously](/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html), but now we are going to really 
dig into it. [Let's take a look at the build script's code](https://github.com/brendanzab/gl-rs/blob/master/gl/build.rs):

(build.rs in gl crate)

```rust
extern crate gl_generator;

use gl_generator::{Registry, Fallbacks, GlobalGenerator, Api, Profile};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&out_dir).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
```

[Build scripts allow the crate authors to include code that is executed before the build
by the cargo](https://doc.rust-lang.org/cargo/reference/build-scripts.html).

When running the build script, cargo pass various useful environment variables, such
as `OUT_DIR`. The `OUT_DIR` points to build directory, where the script writes
generated `bindings.rs` code.

Then, the contents of this file are included in `src/lib.rs` by referencing the same
environment variable `OUT_DIR`.

## Our own gl crate

Let's create a new project-specific `gl` crate, which won't be published to `crates.io`.

(in project root)

```txt
> mkdir lib
> cd lib
> cargo new --vcs none gl
```

`--vcs none` will disable `.git` for this library, because it will be using our project's VCS (if any).

I had this structure in mind:

![Triangle](/images/opengl-rust/06/lib-gl-crate.jpg)

You may pick other name than `/lib` in your project, it's completely arbitrary.

Now we simply copy-paste the setup from original `gl` project, making changes that suits
our project.

We write the same brief code in `lib.rs`:

(lib/gl/src/lib.rs)

```rust
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

Add `[build-dependencies]` section (which is used for build-script) with `gl-generator`
dependency (expect the relative path thing):

(lib/gl/Cargo.toml, incomplete)

```toml
[build-dependencies]
gl_generator = { version = "0.9.0" }
```

This can be simplified to:

(lib/gl/Cargo.toml, incomplete)

```toml
[build-dependencies]
gl_generator = "0.9.0"
```

And we add modified build script:

(lib/gl/build.rs, incomplete)

```rust
extern crate gl_generator;

use gl_generator::{Registry, Fallbacks, StructGenerator, Api, Profile};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file_gl = File::create(&Path::new(&out_dir).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [
        "GL_NV_command_list", // additional extension we want to use
    ])
        .write_bindings(
            StructGenerator, // different generator
            &mut file_gl
        )
        .unwrap();
}
```

As an example we can include some extension, like "GL_NV_command_list", and we 
use different `StructGenerator` instead of `GlobalGenerator`.

## Fixing up our project

In our project dependencies, we can point `gl` to our new `gl` crate:

(Cargo.toml, incomplete)

```toml
[dependencies]
gl = { path = "lib/gl" }
```

When we now try to compile our project, we get bunch of errors. This happens because
with `StructGenerator` OpenGL methods are not global, but members of a struct `gl::Gl`, 
which we should probably now use everywhere.

### First attempt: hello, lifetime specifier

However, if we tried to use a reference to `gl::Gl` in our `Program` struct, we would quickly encounter few issues:

(example, wait, don't do this)

```rust
impl Program {
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        // use "." instead of "::"
        let program_id = unsafe { gl.CreateProgram() }; 
        
        ...
    }
}
```

We would be able to do this in "from_shaders" function, however, `drop` function can not have
any additional parameters:

(example, wait, don't do this)

```rust
impl Drop for Program {
    fn drop(&mut self) { // "Drop" will only have "&mut self" parameter
        unsafe {
            // no "gl" here!
        
            gl.DeleteProgram(self.id);
        }
    }
}
```

First idea, especially if we are new to Rust, would be to store `&Gl` in `Program` struct,
so that we can access it from `drop`:

(example, wait, don't do this, but you may try)

```rust
pub struct Program {
    gl: &gl::Gl,
    id: gl::types::GLuint,
}

... 

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}
```

Then, Rust would inform us that we don't have lifetime specifier:

```txt
error[E0106]: missing lifetime specifier
 --> src\render_gl.rs:6:9
  |
6 |     gl: &gl::Gl,
  |         ^ expected lifetime parameter
```

Then we may add the requested specifier to the `Program`:

(example, wait, don't do this, but you may try)

```rust
pub struct Program<'a> {
    gl: &'a gl::Gl,
    id: gl::types::GLuint,
}
```

After adding lifetime specifier to multiple other places, we will realize that
while it works, `Program<'a>` is very inflexible to use. If we tried, as an example, 
to include `Program<'a>` in another `Struct`, that struct would also need to gain a lifetime
specifier `Struct<'a>`. Consider briefly, that our `Program` won't be the only thing
to use `Gl`, and if all the things required lifetime specifiers, we would have
a very bad time.

Lifetime is inconvenient thing to use in this case. Lifetime specifier `'a` on a struct
means that it contains a pointer to some memory location, and while this struct
is in use, this memory location can not be:

- Moved
- Modified
- Deallocated

The lifetime specifier would make much more sense on something like iterator,
where we would need to ensure that the original data we are iterating over is not modified,
because our iterator has pointers to it.

Ah, you might say, but objects __are__ pointers! Yes, but in other languages pointers usually
point to heap, while Rust's references can point to both! Rust's references are nothing
like objects in Java-like languages, they are pointers to a memory location. And Rust ensures
they are valid at compile time.

### Second attempt: manual de-init

Ok, ok, we give up. What if we do not store `&Gl` in a `Program`, but add a manual `deinit`
function, where we can pass the `&Gl` argument?

(example)

```rust
impl Drop for Program {
    fn deinit(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.DeleteProgram(self.id);
        }
    }
}
```

Of course, we would need to call this `deinit` manually:

```rust
program.deinit(&gl);
```

Not a big problem, right?

It is certainly possible to go this way, and go quite a long way before it becomes annoying. 
However, if we follow this pattern, every thing will get its `init` and `deinit` functions, which 
we will have to always remember to call, sometimes in correct other, and sometimes with a boolean
check. For example, in a hypothetical terrains renderer (which I might have written), or in 
some hypothetical future tutorial (which I do not promise to write), we might see this:

```rust
impl Terrains {
    pub fn init(&mut self, gl: &Gl) {
        self.shader.init(gl);
        self.grid_texture.init(gl);
        self.overlay_texture.init(gl);
        self.tiles_texture.init(gl);
        if self.debug {
            self.debuglines.init(gl);
        }
        self.terrains.load(gl);
    }
    
    pub fn deinit(&mut self, gl: &Gl) {
        self.shader.deinit(gl);
        self.grid_texture.deinit(gl);
        self.overlay_texture.deinit(gl);
        self.tiles_texture.deinit(gl);
        if self.debug {
            self.debuglines.deinit(gl);
        }
        self.terrains.unload(gl);
    }
}
```

It might work. That's all I can say about this approach.

### Almost final attempt: ownership

What if, instead of storing a pointer (yes, it is better to think of references as pointers) in 
a `Program` (and later `Shader`) structs, we put a value there?

(let's do this! in render_gl.rs)

```rust
pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}
```

By placing a value in a struct in Rust, we express the intention for that value
to be deleted togeter with a parent struct.

But we may not want Gl to be deleted, right? Yes, but Gl can be cloned:

(example)

```rust
let gl2 = gl.clone();
```

Gl implements [Clone trait](https://doc.rust-lang.org/std/clone/trait.Clone.html), which performs
a deep copy of gl value. When we create our shaders and program, we can pass clone of it everywhere:

(main.rs, modified)

```rust
let vert_shader = render_gl::Shader::from_vert_source(
    gl.clone(), &CString::new(include_str!("triangle.vert")).unwrap()
).unwrap();

let frag_shader = render_gl::Shader::from_frag_source(
    gl.clone(), &CString::new(include_str!("triangle.frag")).unwrap()
).unwrap();

let shader_program = render_gl::Program::from_shaders(
    gl.clone(), &[vert_shader, frag_shader]
).unwrap();
``` 

Let's roll with it for now, and also add it to `Shader` struct:

(render_gl.rs, modified)

```rust
pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}
```

Modify constructor methods and implementations:

(render_gl.rs, modified)

```rust
impl Program {
    pub fn from_shaders(gl: gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };
        
        ...
        
        Ok(Program { gl, id: program_id })
    }
}
```

A standalone method such as `shader_from_source` that we have written previously does not need 
full ownership of `gl`, a reference will suffice:

(render_gl.rs, modified)

```rust
fn shader_from_source(
    gl: &gl::Gl, // a reference to gl
    source: &CStr,
    kind: gl::types::GLenum
) -> Result<gl::types::GLuint, String> {
    ...
}
```

In `main.rs`, we will need to use new `Gl` constructor to initialize `gl`:

```rust
let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
```

When we do all these replacements, the program should compile and run.

## Final version: shared gl reference

We are doing a deep copy of `Gl` struct. How big is it exactly?

```rust
println!("size of Gl: {}", std::mem::size_of_val(&gl));
```

```txt
size of Gl: 11392
```

Oops, 11KB deep cloning sounds a bit worrisome.
We can put the `gl` struct behind a reference-counted pointer, by wrapping it in
`Rc<Gl>` type:

(example)

```rust
use std::rc::Rc;
let gl = Rc::new(
    gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void)
);
```

But then, every place we use `Gl` would suddenly need the type `Rc<Gl>`, which is annoying.
It would be great if `Gl` was already reference counted inside, then we could clone it cheaply.
If only we had an access to the `gl` crate, we could change what `Gl` is... wait.

### Customised gl crate

In our `gl` crate, we can put generated bindings inside a private `bindings` module:

(rewrite lib/gl/src/lib.rs)

```rust
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// continue here
```

Let's export all the types in `bindings`:

```rust
pub use bindings::*;

// continue here
```

Then, add `Gl` struct with original reference-counted `Gl` inside. The `#[derive(Clone)]` will
will implement `Clone` trait, which will deep clone everything inside. Except, cloning
of `Rc` only increases its reference-count, while it keeps pointing to the same data.
The data is destroyed only when all `Rc` instances are deallocated and the shared 
reference-count reaches zero:

```rust
use std::rc::Rc;

#[derive(Clone)]
pub struct Gl {
    inner: Rc<bindings::Gl>,
}

// continue here
```

To initialize our Gl, forward `load_with` constructor, which will create original `Gl`, but then
wrap it in `Rc`:

```rust
impl Gl {
    pub fn load_with<F>(loadfn: F) -> Gl
        where F: FnMut(&'static str) -> *const types::GLvoid
    {
        Gl {
            inner: Rc::new(bindings::Gl::load_with(loadfn))
        }
    }
}

// continue here
```

We would not want to use `gl.inner` everywhere to access wrapped value, but [there is a
mechanism in rust to forward calls to inner implementation](https://doc.rust-lang.org/book/second-edition/ch15-02-deref.html), which is perfect for this use
case:

```rust
impl Deref for Gl {
    type Target = bindings::Gl;

    fn deref(&self) -> &bindings::Gl {
        &self.inner
    }
}
```

This is the exact same mechanism that forwards `&str` or `[T]` slice functions to `String` or `Vec<T>`.

With that, everything should magically compile now! If we check the size of `Gl` now...

```rust
println!("size of Gl: {}", std::mem::size_of_val(&gl));
```

```txt
size of Gl: 8
```

8 bytes! It's a pointer, finally!

You may recognise that we have not done anything too exceptional here: for example, many
structs in Rust's `sdl` crate wrap reference-counted implementations inside.

### Some cleanup

Our methods now clone `gl` at the call site:

(example)

```rust
let shader_program = render_gl::Program::from_shaders(
    gl.clone(), // cloned at the call site
    &[vert_shader, frag_shader]
).unwrap();
```

Instead, it is a bit more clean to pass a reference, and leave the decision to clone to the implementation:

(main.rs, modify method calls)

```rust
let shader_program = render_gl::Program::from_shaders(
    &gl, // pass reference
    &[vert_shader, frag_shader]
).unwrap();
```

And then modify constructors for `Shader` and `Program`:

(render_gl.rs, `Program` modification sample)

```rust
impl Program {
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        ...

        Ok(Program { gl: gl.clone(), id: program_id })
    }
}

// same with Shader
```

## Documentation

Documentation for _your own_ gl crate can be generated in the usual way:

```txt
cargo doc -p gl --no-deps --open
```

However, now we can't see the full function list, because the original `Gl` became private.
We can re-export it with different name to remedy the situation:

(lib/gl/src/lib.rs)

```rust
pub use bindings::Gl as InnerGl;
```

It's a bit unfortunate though that `Gl` and `bindings::Gl` causes some documentation conflicts.

## Debug Struct Generator

Besides the `StructGenerator`, `gl_generator` crate also contains `DebugStructGenerator`,
which generates implementation that issues error checks after every OpenGL call.

Let's extend our `gl` crate to have `debug` feature, which, when enabled, will compile
`gl` crate using `DebugStructGenerator`.

First, add a new feature to `Cargo.toml`:

(lib/gl/Cargo.toml, incomplete)

```toml
[features]
debug = []
```

In our main `Cargo.toml`, we can re-export `debug` feature from `gl`, so that when
the `debug` is enabled for the main project, child `gl` project is also compiled in `debug` mode:

(Cargo.toml)

```toml
[features]
gl_debug = ["gl/debug"]
```

Compiling crate with `cargo run --features "gl_debug"` will enable this feature for the
project. Note that `cargo run --release` flag is completely different thing (it enables
optimizations), which is what we want. We may need to compile project in release mode
but with OpenGL error checks enabled.

Right, but now we need to actually use this `debug` feature in a build script.

[Cargo documentation](https://doc.rust-lang.org/cargo/reference/environment-variables.html) 
says that our feature is enabled if `CARGO_FEATURE_<name>` environment variable is set.

Let's modify `gl` build script:

(lib/gl/build.rs, full)

```rust
extern crate gl_generator;

use gl_generator::{Registry, Fallbacks, StructGenerator, DebugStructGenerator, Api, Profile};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file_gl = File::create(&Path::new(&out_dir).join("bindings.rs")).unwrap();

    let registry = Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [
        "GL_NV_command_list",
    ]);

    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        registry.write_bindings(
            DebugStructGenerator,
            &mut file_gl
        ).unwrap();
    } else {
        registry.write_bindings(
            StructGenerator,
            &mut file_gl
        ).unwrap();
    }
}
```

Let's make a mistake in our program. What would happen, if instead of binding to `vbo`
variable, we bind to a number 42?

(main.rs, test)

```rust
gl.BindBuffer(gl::ARRAY_BUFFER, 42);
```

Output:

```
[OpenGL] BindBuffer(34962, 42)
[OpenGL] ^ GL error triggered: 1282
```

Well, I guess it's better than nothing!

## Autocomplete

One small thing - my IDE offers no autocomplete for the generated code. However,
it is easy to get it by copy-pasting whole auto-generated `bindings.rs` from
`/target/debug/gl-...` to `mod bindings` namespace. IntelliJ can then even
display proper documentation annotations.

## Discussion

Arguments for using a custom generator are not strong. In the majority of cases, global
`gl` functions would do the job just fine. However, it is good to know this
option is available if needed.

[The code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-06).

From now on, we will continue using this custom gl crate.
