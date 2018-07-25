---
layout: post
title:  "Rust and OpenGL from scratch - Buffers"
date:   2018-07-25
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/07/12/opengl-in-rust-from-scratch-11-vertex-data-types.html), we have finished abstracting OpenGL vertex data types.

This time, we will create thin wrappers around VBOs and VAOs.

Let's review what we had previously. We stored the VBO in a simple value:

```rust
let mut vbo: gl::types::GLuint = 0;
```

We created new GL buffer object with `glGenBuffers` call, for the
`vbo` to receive a new handle to it:

```rust
unsafe {
    gl.GenBuffers(1, &mut vbo);
}
```

We then uploaded data from `vertices` to the VBO:

```rust
unsafe {
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl.BufferData(
        gl::ARRAY_BUFFER, // target
        (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data in bytes
        vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
        gl::STATIC_DRAW, // usage
    );
    gl.BindBuffer(gl::ARRAY_BUFFER, 0);
}
```

Later, we linked this VBO to VAO:

```rust
gl.BindVertexArray(vao);
gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
```

.. So that we could use `VAO` to draw a triangle (which would pick up the correct VBO from its state):

```rust
gl.BindVertexArray(vao);
gl.DrawArrays(
    gl::TRIANGLES, // mode
    0, // starting index in the enabled arrays
    3 // number of indices to be rendered
);
```

One thing we were not doing: we were not cleaning up the VBO
at the end of the program:

```rust
unsafe {
    gl.DeleteBuffers(1, &mut vbo);
}
```

## ArrayBuffer Wrapper

A simple struct with a constructor `new` and a `drop` implementation will ensure
a single creation and a single drop:

(src/render_gl/mod.rs)

```rust
pub mod buffer;
```

(src/render_gl/buffer.rs)

```rust
use gl;

pub struct ArrayBuffer {
    vbo: gl::types::GLuint,
}

impl ArrayBuffer {
    pub fn new(gl: &gl::Gl) -> ArrayBuffer {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        ArrayBuffer {
            vbo
        }
    }
}

impl Drop for ArrayBuffer {
    fn drop(&mut self) {
        unsafe {
            gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}
```

Which would be nice and good, except we can't access the `Gl` in `drop`.
We can reuse the same solution we did [back in shader program wrapper](/rust/opengl/tutorial/2018/02/12/opengl-in-rust-from-scratch-06-gl-generator.html)
where we stored a reference-counted `Gl` inside the shader program struct.

(src/render_gl/buffer.rs)

```rust
use gl;

pub struct ArrayBuffer {
    gl: gl::Gl,
    vbo: gl::types::GLuint,
}

impl ArrayBuffer {
    pub fn new(gl: &gl::Gl) -> ArrayBuffer {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        ArrayBuffer {
            gl: gl.clone(),
            vbo
        }
    }
}

impl Drop for ArrayBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}
```

Yep, because of this additional `gl` field, this wrapper steps a bit 
outside of zero-cost claim. It may also be a problem
if we tried to create millions of small array buffers (which may also be _another_, bigger problem).
Simplest way I can think
of to remedy this would be to avoid all the lies and make another struct 
named `ArrayBuffers`, plural, which would _always_ generate multiple
buffers, but store one reference to `Gl` for all of them, and match OpenGL API 1:1:

```rust
pub struct ArrayBuffers {
    gl: gl::Gl,
    vbo: Vec<gl::types::GLuint>,
}
```

We are not going to follow this route though - for the sake of keeping the API
as simple as possible (but it would not be very different).

Back to the `ArrayBuffer` then. All that's left is adding `bind` and `unbind` methods:

(src/render_gl/buffer.rs, added)

```rust
impl ArrayBuffer {
    ...

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
    
    // continue here
}
```

.. As well as a method to upload the data, that works with any slice or Vec:

```rust
    pub fn static_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                gl::ARRAY_BUFFER, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
    }
```

### Using the Array Buffer

Now we can go back to `main.rs` and replace VBO initialization with the new one:

(src/main.rs, modified)

```rust
use render_gl::buffer;

...


fn run() -> Result<(), failure::Error> {
    ...
    
    let vbo = buffer::ArrayBuffer::new(&gl);
    vbo.bind();
    vbo.static_draw_data(&vertices);
    vbo.unbind();
    
    ...
}
```

And also use it in the VAO code:

(src/main.rs, modified)

```rust
    ...
    
    unsafe {
        gl.BindVertexArray(vao);
        vbo.bind(); // here
    
        Vertex::vertex_attrib_pointers(&gl);
    
        vbo.unbind(); // here
        gl.BindVertexArray(0);
    }
    
    ...
```

This should run.

### Discussion about Buffer

#### The `static_draw_data` method could have called `bind` and `unbind`.

Yes, but let's make the API we expose somewhat consistent: if we expose separate low level
calls such as `bind`, let's not hide secret calls to `bind` in any other methods.

#### The VBO is immutable, but we upload data to it!

In Rust, mutable/immutable is not about some ideal concept of mutability, it 
is about memory safety guarantees. If we are using OpenGL drivers, then all bets are off.
So there. Of course, we do try, but in this case the benefits are not very clear. But we may 
consider `&mut self` references in cases where they helped to
enforce certain API usage.

## Vertex Array Object Wrapper

Let's follow a very similar approach and build VAO wrapper:

(src/render_gl/buffer.rs, added code)

```rust
pub struct VertexArray {
    gl: gl::Gl,
    vao: gl::types::GLuint,
}

impl VertexArray {
    pub fn new(gl: &gl::Gl) -> VertexArray {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
        }

        VertexArray {
            gl: gl.clone(),
            vao
        }
    }
    
    // continue here
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
```

(src/render_gl/buffer.rs, continued)

.. And, like for the VBO, add bind/unbind methods:

```rust
    pub fn bind(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindVertexArray(0);
        }
    }
```

.. And replace the VAO initialization and uses of `bind` in the `main.rs`:

(src/main.rs, changed `run` function)

```rust
...
    // set up vertex array object

    let vao = buffer::VertexArray::new(&gl);  // changed

    vao.bind();                               // changed
    vbo.bind();                               // changed
    Vertex::vertex_attrib_pointers(&gl);      // changed
    vbo.unbind();                             // changed
    vao.unbind();                             // changed
...

    // draw triangle

    shader_program.set_used();                
    vao.bind();                               // changed
    unsafe {
        gl.DrawArrays(
            gl::TRIANGLES, // mode
            0, // starting index in the enabled arrays
            3 // number of indices to be rendered
        );
    }
```

As expected, this should compile and run.

### Discussion

We have removed a lot of `unsafe` code!

#### Again, we might have created a helper for `vertex_attrib_pointers` that calls all the necessary `bind`s...

Yes, but that again would hide some details. And maybe sometimes we would like to avoid
a call or two to `bind` or `unbind`. Let's leave this as explicit as possible
for now.

#### But now the order of which is destroyed - VAO or VBO, depends on the order the values were declared!

Indeed, this might be an issue. For example, we know that VAO holds a reference to
buffer, so we could design an API that reflects that. However, to be able to use
the same buffer for multiple VAOs, we would have to make buffer reference-counted.

Thing is, OpenGL drivers are already counting the references. A call to `glDeleteBuffers`
does not mean immediate deallocation, at least not in this case. If a buffer is referenced
by VAO, it lives until the VAO is destroyed. It feels kind of wasteful to do this
reference-counting twice, so we might just get away without it.

## Element Array Buffer Wrapper

`gl::ELEMENT_ARRAY_BUFFER` works exactly like `gl::ARRAY_BUFFER`,
so to create a rusty wrapper for it we could just copy-paste 
`ArrayBuffer` struct into a new `ElementArrayBuffer` struct.

But I think it's time for a bit of heavier use of generics and traits, 
and this seems like a good
place to start (and a bit non-standard, as we will see shortly). 
We could rename `ArrayBuffer` to generic `Buffer<BufferType>`, where
`BufferType` would hold either `gl::ARRAY_BUFFER` or `gl::ELEMENT_ARRAY_BUFFER` (and 
other buffer types).

We will face two issues:

- Rust does not yet have support for [generics over `int` values][const-generics] (which are very common in C++);
- In Rust, generic parameters have to be used.

But both of these issues have decent solutions that don't impose additional runtime
cost. Let's begin.

### Generic Buffer

Let's rename `render_gl::ArrayBuffer` to `render_gl::Buffer`.

Then, let's make `render_gl::Buffer` generic. As mentioned, const generics are
not yet in the language. However, our type can be generic over another type that
implements a trait. And this trait can have a constant `BUFFER_TYPE` value:

(src/render_gl/buffer.rs, add code)

```rust
pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}
```

Then, when making `Buffer<B>` generic over the `B` type, we can specify
this trait as _trait bound_ for the `B` with `where B: BufferType` syntax:

(src/render_gl/buffer.rs, replace struct ArrayBuffer with impl)

```rust
pub struct Buffer<B> where B: BufferType {
    gl: gl::Gl,
    vbo: gl::types::GLuint,
}

impl<B> Buffer<B> where B: BufferType {
    pub fn new(gl: &gl::Gl) -> Buffer<B> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            gl: gl.clone(),
            vbo,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }

    pub fn static_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                gl::ARRAY_BUFFER, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
    }
}

impl<B> Drop for Buffer<B> where B: BufferType {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}
```

However, this won't compile as-is:

```plain

error[E0392]: parameter `B` is never used
  --> lesson-12\src\render_gl\buffer.rs:17:19
   |
17 | pub struct Buffer<B> where B: BufferType {
   |                   ^ unused type parameter
   |
   = help: consider removing `B` or using a marker such as `std::marker::PhantomData`
   
```

What is this `PhantomData`?

When we create structures that act as if they contain a value, but actually don't,
we have to tell Rust what we are faking. We use this built-in compiler type
`PhantomData` to do exactly that.

In this case, we are acting as if `B` is a value. In particular,
if the `Buffer` is dropped, `B` should be dropped too. In other words,
if we would add it to the struct, it would look like this:

(example)

```rust
pub struct Buffer<B> where B: BufferType {
    ...
    foo: B, // added value
}
```

To explain this to compiler, we add a `_marker` field of
the same type, but wrap it in the `PhantomData`:

(src/render_gl/buffer.rs, modified Buffer<B> struct)

```rust
pub struct Buffer<B> where B: BufferType {
    gl: gl::Gl,
    vbo: gl::types::GLuint,
    _marker: ::std::marker::PhantomData<B>,
}
```

In our use case, this does not make any difference. But in some
cases, we may need to implement internals of a struct using raw pointers without
actually holding a value, and it would matter.
You can read more on that in the [`PhantomData` documentation](https://doc.rust-lang.org/std/marker/struct.PhantomData.html).

[const-generics]: https://github.com/rust-lang/rust/issues/44580

`PhantomData` is zero-sized, so it does not take additional space in the struct.

We also need to modify `new` function to create this `PhantomData`. That's
easy, and the compiler can infer the generic `PhantomData` type:

(src/render_gl/buffer.rs, modified Buffer<B>::new function)

```rust
    pub fn new(gl: &gl::Gl) -> Buffer<B> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            gl: gl.clone(),
            vbo,
            _marker: ::std::marker::PhantomData, // added
        }
    }
```

Now we just need something to implement `BufferType` trait and return `gl::ARRAY_BUFFER`
or `gl::ELEMENT_ARRAY_BUFFER`.

For that, we can use zero-sized structs:

(src/render_gl/buffer.rs, added)

```rust
pub struct BufferTypeArray;
impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;
impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}
```

With this done, we can now initialize our buffer with this call:

(example)

```rust
Buffer::<BufferTypeArray>::new(&gl)
```

But we can do a bit better and shorten `Buffer::<BufferTypeArray>` type
by defining some type aliases:

(src/render_gl/buffer.rs, added)

```rust
pub type ArrayBuffer = Buffer<BufferTypeArray>;
pub type ElementArrayBuffer = Buffer<BufferTypeElementArray>;
```

Now, we can leave the same code in `main` as it was before
making the array generic. Additionally, we have a new `ElementArrayBuffer` 
type ready for use in the next lesson.

[Full source code, is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-12). 