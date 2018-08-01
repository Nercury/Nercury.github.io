---
layout: post
title:  "Rust and OpenGL from scratch - Debug Lines and Shared Render Data Pattern"
date:   2018-08-01
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/07/27/opengl-in-rust-from-scratch-13-safe-triangle-nalgebra.html),
we have finished cleaning up unsafe code in our main function.

It is time to leave the flatland and go to 3D space.

But before we venture further, let's create a simple tool to mark the territory - 
a way to draw ad-hoc lines on screen:

![Debug lines](/images/opengl-rust/14/debug-lines.png)

They will help us later to set up the camera and possibly visually debug all kinds
of things.

## Debug Lines

We will create 3 kinds of markers: a point marker, a ray marker, and a polyline. Each of them
should be easy to use and update. However, it would be great if we could draw all of them 
in a single draw call.

[Full source code is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-14). 