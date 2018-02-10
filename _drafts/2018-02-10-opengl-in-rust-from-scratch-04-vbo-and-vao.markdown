---
layout: post
title:  "Rust and OpenGL from scratch - VBO and VAO"
date:   2018-02-10
categories: rust opengl tutorial
---

Welcome back!

## Sending triangle data to graphics driver

Before running the `'main` loop, initialize vector with vertices for our triangle:

(main.rs, incomplete)

```rust
let vertices: Vec<f32> = vec![
    -0.5, -0.5, 0.0,
    0.5, -0.5, 0.0,
    0.0, 0.5, 0.0
];

// continue here
```

Then, request OpenGL to give us one buffer name (as integer), and write it into
`vbo` variable:

(main.rs, continued)

```rust
let mut vbo: gl::types::GLuint = 0;
unsafe {
    gl::GenBuffers(1, &mut vbo);
}

// continue here
```

For `GenBuffers`, we have to provide a pointer to array which it will overwrite with a
new value. Rust references (`&mut` and `&`) __are__ pointers, so we can simply pass them along.
We must, of course, limit the number of buffers to `1` so it does not overwrite unknown memory nearby.
That would be bad. This `unsafe` block means something.

Then, we can upload data to buffer by binding it and calling `gl::BufferData`, very much like in C:

(main.rs, continued)

```rust
unsafe {
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER, // target
        (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
        vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
        gl::STATIC_DRAW, // usage
    )
}
```

Let's go over `BufferData` arguments:

- Target specifies buffer type, pretty much like in C
- We obtain size of data in bytes by taking size of `f32` (`std::mem::size_of::<f32>()`), multiplying it
by number of items in vector (`vertices.len()`), and then casting it to whatever integer is 
`gl::types::GLsizeiptr`, because Rust requires explicit integer casts.
- We get pointer void pointer to data by using `.as_ptr()` function, but it would return us
correct `*const f32` pointer, while OpenGL needs `*const GLvoid`. We perform explicit cast here.
- We set `gl::STATIC_DRAW` usage hint, pretty much like in C

I suggest [docs.gl](docs.gl) site for OpenGL documentation. For example, 
[glBufferData function page contains comprehensive documentation with examples and information about supported OpenGL versions](http://docs.gl/gl4/glBufferData). 
Very useful.