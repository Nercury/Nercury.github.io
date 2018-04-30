---
layout: post
title:  "Embedded Rust Experiments - LED Displays and Wires"
date:   2018-05-10
categories: rust embedded experiments
---

The words "I built my own LED matrix display" are almost synonymous to "wires, wires everywhere".

![Single Matrix Wires](/images/mcu-02-real/arduino-multiplex-display-zoom-1.jpg)

Funny thing is, this solution is not even _that_ bad. See, the LED display used there
has a 8x8 LED matrix, with a total of 64 individually controlled LEDs. That would mean
64 wires. Clearly, we get away with 16 (if only counting the ones that are connected to the
display). That's because the display is connected in [multiplexed](https://en.wikipedia.org/wiki/Multiplexed_display)
way: 8 wires for rows, and 8 wires for columns, 16 wires in total.

### Multiplexing

If we want to control LEDs individually, only a single column has to be switched on at a
time, with appropriate rows active. By switching columns very quickly, we create
the illusion that all the pixels are visible at the same time. This 4x4 animation
demonstrates the technique very clearly:

![Multiplex 4x4 animation](/images/mcu-02-real/multiplex.gif)

([image source](http://www.franksworkshop.com.au/Electronics/RGB/RGB.htm))

