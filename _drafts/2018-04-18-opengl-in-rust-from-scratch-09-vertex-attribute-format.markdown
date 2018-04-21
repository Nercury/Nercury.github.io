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

Instead, our goal here is to learn and explore, while creating simple utilities that we can understand
and adapt to whatever unknown driver issue we will encounter in the wild, instead of trying to create
the api wrapper that solves everyone's problems.

With this philosophy in mind, we are still going to look at a possible abstraction that is very similar
to what the glium did (although a bit simplified). Then, we will explore another, simpler (but maybe
not that powerful) approach. Then, we will discuss the pros and cons of both.

## Layout-struct approach

What if we create a struct `Layout` that can be used to add format components and then
initialize VAO:

```rust
let layout = Layout::new()
    // Position attribute
    .with(0, Format::f32_f32_f32, Padding::p0)
    // Color attribute
    .with(1, Format::f32_f32_f32, Padding::p0);
layout.attrib_pointers(gl);
```