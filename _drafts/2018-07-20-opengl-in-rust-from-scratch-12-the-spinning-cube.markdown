---
layout: post
title:  "Rust and OpenGL from scratch - The Spinning Cube"
date:   2018-07-20
categories: rust opengl tutorial
---

> ["Rust and OpenGL from scratch"](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html) 
> series aims to introduce you 
> to various Rust features and patterns, from simple to advanced, 
> from common to obscure, while at the same
> time working towards a simple OpenGL renderer.

Welcome back!

Previously, we have added support for many OpenGL data types to our
`render_gl` module. We can now render our triangle using
`GL_UNSIGNED_INT_2_10_10_10_REV` for color, as an example.

It's time we get rid of the triangle though (12 lessons in, it's about time),
and add a third dimension. A spinning cube!

We will start using Model-View-Projection matrix in our shaders,
we will augment the shader API to set uniforms, and we will restructure
our `main` function into separate parts. Let's get started!



[Full source code, as always, is available on github](https://github.com/Nercury/rust-and-opengl-lessons/tree/master/lesson-12).