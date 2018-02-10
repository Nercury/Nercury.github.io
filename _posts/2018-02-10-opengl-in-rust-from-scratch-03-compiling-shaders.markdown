---
layout: post
title:  "Rust and OpenGL from scratch - Compiling Shaders"
date:   2018-02-10
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html), 
we have loaded OpenGL context for SDL window, to handle user input
and output pixels.

In this lesson, we will work towards rendering the classic OpenGL triangle. Classic, because
every every OpenGL tutorial does this.

But first, we will learn how to create safe abstractions in Rust, and we will
build tools for compiling a shader and linking a program.

## "Modern" OpenGL

We will be using what is called "modern OpenGL". Turns out, long ago, it wasn't so
modern. We won't discuss graphics pipeline here from the beginning: instead, I suggest
you follow along using another "modern OpenGL tutorial". I will be using 
[this awesome tutorial as the base](https://learnopengl.com/). 
This lesson will cover more Rust-y side of 
["Hello Triangle" part](https://learnopengl.com/Getting-started/Hello-Triangle).

## Additional OpenGL context setup

Previous part of [learnopengl.com lesson talks about creating a window](https://learnopengl.com/Getting-started/Hello-Window). We've mostly done that,
except I forgot a few things.

First, we need to specify the minimal OpenGL version to use. In our case, we can do that
using SDL gl attributes:

(main.rs, incomplete)

```rust
let gl_attr = video_subsystem.gl_attr();

gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
gl_attr.set_context_version(4, 5);

let window = ... // create the window
```

I will be using quite recent (Core 4.5) version here, but feel free to dial it down
if your context can not be created, down to `Core 3.3` used in `learnopengl.com` tutorial.

Second, we need to set up our viewport. We can do it once, just before `ClearColor`:

(main.rs, incomplete)

```rust
unsafe {
    gl::Viewport(0, 0, 900, 700); // set viewport
    gl::ClearColor(0.3, 0.3, 0.5, 1.0);
}
```

### I see unsafe code

Yeah, me too. Let's continue.

## Shader

We will make a helper function to compile a shader from string, and then another function
to link compiled shaders into a program.

First try:

(main.rs, at the end)

```rust
fn create_shader(source: &str) -> gl::types::GLuint {
    // continue here
}
```

Given a source code, this function should return shader id (as an int).

However, the shader creation may fail, and we may want to retrieve the error message.
For that, we will change return type to `Result<gl::types::GLuint, String>`. If `Result` is `Ok`, we will
get shader id, otherwise we can extract an error message from `Err`.

```rust
fn create_shader(source: &str) -> Result<gl::types::GLuint, String> {
    ...
}
```

Let's start by obtaining shader object id:

```rust
fn create_shader(source: &str) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    
    // continue here
}
```

Ha! We need to specify the shader type. Let's improve the function signature and add
the shader type to it. `type` is a reserved keyword in Rust, but we can name it `kind`:

```rust
fn create_shader(
    source: &str,
    kind: gl::types::GLuint
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    
    // continue here
}
```

Next, we need to set the source for the shader object using [glShaderSource](http://docs.gl/gl4/glShaderSource) function.

However, it is C function, so the third `string` argument passed to it has to be zero terminated C string.
Rust strings are not zero terminated, and they can even contain 0 values inside.

There are two ways to deal with this: convert Rust string slice `&str` to `CString` inside this function,
or let function caller deal with that. I am going to choose the second option, because `CString`s can
also be created directly from bytes, and we will later load our shaders from files, which can be read
as ASCII into byte arrays.

Let's change function parameter from `&str` to `&std::ffi::CStr`:

```rust
// import namespace to avoid repeating `std::ffi` everywhere
use std::ffi::{CString, CStr};

fn create_shader(
    source: &CStr, // modified
    kind: gl::types::GLenum
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    
    // continue here
}
```

`&CStr` can be borrowed from owned `CString` the same way a `&str` is borrowed from a `String`.
Oh, and by the way, [knowing basics about Rust's ownership and borrowing would be required from now on](https://doc.rust-lang.org/book/second-edition/ch04-00-understanding-ownership.html).

With that, we can set shader source and compile it:

```rust
unsafe {
    gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
    gl::CompileShader(id);
}

// continue here
```

We could return `Ok(id)` now and move on, however, we __really__ need to see a proper
error message if the shader fails to compile.

Therefore, we obtain the shader compilation status:

```rust
let mut success: gl::types::GLint = 1;
unsafe {
    gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
}
```

And if it is `0`, we will return an error string, otherwise `Ok(id)`:

```rust
if success == 0 {
    // continue here
}

Ok(id)
```

We will need to write returned error to buffer, therefore we need to know the required length of this buffer.

We get the `len` by querying `GL_INFO_LOG_LENGTH` for the shader object:

```rust
let mut len: gl::types::GLint = 0;
unsafe {
    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
}

// continue here
```

For our buffer, we allocate `Vec` of the correct length and fill it with spaces. Then 
we create `CString` from it (which is going to reuse the same allocation, as well as append 0 at the end):

```rust
// allocate buffer of correct size
let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
// fill it with len spaces
buffer.extend([b' '].iter().cycle().take(len as usize));
// convert buffer to CString
let error: CString = unsafe { CString::from_vec_unchecked(buffer) };

// continue here
```

This code might be a tad confusing:

```rust
[b' '].iter().cycle().take(len as usize)
```

- `[b' ']` is a single-item stack-allocated array which contains ASCII "space" byte.
- `.iter()` obtains iterator over it.
- `.cycle()` repeats this iterator forever, yielding infinite number of spaces.
- `.take(len as usize)` limits returned items to `len`.

The `buffer.extend(...)` for its argument accepts iterator, from which it appends new items.
Iterators are zero-cost, they require no additional allocations and compile down to
compact instructions.

With that, we can ask OpenGL to write shader info log into our `error` value:

```rust
unsafe {
    gl::GetShaderInfoLog(
        id,
        len,
        std::ptr::null_mut(),
        error.as_ptr() as *mut gl::types::GLchar
    );
}

// continue here
```

And finally, we can return an error:

```rust
return Err(error.to_string_lossy().into_owned());
```

`error.to_string_lossy()` converts `CString` to Rust String, replacing any invalid unicode
characters with unicode error character. `to_string_lossy` is implemented on `CStr`, but we were 
able to call it on `CString`, because it inherits all `CStr` methods. However, 
`to_string_lossy` returns a value that can be either String or `&str`, so we use `into_owned` to
obtain a definite String. This last unfortunate step requires an additional allocation, but this
is the error handling path, so hopefully it is not a big deal.

Congratulations! You have just created your first safe API for a bunch of unsafe functions!

However, there are some things we can do to make this API nicer.

### Extracting CString creation from create_shader

We can extract the lines that help us create new empty `CString` to another function.
We might need it later:

(main.rs, append at the end)

```rust
fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
```

And use it in original location:

```rust
let error = create_whitespace_cstring_with_len(len as usize);
```

### Creating a newtype for shader object

Instead of returning a shader object id, we can return a struct named `Shader` that wraps
this `id`. This returned struct would have no overhead, and will consume exactly the same
amount of bytes as `gl::types::GLuint`, but can be much more convenient to use:

(main.rs, above create_shader function)

```rust
struct Shader {
    id: gl::types::GLuint,
}

// continue here
```

And then implement `create` function for Shader:

```rust
impl Shader {
    fn create(
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = create_shader(source, kind)?;
        Ok(Shader { id })
    }
    
    // continue here
}
```

`create` does not have `self` first parameter, and is akin to static functions in Java-like languages.
It is called using `Shader::create` syntax and acts as a constructor.

The return type of it is `Result<Shader, String>`, which may be either the successfully created `Shader`,
or an error `String`.

The question mark `?` at the end of `let id = create_shader(source, kind)?;` line
does the same as would this code:

(example)

```rust
let id = match create_shader(source, kind) {
    Ok(id) => id,
    Err(error) => return Err(error.into()),
};
```

In case the Result was Ok, this block would unwrap id from Ok and assign it to `id` variable,
and in case of Err, the function would return with the same error value.

There is much more to error handling in Rust, 
[and I highly recommend to not skip learning about it](https://doc.rust-lang.org/book/second-edition/ch09-02-recoverable-errors-with-result.html).

### Additional constructor methods for Shader

Now our shader can be created like this:

(example)

```rust
let shader = Shader::create(
    &CStr::from_bytes_with_nul(b"<source code here>\0").unwrap(),
    gl::VERTEX_SHADER
).unwrap();
```

We can create two more helper methods, `create_vert` and `create_frag`, so that we can skip
`gl::VERTEX_SHADER` parameter:

```rust
impl Shader {
    fn create(...) { ... }
    
    fn create_vert(source: &CStr) -> Result<Shader, String> {
        Shader::create(source, gl::VERTEX_SHADER)
    }

    fn create_frag(source: &CStr) -> Result<Shader, String> {
        Shader::create(source, gl::FRAGMENT_SHADER)
    }
}

// continue here
```

### Resource cleanup

As implemented, our Shader type leaks shader id without cleaning it up.

To clean it up, we will implement `Drop` trait for the `Shader`:

(after `impl Shader {}` block)

```rust
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if gl::IsShader(self.id) == gl::TRUE {
                gl::DeleteShader(self.id);
            }
        }
    }
}
```

[By the looks of glDeleteShader documentation](http://docs.gl/gl4/glDeleteShader) it should be
safe to straight up delete the shader, even if the shader object id is 0, and even if the shader is
attached to a program. However, I am worrying about cases where OpenGL context is lost, and we are left with
invalid shader object id.
In this case, `DeleteShader` would generate an error, which would be confusing; therefore I am adding
the `glIsShader` check before issuing the delete call.

## Moving Shader implementation into another module

I am going to call new module `render_gl`, because it will contain utilities such
as the shader for gl rendering.

Let's wrap everything bellow the main function into `pub mod render_gl {}` block. 
Modules do not inherit imports from the parent modules. Therefore we will need to add
`use gl;` at the top of child `render_gl` module. Moreover, by default
Rust imports `std` to the root crate module; therefore we will need `use std;` in child too.

If you try to use `render_gl::Shader` from this module in main and compile our app,
you will get errors saying that `Shader is private`;
we will need to add `pub` keywords for anything we want to be visible from the outside of the module.
In our case, it is the `Shader` struct with the `create...` methods.

With all of that sorted out, the full module code looks like this:

```rust
pub mod render_gl {
    use gl;
    use std;
    use std::ffi::{CString, CStr};

    pub struct Shader {
        id: gl::types::GLuint,
    }

    impl Shader {
        pub fn create(
            source: &CStr,
            kind: gl::types::GLenum
        ) -> Result<Shader, String> {
            let id = create_shader(source, kind)?;
            Ok(Shader { id })
        }

        pub fn create_vert(source: &CStr) -> Result<Shader, String> {
            Shader::create(source, gl::VERTEX_SHADER)
        }

        pub fn create_frag(source: &CStr) -> Result<Shader, String> {
            Shader::create(source, gl::FRAGMENT_SHADER)
        }
    }

    impl Drop for Shader {
        fn drop(&mut self) {
            unsafe {
                if gl::IsShader(self.id) == gl::TRUE {
                    gl::DeleteShader(self.id);
                }
            }
        }
    }

    fn create_shader(
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<gl::types::GLuint, String> {
        let id = unsafe { gl::CreateShader(kind) };
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(id)
    }

    fn create_whitespace_cstring_with_len(len: usize) -> CString {
        // allocate buffer of correct size
        let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
        // fill it with len spaces
        buffer.extend([b' '].iter().cycle().take(len));
        // convert buffer to CString
        unsafe { CString::from_vec_unchecked(buffer) }
    }
}
```

### Moving render_gl module into another file

It is quite simple: create a new file named `render_gl.rs` in `src`, and move the contents of 
`pub mod render_gl {  }` block into it. Then, add a seminocol at the end of `pub mod render_gl`:

```rust
pub mod render_gl;
```

When the Rust compiler encounters a module name with no block `{}`, it tries to load the contents
from a file named with the same name and suffixed with `.rs`; in this case it would be `render_gl.rs`.
If such a file does not exist, it will then try to load contents from `render_gl/mod.rs` file (this
is useful to further subdivide submodules into smaller ones). If that also fails, then we get a 
compilation error.

Module references usually appear near the top of the file, bellow `extern crate` references.
We can move `pub mod render_gl;` there.

## Program

Linking program requires these OpenGL calls:

(example)

```rust
let program_id = unsafe { gl::CreateProgram() };

unsafe {
    gl::AttachShader(program_id, vert_shader_id);
    gl::AttachShader(program_id, frag_shader_id);
    gl::LinkProgram(program_id);
}
```

We can add a function to out `Shader` struct to retrieve shader id:

(render_gl.rs, at the end of `impl Shader` block)

```rust
impl Shader {
    ...

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}
```

This makes `Shader` struct useful even if we never create abstraction for linking
a program:

(example)

```rust
let vert_shader = Shader::create_vert(..)?;
let frag_shader = Shader::create_frag(..)?;

let program_id = unsafe { gl::CreateProgram() };

unsafe {
    gl::AttachShader(program_id, vert_shader.id());
    gl::AttachShader(program_id, frag_shader.id());
    gl::LinkProgram(program_id);
}
```

One small thing: `glDeleteShader` won't delete the shader if it is still
attached to program. Therefore we detach the shader after linking it:

(example)

```rust
let vert_shader = Shader::create_vert(..)?;
let frag_shader = Shader::create_frag(..)?;

let program_id = unsafe { gl::CreateProgram() };

unsafe {
    gl::AttachShader(program_id, vert_shader.id());
    gl::AttachShader(program_id, frag_shader.id());
    gl::LinkProgram(program_id);
    gl::DetachShader(program_id, vert_shader.id());
    gl::DetachShader(program_id, frag_shader.id());
}
```

We will follow the same pattern as for `Shader` struct for our new `Program` struct.
We will wrap the OpenGL program object id, add a static method
that creates the program and links it to specified shaders, and a drop function
that deletes the program object:

(at the top of render_gl.rs file, bellow `use` statements)

```rust
pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn create_linked(shaders: &[&Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }

        unsafe { gl::LinkProgram(program_id); }
        
        // continue with error handling here

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        Ok(Program { id: program_id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            if gl::IsProgram(self.id) == gl::TRUE {
                gl::DeleteProgram(self.id);
            }
        }
    }
}
```

`create_linked` has a single `shaders` parameter, that takes a _slice_ (`&[]`) of
`Shader` references (`&Shader`). They are required just for the linking.

We also need the error handling, which is very similar to error handling of a shader program:

(render_gl.rs, bellow `gl::LinkProgram`)

```rust
let mut success: gl::types::GLint = 1;
unsafe {
    gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
}

if success == 0 {
    let mut len: gl::types::GLint = 0;
    unsafe {
        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
    }

    let error = create_whitespace_cstring_with_len(len as usize);

    unsafe {
        gl::GetProgramInfoLog(
            program_id,
            len,
            std::ptr::null_mut(),
            error.as_ptr() as *mut gl::types::GLchar
        );
    }

    return Err(error.to_string_lossy().into_owned());
}
```

Instead of `GetShaderiv`, we use `GetProgramiv`, instead of `GetShaderInfoLog` we use `GetProgramInfoLog`,
and instead of `COMPILE_STATUS` we use `LINK_STATUS`.

## Using our Shader and Program structs

Let's add `glUseProgram` for our `Program` struct:

(render_gl.rs, impl Program)

```rust
pub fn set_used(&self) {
    unsafe {
        gl::UseProgram(self.id);
    }
}
```

I picked the name `set_used`, because `use` is unfortunately a reserved keyword.

Then, right before the `'main` loop, we can compile our vertex and shader programs:

(main.rs, before loop)

```rust
use std::ffi::CString;

let vert_shader = render_gl::Shader::create_vert(
    &CString::new(include_str!("triangle.vert")).unwrap()
).unwrap();

let frag_shader = render_gl::Shader::create_frag(
    &CString::new(include_str!("triangle.frag")).unwrap()
).unwrap();

// continue here
```

- We may include `use` block anywhere. I add `use std::ffi::CString` close to `CString` usage,
because ideally we would move this code together with the `use` to another function or file.
- When be borrow `&` `CString`, it is coerced into `&CStr`. [This is the same mechanism
that makes `&String` or `&Vec<T>` arguments work for `&str` or `&[T]` parameters](https://doc.rust-lang.org/book/first-edition/deref-coercions.html).
- `include_str!` macro embeds a UTF-8 file contents as a static `&str` value. Essentially the file
gets compiled into our executable as string.
- `.unwrap` for `CString::new` is needed because Rust's UTF-8 strings may contain 0 in the middle and be valid.
This should basically never happen, so we use `unwrap` to panic the program if it does.
- `.unwarp` on `Shader::create_...` functions will panic and terminate our program with a debug
message if shaders fail to compile. This is temporary way we handle the compilation.

Then, we link our shaders:

(main.rs, before loop)

```rust
let shader_program = render_gl::Program::create_linked(
    &[&vert_shader, &frag_shader]
).unwrap();

// continue here
```

And finally, use them:

(main.rs, before loop)

```rust
shader_program.set_used();
```

However, we won't see anything on screen yet, because we are not sending
any draw commands to OpenGL yet.

We will remedy that the next time.

As always, [the code for this lesson is available on github].