---
layout: post
title:  "Rust and OpenGL from scratch - OpenGL Context"
date:   2018-02-09
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html), 
we have created a window using SDL and started accepting window events in a loop.

In this lesson, we will load up OpenGL context and color the window blue.

## OpenGL

If you [search for OpenGL in crates.io](https://crates.io/search?q=opengl), you may find
multiple choices of safe OpenGL wrappers, such as `glium`, `glitter` and maybe others.

The purpose of this series, however, is to learn about the lower level details. That does
not mean that we are going to do all the work ourselves!

## gl and gl_generator

The cornerstone of many higher level OpenGL projects is [gl-rs project](https://github.com/brendanzab/gl-rs/),
which contains three crates:

- `khronos_api` - this crate contains XML files for various Khronos APIs, such as EGL,
WGL, GLX, GL, and others.
- `gl_generator` - is a tool to generate Rust code bindings from `khronos_api` XML files.
Think of it as your own customizable generator for OpenGL loader (something like [GLEW](http://glew.sourceforge.net/)).
- `gl` - is a OpenGL API already generated using `gl_generator`. That was almost correct.
In fact, `gl` rust code is generated every time `gl` is recompiled. More on that in later lesson.

## GL

For now, we will use `gl` crate.

[The gl crate page on crates.io contains information how to set it up](https://crates.io/crates/gl). We will
follow that.

Add `gl` dependency to your `Cargo.toml`:

(Cargo.toml, incomplete)

```toml
[dependencies]
gl = "0.10.0"
```

Add reference to gl crate to your project by referencing `gl` crate at the top of your
`main.rs` file:

(main.rs, incomplete)

```rust
extern crate gl;
```

## Browsing documentation of a dependency locally

We can ask cargo to generate and open documentation on any crate our project depends on:

```txt
cargo doc -p gl --no-deps --open
```

- `-p gl` is a short for `--package gl`, specifies that we want documentation for it
- `--no-deps` disables recursive documentation for all dependencies of `gl`. We want to disable that
on windows, because `gl` transitively depends on `winapi`, which is extremely large, and generating 
docs for `winapi` would take too much time.
- `--open` opens generated documentation in a browser window.

## OpenGL context

We can tell SDL to create OpenGL context. We will use [gl_create_context method on sdl2::video::Window
struct](https://rust-sdl2.github.io/rust-sdl2/sdl2/video/struct.Window.html#method.gl_create_context).

However, only windows created with `opengl` flag can have OpenGL context.

Modify the window creation code to have `opengl` flag:

(main.rs, fn main, incomplete)

```rust
    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl() // add opengl flag
        .resizable()
        .build()
        .unwrap();
```

And then create the `gl_context`:

(main.rs, fn main, incomplete)

```rust
let gl_context = window.gl_create_context().unwrap();
```

## More SDL documentation

It is useful to rememeber that Rust's SDL is a very thin wrapper over C SDL library.
Therefore, online documentation for original SDL maps directly to rust SDL crate.

[Let's open libsdl documentation for SDL_GL_CreateContext](https://wiki.libsdl.org/SDL_GL_CreateContext).

It has this example code to create a window:

```c
// Window mode MUST include SDL_WINDOW_OPENGL for use with OpenGL.
SDL_Window *window = SDL_CreateWindow(
    "SDL2/OpenGL Demo", 0, 0, 640, 480, 
    SDL_WINDOW_OPENGL|SDL_WINDOW_RESIZABLE);
```

For `SDL_CreateWindow` call to work at all, `SDL`'s `video` subsystem must be initialized.
Rust `sdl2` crate ensures that it is so by making window creation require `video_subsystem`:
`video_subsystem.window(...)`.

Instead of flags, on the Rust side we have `WindowBuilder`, which is used to build up the flags
and other parameters for `SDL_CreateWindow` call.

Our calls to make window `.opengl().resizable()` will have the same result as
`SDL_WINDOW_OPENGL|SDL_WINDOW_RESIZABLE` on the C side.

One thing many C examples are usually missing, is error handling. If `SDL_CreateWindow` returns
null, the program exits with segmentation fault and no useful error message. The call
to `.unwrap()` is way better, because it panics with an error message retrieved from `SDL_GetError()`
call. You can try that it is so by omitting `.opengl()` flag in our Rust code; `window.gl_create_context()`
returns error that is sent to the output when `unwrap` panics: 

```txt
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: 
    "The specified window isn\'t an OpenGL window"', src\libcore\result.rs:906:4
```

The `SDL_GL_CreateContext` function requires a window:

```c
// Create an OpenGL context associated with the window.
SDL_GLContext glcontext = SDL_GL_CreateContext(window);
```

In Rust, it becomes a method on the `Window` struct:

```rust
let gl_context = window.gl_create_context().unwrap();
```

In conclusion, original SDL tutorials, online resources, and stack overflow answers
remain useful in Rust.

## Loading function pointers

What does `gl` crate actually do? It forwards OpenGL function calls to 
the driver.

Gl crate itself does not know how to do that, but SDL does.

When we initialize gl, we provide a function to load function pointer by string:

```rust
let gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
```

The `|s| video_subsystem.gl_get_proc_address(s)` is a closure with a single `s` parameter
that is inferred to be a string slice. The return type of `gl_get_proc_address`, however, does
not match `*const std::os::raw::c_void` pointer expected by `gl`, but we can cast it with 
`as` operator.

- [Read more about Rust Strings](https://doc.rust-lang.org/book/second-edition/ch08-02-strings.html)
- [String slices](https://doc.rust-lang.org/book/second-edition/ch04-03-slices.html)
- [Rust closures](https://doc.rust-lang.org/book/second-edition/ch13-01-closures.html)

## Clear color

We can now use gl functions!

Before the `'main: loop {`, we can set gl "clear" color to blue:

(main.rs, incomplete)

```rust
unsafe {
    gl::ClearColor(0.3, 0.3, 0.5, 1.0);
}
```

And replace `// render window contents here` with actual call to clear color, as well call
to SDL2 to swap the window pixels with what we have just rendered:

(main.rs, incomplete)

```rust
'main: loop {
    // .. event handling here
    
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    
    window.gl_swap_window();
}
```

[Full code can be found here](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-02).

Next up, 
[a long and "modern" road to output a triangle to to the screen](/rust/opengl/tutorial/2018/02/10/opengl-in-rust-from-scratch-03-compiling-shaders.html).