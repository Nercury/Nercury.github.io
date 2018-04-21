---
layout: post
title:  "Embedded Rust Experiments - STM32VLDISCOVERY board"
date:   2018-04-21
categories: rust embedded experiments
---

Currently I am learning about microcontrollers (MCU) more powerful than 8-bit Atmega. Naturally,
MCUs need code, and I want to use Rust if possible.

For this purpose I've got [STM32VLDISCOVERY board](http://www.st.com/content/st_com/en/products/evaluation-tools/product-evaluation-tools/mcu-eval-tools/stm32-mcu-eval-tools/stm32-mcu-discovery-kits/stm32vldiscovery.html), which is fairly cheap,
and [was mentioned in japaric's post on Rust for embedded development](https://users.rust-lang.org/t/rust-for-embedded-development-where-we-are-and-whats-missing/10861). 

## Where I am now

Well, I've got the board:

![Discovery VL](/images/mcu-01/discovery-vl.jpg)

I've followed ["Discover the world of microcontrollers through Rust!"](https://japaric.github.io/discovery/README.html) tutorial
to set up tools and the first project. However, since the tutorial is for another board,
I have used [cortex-m-quickstart](https://github.com/japaric/cortex-m-quickstart) project
documentation as a base. I stumbled a bit when I used incorect
architecture for MCU, or until I found the right crates to look into. There is
some duplicated crates, a lot of outdated information (xargo not necessary!), but I
eventually found the before mentioned tutorial-book which put me on the right track.

Now I have this project uploaded to the board, and it sends back the "Hello world" text
over the gdb link:

![Hello world in a console](/images/mcu-01/discovery-vl-helloworld.jpg)

## Goal

Previously, I was hesitant to try Rust, and instead stayed in a comfortable Arduino world.

Here I am controlling various LED displays over I2C interface with Arduino Nano:

![Arduino Nano and LED displays](/images/mcu-01/arduino-displays.jpg)

My first goal is to learn to do the same with Rust and Cortex-M, which looks challenging
at a first glance, primarily because of more advanced MCU with a big
datasheet.

## Summary

While the Rust tools are nothing like Arduino right now, it is not too hard to set them up.
I am going to blog more of my progress here as I stumble on the next interesting thing.