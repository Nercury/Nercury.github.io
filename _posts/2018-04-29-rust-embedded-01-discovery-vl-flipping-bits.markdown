---
layout: post
title:  "Embedded Rust Experiments - Flipping some bits high on STM32VLDISCOVERY board"
date:   2018-04-29
categories: rust embedded experiments
---

The ["Discover the world of microcontrollers through Rust!"][mcu-world]
book describes how to set up rust project for STM32F3DISCOVERY board.

Here I will document the steps to get started with STM32VLDISCOVERY board.

We will take my favourite "from scratch" approach. That way, we build the final
thing step by step while building our understanding of how it all fits together.

I am new to Cortex-M, and I am new to Rust's embedded development, so please excuse me if 
I get some terminology horribly wrong. I am also learning along the way, and then
I try to present what I have learned in a sequential, easy to understand way.

## Documentation

Go to ST site and [locate the board documentation](http://www.st.com/en/evaluation-tools/stm32vldiscovery.html).

Board uses STM32F100RBT6B Cortex-M3 microcontroller. [Locate the documentation of this microcontroller too](http://www.st.com/en/microcontrollers/stm32f100rb.html).
The page has many PDFs. [We are interested in what is called the "reference manual"][mcu]. 
It's big.

At the end of this page, there is a [literature list](#literature).

The final code [is available on the github](https://github.com/Nercury/embedded-experiments/tree/master/part01-blink).

## Connection to the board

The board contains the microcontroller and everything that is required to run it,
as well as programmer called ST-Link.

![Discovery VL labels](/images/mcu-02/discovery-vl-labels.jpg)

By default it is configured to program the microcontroller on the board. However,
we can change some jumpers, connect 4 wires to pins marked "SWD" and use ST-Link 
to program similar microcontroller on another board designed by us. This
board is good for testing this microcontroller, because all the pins are accessible 
and plugable to a breadboard.

We connect to ST-Link over an USB cable.

From now on, I will call the microcontroller MCU.

## USB and ST-Link

We will make sure that we have a connection to our board by installing OpenOCD (which
will be used to upload our program to MCU).

[The book][mcu-world] has wildly different instructions for [Linux](https://japaric.github.io/discovery/03-setup/linux.html),
[Windows](https://japaric.github.io/discovery/03-setup/windows.html) and 
[macOS](https://japaric.github.io/discovery/03-setup/macos.html). Follow them and install
 tools for your platform (except the windows driver).

For Windows, you can download the driver [from the board page we've visited before](http://www.st.com/en/evaluation-tools/stm32vldiscovery.html).

Then, follow the instructions to [verify that you have connection to the board](https://japaric.github.io/discovery/03-setup/verify.html) with one
caveat: change `openocd` call to use different device than the book says. Our
device in our board is `STM32F100RB`. We can browse installed `OpenOCD` `scripts`
to locate the board `cfg` file.

With OpenOCD 0.10, this works for me:

```bash
openocd -f interface/stlink.cfg -f target/stm32f1x.cfg
```

The book uses `interface/stlink-v2-1.cfg`, which works, but we get the deprecation warning
with a note to switch to `interface/stlink.cfg`, which works too.

It is likely that you will receive this error on Windows:

```plain
...
Error: libusb_open() failed with LIBUSB_ERROR_NOT_SUPPORTED
```

Windows has multiple mechanisms to load drivers. OpenOCD uses WinUSB. 
I had to use Zadig tool to install WinUSB version of STM32 STLink driver.
Follow the [instructions here][zadig-instructions], except our driver should be named
"STM32 STLink".

[zadig-instructions]: https://github.com/foss-for-synopsys-dwc-arc-processors/arc_gnu_eclipse/wiki/How-to-Use-OpenOCD-on-Windows

![Discovery VL labels](/images/mcu-02/st-link-blink.gif)

If the LED starts blinking, the connection is successful! You should leave the OpenOCD
running in another terminal window.

## Back to Rust

We will bootstrap a Rust project from scratch.

First, let's create a simple Rust project:

```bash
cargo new --bin blink
cd blink
```

This generates the familiar "Hello, world!" project for us:

(src/main.rs)

```rust
fn main() {
    println!("Hello, world!");
}
```

### No std

MCUs have small, constrained environments. Convenient things like heap require 
more work or are not used at all. We start by disabling Rust's standard library:

(at the top of main.rs)

```rust
#![no_std]
```

And guess what, the `println!` macro no longer exists:

```txt
error: cannot find macro `println!` in this scope
```

So we remove it:

(main.rs, full)

```rust
#![no_std]

fn main() {
}
```

We get more errors when compiling:

```txt
error: language item required, but not found: `panic_fmt`
error: language item required, but not found: `eh_personality`
```

### Lang items and nightly Rust

Turns out, Rust's std library was filling in some gaps in Rust's core
which are now missing. For example, it was handling panics.

We can fill in our own panic handler with this code, which simply never finishes
in a case of panic (and returns `!` "never" type):

(main.rs, bellow main() function)

```rust
#![feature(panic_implementation)]

use core::panic::PanicInfo;

#[panic_implementation]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

For this to work, we need to enable `panic_implementation` feature:

(at the top of main.rs)

```rust
#![feature(panic_implementation)]
```

Features require nightly rust, so if you are not already on nightly, you will
have to switch to it. It's best to add a project specific override:

(in project's dir)

```bash
rustup override set nightly
```

### Note on stability

We switched to nightly Rust, and this means this tutorial may become outdated
quickly. If something does not work, you may need to do your own googling and
reading, much like I had to do while documenting my steps here.

### Eh, personality!

We are down to one error:

```txt
error: language item required, but not found: `eh_personality`
```

The `eh_personality` is used to implement stack unwinding in case a panic occurs.

But this is very dependent on the platform we are compiling to; right up to this point we
attempted to compile `no_std` executable for windows/linux/mac; it's time we switch our target architecture
to the one required by our MCU.

## ARM target

Our board has Cortex-M3 MCU, and the rust "target" name for it is `thumbv7m-none-eabi`.
This identifier is the best-effort combination of names that describe instruction set (thumbv7m),
operating system (none) and executable format (ELF ABI). It does not have "hf" suffix at the end
because our MCU does not support hardware floating point operations.

If we used another Cortex-M MCU, we would choose another target triple:

- `thumbv6m-none-eabi`, for the Cortex-M0 and Cortex-M1 processors
- `thumbv7m-none-eabi`, for the Cortex-M3 processor
- `thumbv7em-none-eabi`, for the Cortex-M4 and Cortex-M7 processors
- `thumbv7em-none-eabihf`, for the Cortex-M4F and Cortex-M7F processors

Rustup can download core library for our target: 

```bash
rustup target add thumbv7m-none-eabi
```

From now on, when we compile our program, we use `--target thumbv7m-none-eabi` flag:

```bash
cargo build --target thumbv7m-none-eabi
```

When we try to compile it now, we see that `eh_personality` error disappeared, and 
instead we get:

```txt
error: requires `start` lang_item
```

## Cortex-M Runtime

Turns out that the main function is not the first thing that is called when the program starts.
On Windows, the entry function is `mainCRTStartup`, which initializes some things and
only then it calls the `main`.

There is also a similar story when we build executable for our MCU: 
_something_ on the MCU has to invoke our main function, like when we press the reset 
button, or when the MCU boots up. And the Rust can't know
about it, therefore it tells us to define our own `start` function, which can then
call the familiar `main`.

There is a library we can link into our program that provides a minimal runtime for
Cortex-M microcontrollers: it has a reset handler which calls our main, and also
defines the `start` lang item.

Let's add it to `Cargo.toml` dependencies:

(Cargo.toml)

```toml
[dependencies.cortex-m-rt]
version = "0.4"
```

And reference it in the root of our crate:

(main.rs)

```rust
extern crate cortex_m_rt;
```

Let's build it:

```txt
error: linking with `arm-none-eabi-gcc` failed: exit code: 1
```

We failed to link! (If you get another error, [make sure you have followed the instructions to
set up the tools for your platform, as well as you have got the ARM linker](#usb-and-st-link))

[The documentation for cortex-m-rt](https://docs.rs/cortex-m-rt/0.4.0/cortex_m_rt/) crate
tells us that we need to pass additional flags to rustc compiler, like this:

(example)

```bash
cargo rustc --target thumbv7m-none-eabi -- \
      -C link-arg=-Tlink.x -C linker=arm-none-eabi-ld -Z linker-flavor=ld
```

It would work, but it is more convenient to put this configuration into
`.cargo/config` file. That way cargo will pick these flags automatically when
building the appropriate architecture.

Let's create `.cargo/config` for our project. It follows `toml` file format, but does
not have `.toml` extension:

(.cargo/config)

```toml
[target.thumbv7m-none-eabi]
runner = 'arm-none-eabi-gdb'
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "linker=arm-none-eabi-ld",
  "-Z", "linker-flavor=ld",
  "-Z", "thinlto=no",
]
```

Now invoke `cargo build --target thumbv7m-none-eabi` as before. It should pick up
the config file and use the correct linker. If we are successful, we will see this
error (I know, it never ends):

```txt
error: linking with `arm-none-eabi-ld` failed: exit code: 1
  |
  = ...
  = note: arm-none-eabi-ld.exe: cannot open linker script file memory.x: 
    No such file or directory
```

When linking the executable, the linker picks up information about our MCU from
file `memory.x`, which describes certain memory locations on our MCU. Different MCUs will
have different values there, so ideally we would need to figure them out from the datasheet.

But if you are using the same board, go ahead and create the same file:

(memory.x)

```python
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* Adjust these memory regions to match your device memory layout */
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

And then we should be greeted by the next error:

```txt
error: linking with `arm-none-eabi-ld` failed: exit code: 1
  |
  = ...
  = note: arm-none-eabi-ld.exe: 
          The interrupt handlers are missing. If you are not linking to a device
          crate then you supply the interrupt handlers yourself. Check the
          documentation.
```

There are 2 things we get from this helpful error message: first, there are device crates
that we can link to, and second, that we can supply the interrupt handlers ourselves.

Supplying a blank interrupt handler ourselves is easy, we just add this snippet of code:

(at the end of main.rs, requires `#![used]` and `#![feature(used)]` crate attributes)

```rust
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {}
```

And indeed, this finally builds! However, the first method of using a device crate sounds
like it would help us avoid a lot of trouble.

Instead of supplying our own interrupt handler, we will link to `stm32f1` device crate,
because the name of our Cortex-M MCU is `STM32F100RB`:

(Cargo.toml)

```toml
[dependencies.stm32f1]
version = "0.1.0"
features = ["stm32f100", "rt"]
```

We select the device we want with `stm32f100` feature flag ([see the list of devices in crate
README](https://crates.io/crates/stm32f1)), and we can also add `rt` feature, which brings
in `cortex-m-rt` we added previously. In fact, we can remove `cortex-m-rt` from
our dependencies now.

With that done, we can get rid of `panic_fmt`
lang item by including `panic-abort` crate, which aborts on panic:

(Cargo.toml)

```toml
[dependencies]
panic-abort = "0.1.1"
```

(main.rs)

```rust
extern crate panic_abort;
```

We can now remove `#![feature(lang_items)]` and `rust_begin_unwind` function.

## The final, minimal result that can run

We are now left with this `main.rs`:

```rust
#![no_std]

extern crate stm32f1;
extern crate panic_abort;

fn main() {
}
```

This (Cargo.toml):

```toml
[package]
name = "part01-blink"
version = "0.1.0"

[dependencies]
panic-abort = "0.1.1"

[dependencies.stm32f1]
version = "0.1.0"
features = ["stm32f100", "rt"]
```

`memory.x` file, `.cargo/config` file, and a decent grasp of how this all fits
together.

One small detail: to minimize the executable size, we should always compile with `--release`
flag. To optimize the usage of many crates, we should link with Link Time Optimization (LTO).
And we can also enable debugging in release mode. We can set this up in `Cargo.toml`
release profile:

(Cargo.toml, at the end)

```toml
[profile.release]
lto = true
debug = true
```

## Running!

When we run our program now, we should find ourselves in a `gdb` console:

```gdb
Reading symbols from target\thumbv7m-none-eabi\debug\part01-blink...done.
(gdb)
```

We should make sure that [OpenOCD is running (discussed previously)](#usb-and-st-link).
Then, from `gdb` we can connect to it:

```gdb
(gdb) target remote :3333
Remote debugging using :3333
...
```

We can now load our program to MCU:

```gdb
(gdb) load
Loading section .vector_table, size 0x130 lma 0x8000000
Loading section .text, size 0x23e lma 0x8000130
Start address 0x8000130, load size 878
Transfer rate: 5 KB/sec, 439 bytes/write.
```

And then continue:

```gdb
(gdb) continue
Continuing.
```

Of course we won't see anything else, because our program is empty right now.

## Lighting up the LEDs

In our case, we are interested in turning on the built-in LEDs on our board. 

![Image of PC8 and PC9 LEDs](/images/mcu-02/leds-pc8-pc9.jpeg)

What do those PC8 and PC9 names mean?

### Ports

MCU's inputs and outputs are grouped into _ports_. Ports are named `A`, `B`, `C` and so on.
On this MCU, each port has 16 _pins_, that correspond to physical pins on the board.
They are named from `0` to `15`. So, the pin `9` on port `C` would be named `PC9`.

### Port I/O

Each pin can be individually configured to be either an _input_ or an _output_.

#### Input

In _input_ mode, pin is configured to _sense_ the voltage applied to it. By default
the pin is configured to sense digital logic, that means that low voltage will
be read as `0`, while the high voltage will be read as `1`. Some pins can also
be configured to read analog voltage, by employing an internal Analog-Digital 
Converter (ADC for short).

But what does the pin sense when there is nothing connected to it? Well, any
ambient voltage around it! Think of it as a very undefined behavior. For that reason
the inputs can be configured to be tied to high (pull-up) or low (pull-down) voltage
over the high-value resistor. The pin can then read any value that is connected to this pin
and does not have such a high resistance, effectively overriding the default 
pull-up or pull-down.

#### Output

An output in so-called push-pull configuration either sources high voltage when the
output is `1` or drains low voltage when the output is `0`. Without heavy terminology,
you would read `1`/`0` if the output is `1`/`0`.

Output pin can also have other configurations. For example, this MCU can configure pin
to actively drain current only when the output is `0`, and do nothing (think - pin unconnected)
when the output is `1`. Such configuration is called open-drain.

### Voltage

What is called Low and High voltage?

#### Low

Low corresponds to digital value `0`, and it is the so-called "ground" value. The pin
for it is named `GND`.

#### High

High voltage corresponds to `1` and is the voltage at which the MCU is operating. 
In this case, MCU is working at 3.3V even though the USB cable supplies 5V.
There is a small voltage regulator chip on our board that supplies 3.3V voltage
to our MCU.

It is simply easier to talk about `high` and `low` than name the exact voltage values.
It is usually the case that some pins on MCU can read higher voltage (in our case it may
be 5V) than the MCU itself is operating on. That would still be "high" for a digital input
pin. If the pins can _actually_ read higher voltage, well, it's best to [read the datasheet
to find out][mcu-datasheet]. Quick answer: yes, this MCU GPIO ports can do that, 
but not in pull-up/pull-down mode (Section 5.3.13).

### GPIO on our MCU

With this background information, we can start reading section __7.1__ of [the reference manual][mcu]
and take some notes.

We find that ports have registers:

- `GPIOx_CRL`, `GPIOx_CRH` are _configuration_ registers;
- `GPIOx_IDR`, `GPIOx_ODR` _data_ registers;
- `GPIOx_BSRR` _set/reset_ register;
- `GPIOx_BRR` _reset_ register;
- `GPIOx_LCKR` _locking_ register.

At this point it is unclear what exactly they all mean, but we should
remember this terminology and continue reading.

The manual lists possible pin modes, which we can understand now:

- Input floating 
- Input pull-up
- Input-pull-down
- Analog
- Output open-drain
- Output push-pull
- Alternate function push-pull
- Alternate function open-drain

We learn that there are some alternate configurations, which we probably won't need.

We find this sentence:

```plain
Each I/O port bit is freely programmable, 
however the I/O port registers have 
to be accessed as 32-bit words
```

We know that we have added `stm32f1` crate with an API to this device,
and we hope we can avoid thinking of exact/relative address locations or how the 
registers have to be accessed. This simplifies the situation a bit.

```plain
The purpose of the GPIOx_BSRR and GPIOx_BRR 
registers is to allow atomic read/modify 
accesses to any of the GPIO registers.
```

We learn that `GPIOx_BSRR` and `GPIOx_BRR` can be used for modifying
pin values (looks relevant!) without data races (maybe Rust can help here?).

Then we have bunch of diagrams in `Figure 11` and `Figure 12`, with electrical schema
of a port bit. Good to know it exists.

`Table 16` - "Port configuration table" and `Table 17` - "Output MODE bits" looks
like useful tables, but we don't have enough information to make sense of them yet.

#### Section 7.1.1, General-purpose I/O (GPIO)

```plain
During and just after reset, the alternate 
functions are not active and the I/O ports are 
configured in Input Floating mode 
(CNFx[1:0]=01b, MODEx[1:0]=00b).
```

We learn that:

- Ports are in input mode at start
- The default mode configuration is `CNFx[1:0]=01b, MODEx[1:0]=00b`, and we are supposed
  to understand this. Hopefully we will, when we get to it.
  
```plain
The JTAG pins are in input PU/PD after reset: ...
```

We also learn that pins `PA15`, `PA14`, `PA13` and `PB4` are reserved for `JTAG`, and are 
used for our debugging. Good to know. 

```plain
When configured as output, the value written 
to the Output Data register (GPIOx_ODR) is 
output on the I/O pin.
```

Looks like `ODR` register is another way to output values. At this point it is unclear
why we have 2 ways (another being `BSRR` and `BRR` we saw earlier).

```plain
The Input Data register (GPIOx_IDR) captures
the data present on the I/O pin at every 
APB2 clock cycle. 
```

In passing, we learn that there is `APB2` clock which is driving
the data reads on I/O pins. It's almost too easy to miss this...
Because at start, this clock is off. We have to turn it on for each port
we wish to use.

#### Section 7.1.2, Atomic bit set or reset

```plain
There is no need for the software to disable 
interrupts when programming the GPIOx_ODR 
at bit level: it is possible to modify only 
one or several bits in a single atomic APB2 
write access.
```

Here we learn that `ODR` can be used to modify multiple bits, and
can do that in atomic access.

```plain
This is achieved by programming to ‘1’ the 
Bit Set/Reset Register (GPIOx_BSRR, or for 
reset only GPIOx_BRR) to select the bits to 
modify. 
```

It took me some time to figure it out, but this basically means that we
write nothing _but_ `1` to `GPIOx_BSRR` or `GPIOx_BRR`. Writing `1` to 
"Set" register sets the value to `1`, writing `1` to "Reset" resets it to `0`,
and writing `0` does nothing.

Again, why so many ways to do that? [I refer you to an excellent answer here](https://electronics.stackexchange.com/questions/99095/stm32f0-gpiox-odr-vs-gpiox-bsrr),
but long story short is that we should use `ODR` to change multiple values at once,
and we should use `BSRR` or `BRR` to modify individual bits (without the need
to read other bits!).

#### Section 7.1.8, Output configuration

We are interested in output, so we skip `7.1.3`, `7.1.4` "alternate functions", 
`7.1.5` "A/F remapping", `7.1.6` "GPIO locking" (interesting, but not now), 
`7.1.7` "input configuration".

There is important point in the list:

```plain
The data present on the I/O pin is sampled 
into the Input Data Register every APB2 
clock cycle
```

So, again, that clock has to be on.

#### Section 7.2, GPIO registers

We skip `7.1.9` "alternate function configuration", `7.1.10` "analog configuration",
`7.1.11` "GPIO configurations for device peripherals" (looks like this would
be useful electrical info if we were using pins for standard communication protocols, such
as SPI/I2C, etc).

This section finally lists what exact values can be written or read from
these GPIO registers.

Sections `7.2.1` and `7.2.2` describe configuration registers `CRL` and `CRH`.

- `CRL` configures port pins from 0 to 7;
- `CRH` configures port pins from 8 to 15.

Each configuration register has two bit sets:

- `MODE`, which selects between Input/Output;
- `CNF`, which configures selected MODE further.

Remember, previously in manual we saw that default pin mode is:
`CNFx[1:0]=01b, MODEx[1:0]=00b`.

We can now see that MODE `00` refers to "Input", and CNF `01` selects
"floating input".

So, if we wanted to change the pin to output, we would choose output MODE (`01` - `11`),
and then output configuration CNF `00` (push-pull mode).

Section `7.2.3` describes `IDR`, the Input Data Register. We won't read anything for now,
but this should be straightforward.

Section `7.2.4`, `ODR`, is the Output Data Register, and we would use it to output
multiple values at once. However, right now we just need to flip some bits,
so we expect the `BSRR` bit flipper would suffice.

Section `7.2.5`, `BSRR`, is our bit flipper.

It has two bit sets:

- `BR`, which resets `ODR` bit to `0` when we write `1` to it.
- `BS`, which sets `ODR` bit to `1` when we write `1` to it.

Section `7.2.5`, `BRR`, is very similar to `BSRR`, but can only reset.

Looks like we have enough information to start writing some bits!

### APB2 clock

We saw few references throughout the GPIO documentation to this APB2 clock,
which has to be enabled for I/O port to work at all.

If we do a quick search in the PDF, we locate the information about it in
`6.3.7` section titled "APB2 peripheral clock enable register", under the "Reset and 
Clock Control" (abbreviated RCC).

We find there that there should be a configuration bit named `IOPCEN` (IO - Port - C - Enable)
which enables port C clock.

### The LEDs

The LEDs are physically connected to `PC8` and `PC9` ports over some resistors.
If there is a high I/O voltage on these pins, the corresponding LEDs should light up.

## Flipping bits with Rust to light up the LEDs

We learned a few things:

- The `APB2` clock for port C needs to be enabled to do anything with port C;
- The port C has to be configured as Output;
- We can then flip some bits to send port C pins `8` and `9` high, lighting up the 
  LEDs connected to pins `PC8` and `PC9`.
  
### `stm32f1` crate documentation

Let's generate and open the docs for our device crate `stm32f1`:

```bash
cargo doc -p stm32f1 --open
```

Inside the documentation, we find `stm32f1::stm32f100` submodule:

![doc image stm32f100](/images/mcu-02/doc-stm32f100.png)

This API is generated with `svd2rust` tool, and 
[inside the svd2rust documentation](https://docs.rs/svd2rust/0.12.0/svd2rust/#peripheral-api)
we can find hints of how to use this API.

Everything starts by getting access to `Peripherals`:

(main.rs, main function)

```rust
let peripherals = stm32f1::stm32f100::Peripherals::take().unwrap();

// continue here
``` 

From the `Peripherals`, we will need `Reset and Clock Control`, or RCC:

```rust
let rcc = peripherals.RCC;

// continue here
```

As well as GPIOC (GPIO C port):

```rust
let port_c = peripherals.GPIOC;

// continue here
```

When we browse the docs, we find that all peripheral types have a pointer
to corresponding `RegisterBlock` type. It refers to a number of registers grouped together. 
For example, the `stm32f1::stm32f100::RCC`
type has a pointer to `rcc::RegisterBlock` (although the `rcc::` is not reflected in
generated documentation):

![doc rcc register block](/images/mcu-02/doc-rcc-register-block.png)

So to find the methods on `RCC`, we can drill down to `rcc::RegisterBlock` type. It contains
`apb2enr` individual register field that has the same name that was in the manual.

All the registers are used to write, modify, read, or reset the bits in them.
Inside the `rcc::APB2ENR` register type, we find methods to do just that:

![doc apb2enr](/images/mcu-02/doc-apb2enr.png)

So, we can call write on `rcc.apb2enr`:

```rust
rcc.apb2enr.write(|w| ???);
```

The `write` method expects a closure that does actual writing, and it gets this reference to
`W` type. It looks like a generic type, but it is not - we can click on it:

![doc write type](/images/mcu-02/doc-write-type.png)

And inside a heap of methods, we find the method that gets us to the bit that
can enable Port C clock:

![doc port c clock enable](/images/mcu-02/doc-port-c-clock-enable.png)

And there we can call the `bit(true)`/`set_bit()` or `bit(false)`/`clear_bit()` functions
to set or unset the bit. Let's do just that:

```rust
rcc.apb2enr.write(|w| w.iopcen().bit(true));

// continue here
```

Similarly, we get to Port C CRH register, and configure pins `8` and `9` to be
in MODE `01` (output) with CNF `00` (push-pull):

```rust
port_c.crh.write(|w| unsafe {
    w
        .mode8().bits(0b01)
        .cnf8().bits(0b00)
        .mode9().bits(0b01)
        .cnf9().bits(0b00)
});
```

There is a bit of unsafe code here, because the `bits` function can theoretically go out
of bounds and flip some bits that it should not have access to.

We can make previous code a bit nicer by introducing some constants:

```rust
const MODE_INPUT: u8 = 0b00;
const MODE_OUTPUT_10MHz: u8 = 0b01;
const MODE_OUTPUT_2MHz: u8 = 0b10;
const MODE_OUTPUT_50MHz: u8 = 0b11;

const CNF_OUTPUT_PUSHPULL: u8 = 0b00;
const CNF_INPUT_FLOATING: u8 = 0b01;

port_c.crh.write(|w| unsafe {
    w
        .mode8().bits(MODE_OUTPUT_10MHz)
        .cnf8().bits(CNF_OUTPUT_PUSHPULL)
        .mode9().bits(MODE_OUTPUT_10MHz)
        .cnf9().bits(CNF_OUTPUT_PUSHPULL)
});

// continue here
```

Finally, we use Port's BSRR "Bit Set" register to flip bits `8` and `9` high:

```rust
port_c.bsrr.write(|w|
    w
        .bs8().set_bit()
        .bs9().set_bit()
);
```

## Wait for it

```gdb
(gdb) target remote :3333
Remote debugging using :3333
0x00000000 in ?? ()
(gdb) load
Loading section .vector_table, size 0x130 lma 0x8000000
Loading section .text, size 0x214 lma 0x8000130
Start address 0x8000130, load size 836
Transfer rate: 5 KB/sec, 418 bytes/write.
(gdb) continue
Continuing.
```

Bam!

![Image of PC8 and PC9 LEDs working](/images/mcu-02/leds-working.jpeg)

## Thoughts

It's not hard to flip some bits once you know which ones. However, it is very
easy to become overwhelmed by the number of things to keep in mind.
Add to the mix that Rust is very new in embedded world, add to the mix that we may be new
to this microcontroller, and the task becomes very daunting.

However, if we persist, we get to the end :)

## Literature

- [Cortex-M Basics][mcu-basics]
- [STM32f1 Reference manual][mcu]
- [cortex-m-rt crate documentation](https://docs.rs/cortex-m-rt/0.4.0/cortex_m_rt/)
- [svd2rust documentation](https://docs.rs/svd2rust/0.12.0/svd2rust)
- [LED switching magic in japaric's older project for VL DISCOVERY board](https://japaric.github.io/vl/src/vl/led.rs.html#31-48)
- [Discover the world of microcontrollers through Rust! (not this board, but very useful)][mcu-world]
- [stm32f100 datasheet][mcu-datasheet]

[mcu-basics]: /resources/mcu-02/cortex-m-basics
[mcu]: http://www.st.com/content/ccc/resource/technical/document/reference_manual/a2/2d/02/4b/78/57/41/a3/CD00246267.pdf/files/CD00246267.pdf/jcr:content/translations/en.CD00246267.pdf
[mcu-world]: https://japaric.github.io/discovery/README.html
[mcu-datasheet]: http://www.st.com/resource/en/datasheet/stm32f100cb.pdf
