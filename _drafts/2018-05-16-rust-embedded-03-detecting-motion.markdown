---
layout: post
title:  "Embedded Rust Experiments - Detecting Motion"
date:   2018-05-16
categories: rust embedded experiments
---

This is an accelerometer, a gyroscope and a magnetometer all in one 3mm x 3.5mm chip:

![LSM9DS1TR](/images/mcu-03/acc-chip.jpg)

([LSM9DS1TR][acc-datasheet] next to a 2-euro coin and a standard DIP package)

For a small number of samples, it does not come cheap: 
[one chip costs around 5 euros](https://octopart.com/search?q=LSM9DS1TR). However, I find it
absolutely amazing anyway that this chip is so easy to get. Besides, having 3 features in
one package means more experiments and less datasheets.

## Prototype board

There are already boards with a price as low as 12 euros designed for evaluating
the LSM9DS1 chip. For example, [this one from sparkfun][acc-sparkfun].

However, with a goal to learn, we are going to build basically the same thing
ourselves. If we get stuck, we can always peek into sparkfun's open-source
design documents for a reference.

### Schematic

I am going to use [EasyEDA's](https://easyeda.com) free online schematic and
PCB editor. We start by creating a new schematic, and searching for LSM9DS1
component:

![Place component](/images/mcu-03/eda-place-component.jpg)
![Find LSM9DS1](/images/mcu-03/eda-find-acc.gif)

Luckily for us, it already exists! We can immediately start building around it:

![LSM9DS1 in schematic](/images/mcu-03/eda-acc-symbol-in-schematic.gif)

We can start reading the [datasheet][acc-datasheet] and taking notes.

```plain
The LSM9DS1 includes an I2C serial bus 
interface supporting standard and fast 
mode (100 kHz and 400 kHz) and an SPI 
serial standard interface. 
```

Great, we will focus on `I2C`.

Let's follow pin descriptions in `Table 2`:

![Pin descriptions](/images/mcu-03/acc-pin-description.gif)

and `Figure 15` example schematic in `4. Application hints`:

![Application hints schematic](/images/mcu-03/acc-hints-schematic.gif)

We _could_ blindly repeat everything we see in this schematic, but
we might miss some crucial detail while doing so. Instead, we will start from
scratch and double check every connection.

#### Power Supply

Let's start from the obvious and figure out the less obvious as we go along.
First, let's plug ground `GND` pins `19` and `20` to our ground:

![Connect ground](/images/mcu-03/eda-01-gnd.gif)

Pins `14`, `15`, `16`, `17` and `18` are named `RES` and have the same comment:

```plain
Reserved. Connected to GND.
```

Okay:

![Connect ground to res](/images/mcu-03/eda-02-gnd-to-res.gif)

Then, we have _two_ different high voltage inputs. `VDDIO`, which is

```plain
Power supply for I/O pins
```

And `VDD`, which is

```plain
Power supply
```

In `Electrical characteristics` (Table 4):
- `VDDIO` has `1.71V` minimum voltage for `I/O`;
- `VDD` has `1.9V` minimum core voltage (maximum is `3.6V`). 

While the core chip voltage (>`1.9V`) is less flexible, we could still interface 
with this device over the lower-voltage `I/O` pins.
Say, if our MCU runs on a `1.8V`, we could supply regulated `1.8V` to
`VDDIO`. For the `VDD`, we could ensure voltage between `1.9V` and `3.6V` 
(maybe directly from battery).

This means it's best to keep `VDD` and `VDDIO` separated. On the
board we are building, we will dedicate separate pins for them,
to leave an option to use a lower-voltage MCU when interfacing over I/O pins.

Let's connect them to `VDD_IO` and `VDD` nets:

![Connect VDDIO and VDD](/images/mcu-03/eda-03-vddio-and-vdd.gif)

The names `VDD_IO` and `VDD` in schematic can be chosen freely,
and they simply mean that everything connected to, say, `VDD`
is connected together. The same goes for `GND` even though it has a
different "ground" symbol. 

![Ground net](/images/mcu-03/eda-05-gnd-net.gif)

There is a third way to do basically _the same thing_, the net label:

![Net label example](/images/mcu-03/eda-04-net-label.gif)

We will use it for clock and data pins soon.

#### Decoupling capacitors

[A decoupling capacitor is a capacitor used to decouple one part of an electrical network (circuit) from another](https://en.wikipedia.org/wiki/Decoupling_capacitor).
The reason to do this is because of noise, caused by other chips in a system,
as well as the noise caused by the chip that is being decoupled.

![Decoupling capacitor schematic](/images/mcu-03/decoupling-capacitor.gif)

This noise occurs because device draws varying amounts of current over time.
When a device suddenly needs to draw more current, the decoupling capacitor 
acts as a local energy storage to draw this current from, and reduces a
voltage dip for other devices that would occur if there would be no such
capacitor.

It all seems quite complicated - and it is. But more often than not, the
device datasheets have decoupling capacitor recommendations. That's what we are 
going to go with.

We saw the recommended capacitors listed under the `Table 2`, bellow the pin
descriptions.

For both `VDD` pins, it said:

```plain
...
4. Recommended 100 nF plus 10 μF capacitors. (C3)
5. Recommended 100 nF plus 10 μF capacitors. (C4)
```

For `VDDIO`:

```plain
1. Recommended 100 nF filter capacitor. (C2)
```

In the `4. Application hints - 4.1 External capacitors` section, there is more
information:

```plain
Power supply decoupling capacitors 
(C2, C3 = 100 nF ceramic, C4 = 10 μF Al) 
should be placed as near as possible to 
the supply pin of the device (common 
design practice).
```

It turns out, the `VDD` line needs _two_ capacitors.

And we can't just plop same type, but different value
capacitors everywhere and call it a day. The
`0.1 μF` capacitor needs to be ceramic, and `10 μF` one 
- aluminum electrolytic.

Why though? To scratch the surface of this big topic, I will try
to summarize it with this diagram:

![Capacitor impendance and frequency relation](/images/mcu-03/cap-impendance-frequency.gif)

High impedance means that no current flows though the capacitor,
and the low impedance means that the capacitor actually works.

(this image is [from an excellent answer to the question "does this mean my
capacitor no longer functions above certain frequency?"](https://electronics.stackexchange.com/questions/327975/capacitance-vs-frequency-graph-of-ceramic-capacitors))

Where we can we find more information about frequency vs. impedance of
the `10 μF` and `0.1 μF` that we need? Their datasheets, of course.

Since we are designing this board with the intent to actually _produce_
it, we should pick the exact components we plan to place.

One of the nice places to look for parts is [Octopart](https://octopart.com).
It is fast, contains price and availability information, and all the components
are searchable by almost any property.

First, let's find an aluminum electrolytic capacitor with the value of `0.1 uF`.

Let's go to _Passive Components_ / _Capacitors_ / _Aluminum Electrolytic Capacitors_,
and specify `0.1 uF` capacitance in the filter, availability - in-stock-only, and
I also filter the distributor to the one that is most hassle free for me (I live in
EU, so I select Farnell). We can also sort by price, to get rid of the more exotic
options.

Few more filters I like: I usually filter by tolerance, and select the smallest tolerance
with a good variety and good stock availability. In this case, I am going with `10%` 
(I started from `5%` though, and found a very limited amount of options). 
The big tolerance caps are cheaper, but since the price
is so low anyway, I will order a bit more of everything, to mitigate the risk of loosing
some of the parts under the table.

Speaking about loosing the parts, let's also select the package size of our part.
Looks like this kind of cap is widely available as a SMD package (surface mount).

![SMD sizes](/images/mcu-03/dust.jpg)

The size of `0201` is dust, we would need a microscope to solder it or anything smaller.
The `0402` is sand, but is surprisingly manageable with some practice. The `0603` is
"normal" while still being very tiny. The `0805` and anything bigger is very easy to 
work with.

Looks like in this case, the smaller packages are 10 times cheaper (in small quantities), 
and I kind of like the idea of getting away with the smallest component possible.

![0.1 uF 0402 CMD capacitor](/images/mcu-03/cap-0u1.gif)

Few more things to watch out. Smaller capacitors usually have lower minimum voltage. 
`16V` is fine in our case. Another property is the type of dielectric quality, [and
I consulted this article to find out that X5R is more than enough](http://www.raviyp.com/embedded/217-difference-between-x7r-x5r-x8r-z5u-y5v-x7s-c0g-capacitor-dielectrics).


[acc-datasheet]: http://www.st.com/resource/en/datasheet/lsm9ds1.pdf
[acc-sparkfun]: https://www.sparkfun.com/products/13284