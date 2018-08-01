---
layout: post
title:  "Rust and OpenGL from scratch - Camera"
date:   2018-08-02
categories: rust opengl tutorial
---

Welcome back!

Let's learn how to build a camera and go 3D.

## 3D graphics quick summary

To display a 3D view, we need to _transform_ positions that have 3 dimensions (x, y, z)
to positions on screen that have only 2 dimensions (x, y). This view can also mimic
a view of a camera, where objects farther away from the camera are displayed smaller.

Let's say we have two objects that are moving away from the camera. If the camera
is stationary, the objects will appear to be getting closer in 2D view. This
is called a perspective view.

For each stationary position and rotation of the camera, it is possible to 
create a transformation such that it can transform a 3D object position to 2D
view position as if viewed from that camera.

When the camera is updated, we also update the transformation. When we display the
objects in a 2D view, we transform their position based on the camera transformation.

It goes a bit deeper: for each _pixel_ we display in 2D view, we transform a point on
a 3D object to a point on 2D screen.

With this figured out, we won't need to think about this transformation again.
If we use window events like key presses and mouse movement to change the position
and orientation of the camera, then we can continue working with our objects
in the 3D space and use the camera to look around. The illusion will be complete.

### Representing Position and Transformation

We use Vectors to represent position and Matrices to represent transformation.
Additionally, we can use Quaternions for rotation.

The most important thing here is to get some intuition in vectors, transformations and matrices.
For a very visual walkthrough, I recommend [linear algebra video series by
3Blue1Brown](https://www.youtube.com/watch?v=fNk_zzaMoSs&list=PLZHQObOWTQDPD3MizzM2xVFitgF8hE_ab&index=2).

All these concepts apply to 2D world too, with a third axis remaining at 0.
If you are new to computer graphics and are going to avoid 3D and build a 2D game,
learning these concepts will still make your life easier.

### Model, View, Projection

Rememeber how we defined positions of triangle vertices in an array?

(example)

```rust
Vertex {
    pos: (0.5, -0.5, 0.0).into(),
}, // bottom right
Vertex {
    pos: (-0.5, -0.5, 0.0).into(),
}, // bottom left
Vertex {
    pos: (0.0,  0.5, 0.0).into(),
}  // top
```

Imagine we would render many triangles. It would be unfortunate for if we had to calculate
different vertices for each different position and rotation of the triangle in the
2D/3D world.

Instead, we can say that our triangle is a _Model_, and these coordinates are in
__model space__.

![Triangle](/images/opengl-rust/15/model-space.png)

We can place many instances of the triangle in the world. Each can
have their own individual translation, rotation, scale and shear, stored
together in a __model transformation matrix__. They are all now
in the __world space__:

![Triangle](/images/opengl-rust/15/world-space.png)

Then, the camera comes in:

![Triangle](/images/opengl-rust/15/world-space-camera.png)

Like any model, camera also has position and rotation. They are used to 
create a __view transformation matrix__, that transforms all the world as if the origin
was at the point of camera. The objects are now in the __view space__:

![Triangle](/images/opengl-rust/15/view-space.png)

Lastly, the __projection matrix__ transforms the world into the __screen space__.

## Building a Camera in Rust

We will build a camera that swings around some target point. We will put 
the camera in another `camera` submodule, and name the camera `TargetCamera`:

(src/main.rs)

```rust
pub mod camera;
```

(src/camera/mod.rs)

```rust
mod top_down;
pub use self::top_down::TargetCamera;
```

(src/camera/mod.rs)

```rust
mod top_down;
pub use self::top_down::TargetCamera;
```

(src/camera/target.rs)

```rust
use nalgebra as na;

pub struct TargetCamera {
    target: na::Vector3<f32>,
    rotation: na::Quaternion<f32>,
    distance: f32,
}
```

The target is

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-15). 