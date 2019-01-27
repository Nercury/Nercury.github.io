---
layout: post
title:  "Embedded Rust Experiments - LED Displays and Wires"
date:   2019-05-10
categories: rust embedded experiments
---

The words "I built my own LED matrix display" are almost synonymous to "wires, wires everywhere".

![Single Matrix Wires](/images/mcu-0x/arduino-multiplex-display-zoom-1.jpg)

Funny thing is, this solution is not even _that_ bad. See, the LED display here
has a 8x8 LED matrix, with a total of 64 individually controlled LEDs. That would mean
64 wires. Clearly, we get away with 16 (if only counting the ones that are connected to the
display). That's because the display is connected in [multiplexed](https://en.wikipedia.org/wiki/Multiplexed_display)
way: 8 wires for rows, and 8 wires for columns, 16 wires in total.

### Multiplexing

If we want to control LEDs individually, only a single column has to be switched on at a
time, with appropriate rows active. By switching columns very quickly, we create
the illusion that all the pixels are visible at the same time. This 4x4 animation
demonstrates the technique very clearly:

![Multiplex 4x4 animation](/images/mcu-0x/multiplex.gif)

([image source](http://www.franksworkshop.com.au/Electronics/RGB/RGB.htm))

The ones and zeroes here represent the rows and columns we want to be active, but not the
actual voltage. For the current to flow over a LED diode, we need to connect the anode to 
_high_ and cathode to _low_ voltage, because the current only flows one way through the diode. 
The column values in the picture above would need to be inverted from `1` to `0` and vice-versa.

![LED pinout](/images/mcu-0x/led_pinout.png)

([image source](https://www.allaboutcircuits.com/tools/led-resistor-calculator/))

![ULN2803A](/images/mcu-0x/uln2803a.jpg)

![Multiplex with inverter and resistors](/images/mcu-0x/multiplex-full.png)