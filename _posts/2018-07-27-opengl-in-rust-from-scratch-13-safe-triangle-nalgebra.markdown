---
layout: post
title:  "Rust and OpenGL from scratch - Safe Triangle and nalgebra"
date:   2018-07-27
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/07/25/opengl-in-rust-from-scratch-12-buffers.html),
we have create safe (ish) wrappers for VBO and VAO.

This time, we will continue along this path and remove the rest of unsafe code from our
`run` function.

## A structure to hold data for Triangle

The data that is necessary for Triangle is the shader program, VAO and VBO.

We can easily group them together into `Triangle` struct, and move this struct to
another file.

(src/main.rs, added module reference)

```rust
mod triangle;
```

(src/triangle.rs, added new file)

```rust
pub struct Triangle {
    program: render_gl::Program,
    _vbo: buffer::ArrayBuffer, // _ to disable warning about not used vbo
    vao: buffer::VertexArray,
}

impl Triangle {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Triangle, failure::Error> {
        // initialization code that uses resources to
        // load data for triangle and wrap this data in
        // Triangle struct, or return an error
    }

    pub fn render(&self, gl: &gl::Gl) {
        // function that renders the triangle based on loaded data
    }
}
```

The `new` function uses the `Resources` to load the data for the triangle into
OpenGL objects, and then store handles to those objects inside the `Triangle` struct.

Loading can fail, therefore the `new` returns a `Result` type. If we wanted to care
a bit more about performance here, like, if this kind of object
would be created many times, we would return a custom error type that
would implement `Fail`. We have [discussed error performance and types back in the
`Failure` lesson](/rust/opengl/tutorial/2018/02/15/opengl-in-rust-from-scratch-08-failure.html).

The render renders, not much to add here.

The triangle also depends on the `Vertex` type, which we can copy-pase here
together with the `new` and `render` implementation:

(src/triangle.rs, full file, modified)

```rust
use gl;
use failure;
use render_gl::{self, data, buffer};
use resources::Resources;

#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::u2_u10_u10_u10_rev_float,
}

pub struct Triangle {
    program: render_gl::Program,
    _vbo: buffer::ArrayBuffer, // _ to disable warning about not used vbo
    vao: buffer::VertexArray,
}

impl Triangle {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Triangle, failure::Error> {

        // set up shader program

        let program = render_gl::Program::from_res(gl, res, "shaders/triangle")?;

        // set up vertex buffer object

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

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vbo.unbind();
        vao.unbind();

        Ok(Triangle {
            program,
            _vbo: vbo,
            vao,
        })
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.program.set_used();
        self.vao.bind();

        unsafe {
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0, // starting index in the enabled arrays
                3 // number of indices to be rendered
            );
        }
    }
}
```

In the main function, we can now create the triangle with this simple call:

```rust
let triangle = triangle::Triangle::new(&res, &gl)?;
```

And render replace rendering with this:

```rust
triangle.render(&gl);
```

This should build and run. We can also clean up unused use statements
if we get any warnings.

## Side Note

That `failure_to_string` function at the end of main does not look nice,
so we can hide it in `src/debug.rs` submodule.

## Discussion: dependencies in render loop

We have built a simple triangle wrapper, and it looks like this
will be the way we will create further simple "components" that we
will hook into initialization and rendering logic.

These components will depend on each other and some will even modify the state
of each other.

In Object-Oriented languages, it is a common pattern to inject such
components into each other at the time of initialization. However,
in Rust, if we inject one mutable reference of, say, a `triangle` somewhere,
we can't do that for another component - the code won't compile:

(example)

```rust
let mut triangle = triangle::Triangle::new(&res, &gl)?;
let config = Config::new(&mut triangle)?;
let controller = Controller::new(&mut triangle)?; // triangle is already borrowed as mutable
```

Here we assume that hypothetical `Config` and `Controller` keeps
the mutable pointer to `triangle` inside:

(example)

```rust
struct Config<'a> {
    triangle: &'a mut Triangle,
}

struct Controller<'a> {
    triangle: &'a mut Triangle,
}
```

One solution would be to put triangle behind a reference-counted pointer:

(example)

```rust
let triangle = Rc::new(triangle::Triangle::new(&res, &gl)?);
let config = Config::new(triangle.clone())?;
let controller = Controller::new(triangle.clone())?;

...

// somewhere else

struct Config {
    triangle: Rc<Triangle>,
}

struct Controller {
    triangle: Rc<Triangle>,
}
```

This way, the triangle is shared, we get rid of lifetimes, but the triangle is
still immutable! To make it mutable, we would have to use what is called
_interior mutability_: additionally wrap the triangle in a `RefCell` that makes
borrowing checks dynamically at runtime instead of compile time:

(example)

```rust
let triangle = Rc::new(RefCell::new(triangle::Triangle::new(&res, &gl)?));
let config = Config::new(triangle.clone())?;
let controller = Controller::new(triangle.clone())?;

...

// somewhere else

struct Config {
    triangle: Rc<RefCell<Triangle>>,
}

struct Controller {
    triangle: Rc<RefCell<Triangle>>,
}
```

If we ignore the general uglyness for a moment, this would work.
Let's say somewhere bellow in `run` function we use both `Config` and `Controller` this
way:

(example)

```rust
config.save_triangle_configuration();
..
controller.shake_triangle();
```

Inside of these functions, there would be some code that mutably borrows the triangle
to save and shake it:

(example)

```rust
impl Config {
    fn save_triangle_configuration(&self) {
        self.triangle.borrow_mut().save();
    }
}

impl Controller {
    fn shake_triangle(&self) {
        self.triangle.borrow_mut().harlem();
    }
}
```

It would work, but it is a bit annoying to have even this overhead in a
tight rendering loop. Not to mention it looks ugly!

The insight here would be that if the triangle is only used in these functions,
we can limit how it is used by borrowing it only in the method call:

(example)

```rust
impl Config {
    fn save_triangle_configuration(&self, triangle: &mut Triangle) {
        triangle.save();
    }
}

impl Controller {
    fn shake_triangle(&self, triangle: &mut Triangle) {
        triangle.harlem();
    }
}
```

The `Config` and `Controller` no longer stores a reference to triangle.
The setup and usage looks like this:

(example)

```rust
let mut triangle = triangle::Triangle::new(&res, &gl)?;
let config = Config::new();
let controller = Controller::new();

..

config.save_triangle_configuration(&mut triangle);
..
controller.shake_triangle(&mut triangle);
```

The upside of this approach is that we can clearly see the dependencies and
what mutates what inside the render loop, which would be hard to track
with the `RefCell` approach.

The downside is that we can't pull OO shenanigans and create a sea of objects
with unclear data flow patterns where any component can be modified to mutate whatever
and whenever without a need to change an external component signature.
Ok, maybe this is not a downside. The downside is that we can't implement something like
a "Drawable" interface, because a `draw` on one component might need to borrow a completely
different thing than a `draw` on another component, making function
signatures different.

Is this a big issue? It is hard to tell now, when we are just starting.
However, we may always rememeber the `RefCell`, if only some use case justifies it.

## Viewport

Great, we have a better idea of how we want to continue. Our "components"
are going to be quite cheap, so we may not hesitate and create one for every
OpenGL use case.

For example, the viewport: right now, our window is resizable, therefore
we should update the viewport size on a window resize.

So our viewport struct should start with an initial width/height, have a method
to update width/height, and a function which changes OpenGL state
based on width/height.

Then, the viewport is really an utility that belongs to `render_gl` submodule.

(srs/render_gl/viewport.rs)

```rust
use gl;

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Viewport {
    pub fn for_window(w: i32, h: i32) -> Viewport {
        Viewport {
            x: 0,
            y: 0,
            w,
            h
        }
    }

    pub fn update_size(&mut self, w: i32, h: i32) {
        self.w = w;
        self.h = h;
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.Viewport(self.x, self.y, self.w, self.h);
        }
    }
}
```

Inside the `render_gl`, we may hide the fact that `Viewport` lives in its
own separate file by not making the module public and re-exporting the name
with `pub use` (like we did with the shader):

(src/render_gl/mod.rs, added)

```rust
mod viewport;
pub use self::viewport::Viewport;
```

I have not used `new` for `Viewport` constructor, and instead settled for `for_window`
where we don't need to specify `x` and `y`. It is common in Rust
to see different names for constructors that customise initialization.
And we can skip the `new` (which should probably have parameters to initialize every variable)
until it is needed.

We will likely adopt a convention to use `set_used` name for
functions that modify global open gl state. In this case, it actually
switches viewport configuration.

Inside the main, we can initialize the viewport:

(src/main.rs, modified)

```rust
let mut viewport = render_gl::Viewport::for_window(900, 700);
```

Switch to it before the main loop starts:

(src/main.rs, modified)

```rust
viewport.set_used(&gl);
```

And then, something new: we may update the viewport when the window is resized:

(src/main.rs, modified event loop)

```rust
for event in event_pump.poll_iter() {
    match event {
        sdl2::event::Event::Quit {..} => break 'main,
        sdl2::event::Event::Window {
            win_event: sdl2::event::WindowEvent::Resized(w, h),
            ..
        } => {
            viewport.update_size(w, h);
            viewport.set_used(&gl);
        },
        _ => {},
    }
}
```

This should run, and the triangle will now adjust its size on a window resize.

## Color Buffer

For the color buffer, we might store 4 floats inside a `ColorBuffer` struct:

(example)

```rust
pub struct ColorBuffer {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```

Or, we may bring in a `Vector4` type from some linear algebra crate that is also
suitable for games. There are quite a few of those to choose from.
In my first OpenGL renderer, I was using `cgmath` together with `collision`,
and had very few issues.

I have avoided other crates because of their excessive use of generics,
which makes auto-generated Rust documentation hard to understand, and harder if
we are new to Rust. Compared to that, `cgmath` is very straightforward,
and I still recommend it.

However, one other crate has caught my eye: `nalgebra`, together with
the crate that is build on it, `ncollide`. Looking further down the road,
there is also `nphysics`, that builds on both, a physics engine written
entirely in Rust!

While the reference documentation of `nalgebra` and `ncollide` is
a bit hard to navigate, the online documentation hands-down wins the contest.
Check out [nalgebra.org](http://nalgebra.org/) and [ncollide.org](http://ncollide.org/)!

An assurance that these lower-level functions are suitable for a
physics engine contributed the most for decision to bite
the bullet (no pun intended) and pick `nalgebra` for this and further lessons.

Let's include `nalgebra` in our project:

(Cargo.toml, added dependency)

```toml
nalgebra = "0.16"
```

(src/main.rs, added crate reference)

```rust
extern crate nalgebra;
```

We can then use 'nalgebra's `Vector4` type in our `ColorBuffer` struct:

(src/render_gl/mod.rs, added)

```rust
mod color_buffer;
pub use self::color_buffer::ColorBuffer;
```

(src/render_gl/color_buffer.rs)

```rust
use nalgebra as na;
use gl;

pub struct ColorBuffer {
    pub color: na::Vector4<f32>,
}

impl ColorBuffer {
    pub fn from_color(color: na::Vector3<f32>) -> ColorBuffer {
        ColorBuffer {
            color: color.fixed_resize::<na::U4, na::U1>(1.0),
        }
    }

    pub fn update_color(&mut self, color: na::Vector3<f32>) {
        self.color = color.fixed_resize::<na::U4, na::U1>(1.0);
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(self.color.x, self.color.y, self.color.z, 1.0);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
```

However, I have chosen to go with a simpler constructor that does
not specify an alpha, and uses `Vector3`, while there is `Vector4` in the struct.
That means we have to convert `Vector3` to `Vector4`.

We could simply construct it:

(example)

```rust
ColorBuffer {
    color: na::Vector4::new(color.x, color.y, color.z, 1.0),
}
```

However, there is a way to extend the vector and add one dimension:

(sample, used in code above)

```rust
ColorBuffer {
    color: color.fixed_resize::<na::U4, na::U1>(1.0),
}
```

What is going on? Well, almost all the types in `nalgebra` are aliases
to the _very_ generic `Matrix` type (we have created a few
aliases ourselves in the previous lesson).

Vector is simply a Matrix with one column. To resize, we use [matrix resize](http://nalgebra.org/vectors_and_matrices/#matrix-resizing)
function `fixed_resize` with generic parameters `na::U4` and `na::U1`, which mean
`4` rows and `1` column, respectively. Because Rust does not yet support parametrization over 
integer values, these types are used as a workaround. This is the same approach that we have used
in the previous lesson, for our `Buffer<BufferTypeArray>` and
`Buffer<BufferTypeElementArray>` aliases.

The resized matrix will be bigger, so the function has an argument for
a value to fill in these new cells, here we pass `1.0` for alpha.

Now we can initialize color buffer with a color we want:

(src/main.rs)

```rust
use nalgebra as na;
...
let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));
```

Then, call the `set_used` on this `ColorBuffer`, to change the clear color:

(src/main.rs)

```rust
color_buffer.set_used(&gl);
```

And use the `clear` to clear the color:

```rust
color_buffer.clear(&gl);
```

This should compile and run.

We have finally created some breathing space inside our main function.
Next time, we will try to explore a bit more than triangle!

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-13).
