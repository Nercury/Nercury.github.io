---
layout: post
title:  "Rust and OpenGL from scratch - Vertex Data Types"
date:   2018-07-12
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/07/11/opengl-in-rust-from-scratch-10-procedural-macros.html), 
we have created a procedural macro, which, given this `Vertex` definition:

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::f32_f32_f32,
}
```

...auto generates `vertex_attrib_pointers` function for `Vertex`, which calls 
this `vertex_attrib_pointer` method for each `Vertex` field with correct arguments:

(example, some code omitted)

```rust
#[repr(C, packed)]
pub struct f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
}

impl f32_f32_f32 {
    pub unsafe fn vertex_attrib_pointer(
        gl: &gl::Gl, stride: usize, location: usize, offset: usize
    ) {
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
}
```

But so far we have only had one data type: `f32_f32_f32`.

In this post, we will walk over implementing the remaining OpenGL data types.

However, most of it is _very_ boring, therefore we will start by looking at more
exotic data type, which in OpenGL is named `GL_UNSIGNED_INT_2_10_10_10_REV`.

## GL_UNSIGNED_INT_2_10_10_10_REV

This is 32-bit 4-dimensional vector, where the first dimension has 2 bits, 
and the last 3 dimensions have 10 bits each.

It is useful for representing a color with an alpha, where the alpha does not 
require much precision. In this case, 2 bits can be used for alpha (for a
total of 4 values), and the remaining bits for 3 colors.

At the first glance, this might look very inconvenient. Let's say we want this
to represent the color with alpha, as mentioned. If a shader received a 4-integer
vector with varying-width integers, it would require additional fiddling to
normalize these values back to floats between 0 and 1.

Luckily, the 4-th parameter to `glVertexAttribPointer` does exactly that - converts
these integers to normalized floating point values, between 0 and 1. We can
then use `vec4` in the shader code, provided we set normalized to `gl::TRUE`.

All we need to do is pack 4 values into this format.
To test it out, we will use this format for colors of our triangle,
bringing down the used bytes for vertex from 24 to 16, making it
a _very optimized hello-world triangle_.

I bet no other tutorial has delayed the spinning cube for so long as this one ;)

## Wrapper type for `GL_UNSIGNED_INT_2_10_10_10_REV`

Spoiler: there is a crate for it!

The crate is named `vec-2-10-10-10`, and [can be found here](https://crates.io/crates/vec-2-10-10-10).

We will overview how it works, to get familiar with an API.

At the core of it, the `Vector` struct wraps a simple 32-bit integer:

(code from vec-2-10-10-10 crate)

```rust
#[repr(C, packed)]
pub struct Vector {
    data: u32,
}
```

It makes sense to store all vector components as a single integer, because the bits
will need to be shifted to the right places anyways.

For example, let's say that values `w` (2 bits), `x`, `y` and `z` (10 bits) all
contain the right bits:

- `w` [00000000 00000000 00000000 000000<span style="color: #7d0dd8; font-weight: bold">10</span>]
- `z` [00000000 00000000 000000<span style="color: #0d2bd8; font-weight: bold">11 11111111</span>]
- `y` [00000000 00000000 000000<span style="color: #115e21; font-weight: bold">01 01010101</span>]
- `x` [00000000 00000000 000000<span style="color: #8e0a05; font-weight: bold">00 11001100</span>]

To construct the final value, we need to shift `w` 30 bits to left,
`z` 20 bits to left, `y` 10 bits to left, leave `x` in the same place, and
combine them together (with OR):

```rust
let mut c: u32 = 0;
c |= w << 30;
c |= z << 20;
c |= y << 10;
c |= x << 0;

Vector {
    data: c
}
```

- `data` [<span style="color: #7d0dd8; font-weight: bold">10</span><span style="color: #0d2bd8; font-weight: bold">111111 1111</span><span style="color: #115e21; font-weight: bold">0101 010101</span><span style="color: #8e0a05; font-weight: bold">00 11001100</span>]

To convert floating-point `x`, `y`, `z` and `w` from 0 to 1 to the initial integers,
we can multiply them to the respective max int value, round the results,
and then cast them to integer types:

```rust
// example values
let x: f32 = 0.3;
let y: f32 = 0.6;
let z: f32 = 0.0;
let w: f32 = 1.0;

let x = (clamp(x) * 1023f32).round() as u32; // 2^10 = 1024
let y = (clamp(y) * 1023f32).round() as u32;
let z = (clamp(z) * 1023f32).round() as u32;
let w = (clamp(w) * 3f32).round() as u32; // 2^2 = 4
```

So, these two actions are combined in `Vector` constructor:

```rust
pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector {
    let x = (clamp(x) * 1023f32).round() as u32;
    let y = (clamp(y) * 1023f32).round() as u32;
    let z = (clamp(z) * 1023f32).round() as u32;
    let w = (clamp(w) * 3f32).round() as u32;

    let mut c: u32 = 0;
    c |= w << 30;
    c |= z << 20;
    c |= y << 10;
    c |= x << 0;

    Vector {
        data: c
    }
}
```

The `Vector` also have methods to retrieve the values, as well as change them
individually while leaving other bits intact. [Full API doc is here](https://docs.rs/vec-2-10-10-10/0.1.2/vec_2_10_10_10/struct.Vector.html).

What is important to us, that this vector should be directly usable to render
our triangle colors. However, it does not implement `vertex_attrib_pointer` method,
and it really shouldn't - it's not a concern of this small library.

But we are free to create a new zero-cost wrapper type, sometimes also called
a _newtype_, to wrap the `vec_2_10_10_10::Vector` functionality, and also
implement the `vertex_attrib_pointer` method that we need.

## The `u2_u10_u10_u10_rev_float` Wrapper

Inside `Cargo.toml`, we can reference `vec_2_10_10_10` crate:

(Cargo.toml, added dependency)

```toml
[dependencies]
...
vec-2-10-10-10 = "0.1.2"
```

As well as the crate reference:

(src/main.rs)

```rust
...
extern crate vec_2_10_10_10;
...
```

Cargo has this interesting convention where `-` and `_` in the crate names are interchangeable,
but when adding extern crate we need to use `_`.

Inside `render_gl::data`, we can now create a wrapper for `u2_u10_u10_u10_rev_float`.

I am suffixing the type with `_float`, because there might be another type which _does
not_ normalize the integers to floats when passing them to the shader:

(src/render_gl/data.rs, added new struct)

```rust
#[allow(non_camel_case_types)]
#[repr(C, packed)]
pub struct u2_u10_u10_u10_rev_float {
    pub inner: ::vec_2_10_10_10::Vector,
}

impl From<(f32, f32, f32, f32)> for u2_u10_u10_u10_rev_float {
    fn from(other: (f32, f32, f32, f32)) -> Self {
        u2_u10_u10_u10_rev_float {
            inner: ::vec_2_10_10_10::Vector::new(other.0, other.1, other.2, other.3)
        }
    }
}
```

We are not going to re-implement all the same constructor with the setters and
getters; instead, we will simply expose the `inner` field as public.

But the helper `From<(f32, f32, f32, f32)>` is nice to have, to be able to 
convert `(f32, f32, f32, f32)` to `u2_u10_u10_u10_rev_float` with `.into()` call.

Now, all that's left is to add `vertex_attrib_pointer` method with correct
arguments to `glVertexAttribPointer`:

(src/render_gl/data.rs, added impl)

```rust
impl u2_u10_u10_u10_rev_float {
    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::UNSIGNED_INT_2_10_10_10_REV, // data type
            gl::TRUE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}
```

## Using `u2_u10_u10_u10_rev_float` instead of `f32_f32_f32` for Triangle Color

Inside `main.rs`, let's modify `Vertex` type to use this new type:

(src/main.rs, modified code snippet)

```rust
#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::u2_u10_u10_u10_rev_float, // changed
}
```

And we need to change `vertices` initialization:

(src/main.rs, modified code snippet)

```rust
let vertices: Vec<Vertex> = vec![
    Vertex {
        pos: (0.5, -0.5, 0.0).into(),
        clr: (1.0, 0.0, 0.0, 1.0).into()
    }, // bottom right
    Vertex {
        pos: (-0.5, -0.5, 0.0).into(),
        clr: (0.0, 1.0, 0.0, 1.0).into()
    }, // bottom left
    Vertex { 
        pos: (0.0,  0.5, 0.0).into(),
        clr: (0.0, 0.0, 1.0, 1.0).into()
    }  // top
];
```

This vector now has 4 components, the last one used for alpha. We can set it to `1.0`.

We also need to modify the shader: it is expecting `vec3`, but now we are passing `vec4`:

(shaders/triangle.vert, full code)

```glsl
#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color; // changed

out VS_OUTPUT {
    vec3 Color;
} OUT;

void main()
{
    gl_Position = vec4(Position, 1.0);
    OUT.Color = Color.xyz; // changed, ignore w component
}
```

Let's compile it and run. Magic! It's like nothing ever happened! Try
modifying the triangle colors and check that yes, this is actually working!

## The rest of the Data Types

Let's start from the simplest data type, `i8`. It's a newtype for built-in `i8`
type, but I had to name it `u8_`, to avoid name clash. It has `vertex_attrib_pointer` implemented. 
Here is the implementation:

(src/render_gl/data.rs, added code)

```rust
#[allow(non_camel_case_types)]
#[repr(C, packed)]
pub struct i8_ {
    pub d0: i8,
}

impl i8_ {
    pub fn new(d0: i8) -> i8_ {
        i8_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<i8> for i8_ {
    fn from(other: i8) -> Self {
        i8_::new(other)
    }
}
```

Here, I am using VertexAttrib<span style="font-weight: bold">I</span>Pointer.

One more example, for `i8_float` type. Here we are continuing our convention to suffix
integer types that are normalized to floats with `_float`. Otherwise everything else is very
similar:

(src/render_gl/data.rs, added code)

```rust
#[allow(non_camel_case_types)]
#[repr(C, packed)]
pub struct i8_float {
    pub d0: i8,
}

impl i8_float {
    pub fn new(d0: i8) -> i8_float {
        i8_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<i8> for i8_float {
    /// Create this data type from i8
    fn from(other: i8) -> Self {
        i8_float::new(other)
    }
}
```

To avoid getting bored to death, we won't continue this here in the blog post.

One approach would be to add these implementations as-needed, to make sure every new added
implementation is tested. 

However, I fear that every time I will need to add something new, I will have to
read OpenGL docs again. Therefore I went ahead and added all the possible OpenGL
data types that I could think of.

In the process, I've also found a [`half` crate](https://crates.io/crates/half), for
half-precision floating point value `f16` (software implementation). Some of the
exotic types, such as `i2_i10_i10_i10_rev`, `u10_u11_u11_rev`, `i2_i10_i10_i10_rev_float` and
`u10_u11_u11_rev_float` will only have stub implementations with no nice inner type
(I've left `u32` there and a TODO comment).

Why we have avoided using macros? Well, they don't work well for API generation - we
would loose most of autocomplete suggestions, because our tools are not there yet.

[Full source code, as always, is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-11).

[Next time](/rust/opengl/tutorial/2018/07/25/opengl-in-rust-from-scratch-12-buffers.html), we will create abstractions for VAO and VBO.