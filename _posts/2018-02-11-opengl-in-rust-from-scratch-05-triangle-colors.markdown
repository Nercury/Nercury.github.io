---
layout: post
title:  "Rust and OpenGL from scratch - Colored Triangle"
date:   2018-02-11
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/11/opengl-in-rust-from-scratch-04-triangle.html), 
we have filled the VBO, configured VAO, and painted triangle in a window.

This time, we will add different color for every triangle corner.

To avoid repeating the same things other OpenGL tutorials already teach, we are loosely following 
[learnopengl.com](https://learnopengl.com) lessons, and we are now at [Hello Triangle page](https://learnopengl.com/Getting-started/Hello-Triangle).

## More of the same

We can extend our data to include red, green and blue for each color, respectively:

(main.rs, modified previous code)

```rust
let vertices: Vec<f32> = vec![
    // positions      // colors
    0.5, -0.5, 0.0,   1.0, 0.0, 0.0,   // bottom right
    -0.5, -0.5, 0.0,  0.0, 1.0, 0.0,   // bottom left
    0.0,  0.5, 0.0,   0.0, 0.0, 1.0    // top
];
```

Then we change the stride in data layout from 3 to 6, because we've added
3 additional floating point value for each vertex:

(main.rs, modified previous code)

```rust
    gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
    gl::VertexAttribPointer(
        0, // index of the generic vertex attribute ("layout (location = 0)")
        3, // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        std::ptr::null() // offset of the first component
    );
    
    // continue here
```

We will configure the vertex shader to pick up this color information at location 1:

(continued)

```rust
    gl::EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
    gl::VertexAttribPointer(
        1, // index of the generic vertex attribute ("layout (location = 0)")
        3, // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
    );
```

Let's modify vertex shader to pick up the color as `vec3`:

```glsl
#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Color;

out VS_OUTPUT {
    vec3 Color;
} OUT;

void main()
{
    gl_Position = vec4(Position, 1.0);
    OUT.Color = Color;
}
```

And then the fragment shader:

```glsl
#version 330 core

in VS_OUTPUT {
    vec3 Color;
} IN;

out vec4 Color;

void main()
{
    Color = vec4(IN.Color, 1.0f);
}
```

If you follow `learnopengl.com`, you may notice that we changed up a bit the way 
we pass color from vertex to fragment shader. The `VS_OUTPUT` struct groups output and
input variables nicely, and we don't need to add suffixes for our values such as `aColor`.

## Drawing

Drawing code remains the same, and we get the expected result:

![Triangle](/images/opengl-rust/05/triangle.jpg)

## Discussion

While is is quite possible and educational to use OpenGL this way,
it is incredibly error-prone. If we were to forget to modify `stride` for the 
VAO configured in previous lesson, we would get some garbage on screen,
and would have to spend some time searching for bug and double-checking
everything.

In the next lessons, we will begin building some abstractions to encapsulate
error-prone things related to VAO setup.

There are additional topics discussed in [learnopengl.com](https://learnopengl.com)
lesson that we are going to skip for now, such as passing uniforms to shader
or Element Buffer Objects. It is a bit tedious, but not that hard to rewrite C code
to Rust, and I feel that we have discussed enough examples of that. I am going
to leave these things as "exercises to the reader".

As always, [the code for this part is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-05).
