---
layout: post
title:  "Rust and OpenGL from scratch - Vertex Attribute Format"
date:   2018-04-18
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

We are going to expand `render_gl` submodule. We arn't using 
[Rust 2018](https://rust-lang-nursery.github.io/edition-guide/2018/index.html) syntax yet,
so we need to:

- create directory `render_gl`
- move `render_gl.rs` to `render_gl/mod.rs`

This should compile.

With this done, we can move all the code into even deeper `shader` submodule, leaving `render_gl`
free to define more submodules at the top.

First create `render_gl/shader.rs`, and move everything that was inside `render_gl/mod.rs` there.

Then, inside the `render_gl/mod.rs` we can keep the `shader` submodule private (use `mod shader`
instead of `pub mod shader`), and re-export types from `render_gl::shader` as members of `render_gl`:

(render_gl/mod.rs, full file)

```rust
mod shader;

pub use self::shader::{Shader, Program, Error};
```

This should continue compiling. If it was a bit confusing, you may always check out the final
code (link should be at the end of this post).

## A new type for a vertex attribute

Currently, our vertex data is in continuous `f32` array:

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
vertex shader. The data should not be limited to floats.

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
4-byte wide `f32`, but might be an issue for other data types we are going to create, like `i16_i16`; 
therefore we will use `repr(C, packed)` for everything, just to make this explicit.

This is the reason we are creating our own type, and not re-using tuple `(f32, f32, f32)` - its 
layout can change with the compiler version.

This is also the reason for not re-using `Vec3` type from
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
that has `From` implementation for the returned value type. This implementation is
in Rust standard library:

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

We need to change it to

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

## A new type for a vertex

To move further, we will create a new vertex type for our vertex data, which
will contain color and position:

(main.rs, above `fn main()`)

```rust
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32,
}
```

We define this vertex inside main, because it should be customized for whatever
shader we are writing. Right now we named it `Vertex`, because there are no other types
of vertices, but we can imagine types like `MetalShaderVertex` or `ParticleSystemVertex`.

And then, let's again modify `vertices` initialization:

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

The code should compile and we should be still greeted by the triangle.