# Rust Pico Blink

Blink a LED on the Pi Pico using Rust

## Minimal implementation

A minimal implementation that runs on the Pi Pico is very similar to the "standard"
minimal [implementation](https://docs.rust-embedded.org/book/start/qemu.html#program-overview),
but there are a few specifics. The details are specified on the
[RP2040 Project Template](https://github.com/rp-rs/rp2040-project-template) page, but the
most important parts are as follows:

* Since there is a boot loader on the chip, it must be initialized. This is done using the
  instructions on
  [Boot2](https://github.com/rp-rs/rp2040-project-template?tab=readme-ov-file#notes-on-using-rp2040_boot2)
* The memmap file (memory.x) must take into consideration the boot loader


### Memmap

```memory.x
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;
```

## Documentation

* [Rust embedded book](https://docs.rust-embedded.org/book/)
* [rp-rs](https://github.com/rp-rs)
* [pico microcontrollers](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html)
* [RP2040 datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
* [GPIO setup](https://embedded-rust-101.wyliodrin.com/docs/lab/02)

