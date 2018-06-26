---
layout: post
title:  "Rust and OpenGL from scratch - Vertex Attribute Format"
date:   2018-06-27
categories: rust opengl tutorial
---

Welcome back!

Previously, 
we have spent a few lessons [taking control of our gl bindings](/rust/opengl/tutorial/2018/02/12/opengl-in-rust-from-scratch-06-gl-generator.html), [setting up simple
resource loader from files](/rust/opengl/tutorial/2018/02/14/opengl-in-rust-from-scratch-07-basic-resources.html) and [tidying up the error handling with the `failure` crate](/rust/opengl/tutorial/2018/02/15/opengl-in-rust-from-scratch-08-failure.html).

It's time we start tackling the most error-prone part of our sample code, the one that configures 
vertex array object attributes.

If you were not following the series [from the beginning](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html), you can jump-in by [loading the
code from the previous lesson](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-08), and
I will try my best to give enough context to follow from there. 

## Current state
 
Currently, we set up Vertex Array Object (VAO) once:

(example, main.rs, run() function)

```rust
let mut vao: gl::types::GLuint = 0;
unsafe {
    gl.GenVertexArrays(1, &mut vao);
}

// continued from here
```

Then, inside the unsafe block (because all the functions are unsafe, we just wrap them all inside a single
unsafe block), we bind array and buffer (so that they are linked):

```rust
unsafe {
    gl.BindVertexArray(vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    
    // continued from here
}
```

Say that we are setting up `0`-th attribute of VAO:

```rust
    gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
    
    // continued from here
```

And and finally set up the format of this attribute. In our case, it is used to pass position to shader:

```rust
    gl.VertexAttribPointer(
        0, // index of the generic vertex attribute ("layout (location = 0)")
        3, // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        std::ptr::null() // offset of the first component
    );
    
    // continued from here
```

Then, we do the same for the color attribute:

```rust
    gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
    gl.VertexAttribPointer(
        1, // index of the generic vertex attribute ("layout (location = 0)")
        3, // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
    );
```

Note how easy it is to make a mistake here. And this is only an example of a simple triangle! 

In this lesson, we will focus on a zero-cost abstraction
that will help us reduce the possibility of mistakes while setting up VAOs.

## What we are dealing with

`VertexAttribPointer` call definition looks like this:

```C
void glVertexAttribPointer(
    GLuint index, /* index of the generic vertex attribute */
    GLint size, /* the number of components per generic vertex attribute: 1, 2, 3, 4 */
    GLenum type, /* data type of each component, like GL_FLOAT */
    GLboolean normalized, /* fixed-point data values converted to floats */
    GLsizei stride, /* byte offset between consecutive generic vertex attributes */
    const GLvoid * pointer /* offset of the first component of the first generic vertex attribute */
);
```

[VertexAttribPointer](http://docs.gl/gl4/glVertexAttribPointer) is a complex API call, that can configure at least 
66 different attribute formats.
It has 3 function variants (`glVertexAttribPointer`, `glVertexAttribIPointer`, `glVertexAttribLPointer`),
varying `type` parameters per function (`GL_BYTE`, `GL_FLOAT`, etc.), int-to-float conversions 
(so-called "normalized" values),
half-float formats or formats where 4 floating point values are packed into 4 bytes, paddings between
values configured with "stride", various dependencies between these parameters, 
different capabilities in different OpenGL versions, and driver bugs.

In fact, the last "driver bugs" point makes it abundantly clear that we won't create a perfect
abstraction here. For the awesome attempt to do that, you should check out the [glium](https://github.com/glium/glium)
library, as well as post by its original author on [why he is leaving glium](https://users.rust-lang.org/t/glium-post-mortem/7063).

Instead, we will focus on building VAOs in a way that gradually improves maintainability of our code
while leaving the design as simple as possible.

## Prerequisites

We are going to expand `render_gl` submodule. We are not using 
[Rust 2018](https://rust-lang-nursery.github.io/edition-guide/2018/index.html) syntax yet,
so we need to:

- create directory `render_gl`
- move `render_gl.rs` to `render_gl/mod.rs`

This should compile.

With this done, we can move all the code into even deeper `shader` submodule, leaving `render_gl`
free to define more submodules at the top.

First, create `render_gl/shader.rs`, and move everything that was inside `render_gl/mod.rs` there.

Then, inside the `render_gl/mod.rs` we can keep the `shader` submodule private (use `mod shader`
instead of `pub mod shader`), and re-export types from `render_gl::shader` as members of `render_gl`:

(render_gl/mod.rs, full file)

```rust
mod shader;

pub use self::shader::{Shader, Program, Error};
```

This should continue compiling. If it was a bit confusing, you may always check out the final
code (link should be at the end of this post).

## A new type for a Vertex Attribute

Currently, our vertex data is in a continuous `f32` array:

(existing code, main.rs)

```rust
// set up vertex buffer object

let vertices: Vec<f32> = vec![
    // positions      // colors
    0.5, -0.5, 0.0,   1.0, 0.0, 0.0,   // bottom right
    -0.5, -0.5, 0.0,  0.0, 1.0, 0.0,   // bottom left
    0.0,  0.5, 0.0,   0.0, 0.0, 1.0    // top
];
```

This is suboptimal: rememeber, every line here contains the data that is received by
the vertex shader. The data should not be limited to floats.

Instead, we can create a new type for each possible vertex attribute.
Here, we have two attributes per vertex: position and color, both are using `vec3` floating
point vector. We will start by creating a type for this vector in the `render_gl` module:

(render_gl/mod.rs, new line at the top)

```rust
pub mod data;

// the rest of the exsiting file
```

(render_gl/data.rs, new file)

```rust
#[allow(non_camel_case_types)]
#[repr(C, packed)]
pub struct f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
}

// continue here
```

We allow `non_camel_case_types`, because this name does not follow Rust conventions. Of course,
you may pick another name, but I like the readability of this one.

The `repr(C, packed)` makes rust use `C`-like layout, and `packed` makes sure `f32` components
do not contain gaps between. Otherwise Rust might align fields to 4 bytes, which is no-issue with
4-byte wide `f32`, but might be an issue for other data types we are going to create in later
lessons, like `i16_i16`; 
therefore we will use `repr(C, packed)` for everything, just to make this explicit.

This is a reason we are creating our own type, and not re-using tuple `(f32, f32, f32)` - its 
layout can change with the compiler version.

It is also a reason for not re-using `Vec3` type from
some existing math library. We need full control of the layout that must be in agreement 
with OpenGL spec. This also means that this struct is specific to OpenGL renderer; if we were
to write, say, `Vulkan` renderer, we would take care of the structs specific to it in a
context of `Vulkan`.

Let's create a convenience constructor:

(render_gl/data.rs, continued)

```rust
impl f32_f32_f32 {
    pub fn new(d0: f32, d1: f32, d2: f32) -> f32_f32_f32 {
        f32_f32_f32 {
            d0, d1, d2
        }
    }
}

// continue here
```

This allows us to initialize the data this way:

(example)

```rust
f32_f32_f32::new(0.5, -0.5, 0.0)
```

We can shrink the code a bit more if we implement conversion from tuple:

(render_gl/data.rs, continued)

```rust
impl From<(f32, f32, f32)> for f32_f32_f32 {
    fn from(other: (f32, f32, f32)) -> Self {
        f32_f32_f32::new(other.0, other.1, other.2)
    }
}
```

This allows us to write more brief initialization from tuple:

```rust
let result: f32_f32_f32 = (0.5, -0.5, 0.0).into();
```

This magic may require some explanation: the `into` method is implemented for any type
that has `From` implementation for the returned value type. Implementation of `Into` is
in the Rust standard library:

(rust standard library)

```rust
impl<T, U> Into<U> for T where U: From<T>
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

It is very useful for implementing something akin to explicit casts between types.

With that done, we can include our `render_gl::data` types in `main.rs`:

(main.rs, another `use` statement)

```rust
use render_gl::data;
```

And then we can replace our `vertices` initialization with our new type:

(main.rs, replace existing code)

```rust
// set up vertex buffer object

let vertices: Vec<data::f32_f32_f32> = vec![
    // positions      // colors
    (0.5, -0.5, 0.0).into(),   (1.0, 0.0, 0.0).into(),   // bottom right
    (-0.5, -0.5, 0.0).into(),  (0.0, 1.0, 0.0).into(),   // bottom left
    (0.0,  0.5, 0.0).into(),   (0.0, 0.0, 1.0).into()    // top
];
```

If we run it now, we would see something undefined (or nothing) on screen.
It's because `gl.BufferData` length in bytes was defined as this:

(existing code)

```rust
vertices.len() * std::mem::size_of::<f32>()
```

Out element in the vector is not longer a `f32`, it's `data::f32_f32_f32`.
We need to change the code to match it:

```rust
vertices.len() * std::mem::size_of::<data::f32_f32_f32>()
```

Let's modify `gl.BufferData` call:

(main.rs, replace `gl.BufferData` call)

```rust
gl.BufferData(
    gl::ARRAY_BUFFER, // target
    (vertices.len() * std::mem::size_of::<data::f32_f32_f32>()) as gl::types::GLsizeiptr, // size of data in bytes
    vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
    gl::STATIC_DRAW, // usage
);
```

Let's run it, the triangle should be back on screen!

## A new type for a Vertex

To move further, we will create a new vertex type for our vertex data, which
will group color and position into a single struct:

(main.rs, above `fn main()`)

```rust
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32,
}
```

We define this vertex inside main, because we will customize it later for whatever
shader we will be writing. Right now we named struct `Vertex`, because there are no other types
of vertices, but we can imagine types like `MetalShaderVertex` or `ParticleSystemVertex`.

And finally, let's modify `vertices` initialization:

(main.rs, replace code)

```rust
// set up vertex buffer object

let vertices: Vec<Vertex> = vec![
    Vertex { pos: (0.5, -0.5, 0.0).into(),  clr: (1.0, 0.0, 0.0).into() }, // bottom right
    Vertex { pos: (-0.5, -0.5, 0.0).into(), clr: (0.0, 1.0, 0.0).into() }, // bottom left
    Vertex { pos: (0.0,  0.5, 0.0).into(),  clr: (0.0, 0.0, 1.0).into() }  // top
];
```

Some verbosity here is not an issue. In a real application, we would rarely load this data by hand:
usually we would import it from a mesh file (exported from some 3D modeling program) or auto-generate
it procedurally.

Let's not forget to fix `gl.BufferData` call (use `size_of::<Vertex>`):

(main.rs, replace code)

```rust
gl.BufferData(
    gl::ARRAY_BUFFER, // target
    (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
    vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
    gl::STATIC_DRAW, // usage
);
```

The code should compile and we should be again greeted by our humble triangle.

## Nicer vertex attribute pointers Setup

Currently, the code that sets up vertex attribute pointer with `gl.EnableVertexAttribArray`
and `gl.VertexAttribPointer` would be very error-prone to maintain. Using our new
`Vertex` as a starting point, we are going to gradually refactor this set up code...

(main.rs, snippet)

```rust
// replace this code

gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
gl.VertexAttribPointer(
    0, // index of the generic vertex attribute ("layout (location = 0)")
    3, // the number of components per generic vertex attribute
    gl::FLOAT, // data type
    gl::FALSE, // normalized (int-to-float conversion)
    (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
    std::ptr::null() // offset of the first component
);
gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
gl.VertexAttribPointer(
    1, // index of the generic vertex attribute ("layout (location = 0)")
    3, // the number of components per generic vertex attribute
    gl::FLOAT, // data type
    gl::FALSE, // normalized (int-to-float conversion)
    (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
    (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
);
```

...into a call on the `Vertex`:

(main.rs, replace previous snippet)

```rust
// replace previous code with this

Vertex::vertex_attrib_pointers(&gl);
```

Let's create this function on `Vertex` type (this function does not have `self` parameter, 
so it is similar to a static method in other languages):

(main.rs, bellow `struct Vertex { ... }`)

```rust
impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        unsafe {
            gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl.VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null() // offset of the first component
            );
            gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
            gl.VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
            );
        }
    }
}
```

(The above should compile)

Here, we will continue to move two very similar parts into an implementation
on `data::f32_f32_f32` type. But first, let's refactor `vertex_attrib_pointers` code a bit
so that it would be easy to see the duplicate code:

(main.rs, rewritten `Vertex::vertex_attrib_pointers` implementation)

```rust
impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

        let location = 0; // layout (location = 0)
        let offset = 0; // offset of the first component

        unsafe {
            gl.EnableVertexAttribArray(location);
            gl.VertexAttribPointer(
                location,
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                stride as gl::types::GLint,
                offset as *const gl::types::GLvoid
            );
        }

        let location = 1; // layout (location = 1)
        let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

        unsafe {
            gl.EnableVertexAttribArray(location);
            gl.VertexAttribPointer(
                location,
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                stride as gl::types::GLint,
                offset as *const gl::types::GLvoid
            );
        }
    }
}
```

(The above should compile)

The magic numbers got replaced by the actual sizes of `Vertex` and its components:

- `6 * std::mem::size_of::<f32>()` got replaced by `std::mem::size_of::<Self>()` (where `Self` refers to "this" `Vertex` type);
- `3 * std::mem::size_of::<f32>()` got replaced by `std::mem::size_of::<data::f32_f32_f32>()`.

Most importantly, the code in `unsafe` blocks is exactly the same, and can be now
moved to a function on `data::f32_f32_f32` type:

(render_gl/data.rs, inside `impl f32_f32_f32` block)

```rust
pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
    gl.EnableVertexAttribArray(location as gl::types::GLuint);
    gl.VertexAttribPointer(
        location as gl::types::GLuint,
        3, // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        stride as gl::types::GLint,
        offset as *const gl::types::GLvoid
    );
}
```

(don't forget small `use gl;` at the top of `render_gl/data.rs`)

And the `Vertex::vertex_attrib_pointers` can be futher simplified to:

(main.rs, replaced `vertex_attrib_pointer` code)

```rust
impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

        let location = 0; // layout (location = 0)
        let offset = 0; // offset of the first component

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }

        let location = 1; // layout (location = 1)
        let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }
    }
}
```

(The above should compile)

I've chosen to make `f32_f32_f32::vertex_attrib_pointer` function unsafe while
leaving parent `Vertex::vertex_attrib_pointers` "safe". My reasoning is twofold. First,
it may be easier to screw something up while using `f32_f32_f32::vertex_attrib_pointer` directly
(say, argument order) than `Vertex::vertex_attrib_pointers`. Second, `data::f32_f32_f32`
is a reusable type inside a lower level library, where we want to warn "future us" about
the dangers of it, while `Vertex` is a type that is specific to whatever experimental
shader we are building, and we may favor more convenience while experimenting. We may get back to this
topic in future if it starts causing issues.

This was a small change, but it added a lot of convenience. If you squint a little,
you may even start to wonder if this `impl Vertex` could be auto-generated.

<!--
Next time, we will learn about procedural macros, create our own `#[derive(VertexAttributePointers)]`
macro, and auto-generate `vertex_attrib_pointers` code.
-->

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-09).