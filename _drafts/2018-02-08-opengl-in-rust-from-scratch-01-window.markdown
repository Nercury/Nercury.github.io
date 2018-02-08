---
layout: post
title:  "Rust and OpenGL from scratch - Window"
date:   2018-02-08
categories: rust opengl tutorial
---

Welcome back!

[Previously](/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html), 
we got some motivation to use Rust and configured our 
development environment.

In this part, we will create a window! If that does not sound too exciting, we will also learn
about Rust libraries. Very exciting.

## A window

To create a window that works across multiple platforms, as well as provides such niceties as
OpenGL context or multi-platform input, we will use SDL2.

