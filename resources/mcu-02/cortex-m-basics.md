---
layout: page
permalink: /resources/mcu-02/cortex-m-basics
---

[Original document](http://pramode.net/fosstronics/basics.txt).

## ARM Cortex-M3 and STM32 Basics

### Objective

All the datasheets and reference manuals which describe a processor architecture in detail, 
taken together, may run into thousands of pages. Very often, the systems programmer who 
wishes to write / port something like a real time OS will need only a core set of 
information regarding the:

  * The processor memory map
  * Reset behaviour
  * System clock
  * Interrupts
  * Timer
  * Basic instruction set 
  * Toolchain

This wiki page is an experiment in gathering just the amount of information about an 
ARM Cortex-M3 based processor which will help us understand the working of a simple 
RTOS (in this case, [FreeRTOS](http://www.freertos.org)).

### About ARM Cortex-M3

Check out the [the official description of the Cortex-M3 core](http://www.arm.com/products/CPUs/ARM_Cortex-M3.html). 
An [ARM whitepaper](http://www.arm.com/pdfs/IntroToCortex-M3.pdf) is also available. 
From a programmer's point of view, some of the interesting points about this new architecture
are:

  * A new Thumb-2 instruction set with 16 bit and 32 bit instructions.
  * Hardware stack manipulation - registers are pushed/popped automatically 
  during ISR entry/exit
  * A SysTick timer integrated into the processor core - useful for writing RTOS code.

### About STM32F103RBT6

[Luminary Micro](http://luminarymicro.com) was the first company to bring out Cortex-M3 
based processors. This was followed by [STMicro bringing out STM32](http://www.st.com/stm32). 
Cortex-M3 based processors have been announced by 
[NXP](http://www.nxp.com/news/content/file_1478.html), but they are not yet available 
in the market.

The [STM32F103RB](http://www.st.com/mcu/devicedocs-STM32F103RB-110.html) device comes with 
128Kb of flash and 20Kb RAM. An innovative evaluation board, the 
[STM32 Primer](http://www.stm32circle.com), is based on the STM32F103RB.


### Memory Map

The Cortex-M3 has a linear 32 bit address space. The first 1Gb is split evenly between 
code and data; code (flash) is from `0x0` to `0x1fffffff` and data (SRAM) is from `0x20000000` 
to `0x3fffffff`. Peripherals are mapped to locations `0x40000000` to `0x5fffffff`.

```plain
Note: The STM32F10xxx processor's flash is mapped to location 0x08000000; but it 
can also be accessed from location 0x0.
```

There is a special `System control space` from `0xe000e000` to `0xe000efff`. This 
region holds, among other things, registers related to the Nested Vectored 
Interrupt controller and the SysTick timer.

Cortex-M3 processors are little endian.

### Interrupt Vectors

The interrupt vector table starts at address `0x4` (the cortex processor uses 
the 4 byte value at address `0x0` to initialize the stack pointer). 
Each entry is 4 bytes long and should contain the address of the routine 
to be executed. The table starts with the reset vector at location `0x4`.
 
For the cortex core to start executing code, the only requirement is 
initializing the first two locations properly. The stack pointer can be made to 
point to the top of available SRAM and the reset vector should point to the code 
which is to be executed.

### GPIO Ports

GPIO ports are configured as `input floating` upon reset and alternate functions are 
not enabled. Each port will have upto 16 pins.

Each GPIO port has two 32 bit configuration registers - `GPIOx_CRL` and `GPIOx_CRH`, 
two 32 bit data registers, `GPIOx_IDR` and `GPIOx_ODR`, a 32 bit set-reset register, 
`GPIO_xBSRR`, a 16 bit reset register, `GPIOx_BRR` and a 32 bit locking register, `GPIOx_LCKR`.

Each GPIO pin can be configured as one of:

  * Input floating
  * Input pull-up
  * Input pull-down
  * Analog input
  * Output open-drain
  * Output push-pull
  * Alternate function push-pull
  * Alternate function open-drain

### Configuring the GPIO ports

Each I/O pin of a GPIO port has four configuration bits - `GPIOx_CRL` holds configuration 
bits for 8 pins and `GPIOx_CRH` holds the configuration bits for the remaining 8 pins.

The four configuration bits are divided into two MODE bits and two CNF bits - 
with MODE bits occupying lower bit positions. For example, in the case GPIO port B, 
bits 0 and 1 of `GPIOB_CRL`` a`re the MODE bits and bits 2 and 3 are the CNF bits for pin 0. 
The MODE bits can have the following values:

  * `00` - Input mode (reset state)
  * `01` - Output mode (max 10MHz speed)
  * `10` - Output mode (max 2MHz speed)
  * `11` - Output mode (max 50MHz speed)

The CNF bit values are inte`rp`reted differently according to the MODE. If MODE is input, 
the CNF bit values are interpreted as:

  * `00` - Analog input
  * `01` - Floating input
  * `10` - Input with pull-up/pull-down
  * `11` - reserved`

`If MODE is output, the CNF bit values are interpreted as:

  * `00` - General purpose output push-pull
  * `01` - General purpose output open-drain
  * `10` - Alternate function output push-pull
  * `11` - Alternate function output open-drain

Here is a link [which explains difference between push-pull and open-drain](http://www.edaboard.com/ftopic252407.html) 
configurations. Normally, to drive LED's, we will configure the output pins as push-pull.
 
### The Bit Set/Reset register

Bits 0 to 15 of `GPIOx_BSRR` are used to "set" the corresponding port pin - writing a 
0 to these bits have no effect while writing a 1 will set the port pin. 
Bits 16 to 31 are used to "reset" the corresponding port pin - writing a 
0 to these bits have no effect while writing a 1 will reset the port pin.

### Clocking

Upon reset, an internal RC oscillator of 8MHz frequency is used as the system clock. 
For better precision, an external crystal can be used as the clock source. 
An internal PLL can be used to multiply the base clock (either internal or external) 
to the maximum system clock frequency of 72MHz.

Clocks to all peripherals (except SRAM and flash) are disabled on reset - so 
if we wish to use say the GPIO ports, it's corresponding `peripheral clock` has to be enabled.

### Enabling GPIO Clock 

No peripheral unit will work without its clock being enabled. Lighting up an LED 
on a GPIO port requires that the corresponding peripheral clock is enabled.
 
Bits 2 to 8 of the RCC_APB2ENR register (address `0x40021018`) control the clocks 
to IO ports A to G.  

### Boot Modes

The STM32F10xxx has 3 different boot modes - mode selection is done using two pins 
BOOT0 and BOOT1. 

```plain
| BOOT 1   |  BOOT 0   |   Boot Mode          |    
|   x      |    0      |   Main Flash memory  |
|   0      |    1      |   System memory      |
|   1      |    1      |   Embedded SRAM      |
```

### SysTick Timer

A timer which generates interrupts periodically is at the heart of most real-time 
operating systems. The Cortex-M3 core has a very interesting feature - 
it provides a `SysTick` timer integrated into the CPU core which can be used for 
providing the RTOS `tick`. Because this timer is part of the processor core, 
all Cortex-M3 devices, irrespective of the vendor, will have this feature. 

```plain
Note: the SysTick timer may not be properly documented in the vendor's processor 
reference manual - it is better to check out the official ARM documentation.
```

The SysTick timer's operation is very simple. The counter starts counting down from 
the value specified in the `reload` register (address `0xe000e014`) - when the value 
reaches 0, it optionally generates an interrupt and once again starts counting down 
from the initial value specified in the `reload` register. 

The SysTick interrupt is by default enabled in the NVIC (nested vectored interrupt 
controller) - we only have to enable it in the timer by setting the SysTick control 
and status register's (address `0xe000e010`) D1'th bit to 1. Setting the D0'th bit 
of the same register enables the timer. Setting the D2'th bit to 1 chooses the CPU 
clock as the clock source of the timer. 

```plain
Note: A bit of confusion here - is it the cpu clock or the AHB clock? 
What if the bit is zero?
```