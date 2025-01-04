# Rust Pico Blink

Blink a LED on the Pi Pico using Rust

## Hardware setup

The hardware needed for the setup is as follows:

* [Raspberry PI Pico](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html#pico-1-family)
* A LED connected to GPIO 22 (Pin 28 on the
  [Pinout](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html#pinout-and-design-files-3))
via a resistor (220 Ohm).
* A [Raspberry Pi Debug
  Probe](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html#about-the-debug-probe)


## Software download

There are a few diffent ways to download the binary to the Raspberry Pi Pico; using a
debug-probe or by loading a UF2 over USB. Details can be found on the [RP2040 project
template page](https://github.com/rp-rs/rp2040-project-template?tab=readme-ov-file#running)

### Using a debug Probe

* Download the binary with `cargo embed` using the configuration in the `Embed.toml` file

or

* set the runner in the `config.toml` file to `runner = "probe-rs run --chip RP2040
  --protocol swd"`
* Download the binary using `cargo run`

### Using UF2 over USB

This method requires that `elf2uf2-rs` has been installed. Use `cargo install elf2uf2-rs
--locked` to do this.

* Set the runner in the `config.toml` file to `runner = "elf2uf2-rs -d"`
* Boot the Raspberry Pi (RP2040) into USB Bootloader mode. On the Raspberry Pi, this is
  done by holding the BOOTSEL button pressed when powering on the chip.
* Mount the drive.
* Download the binary using `cargo run`

### Using 'drag-and-drop'

Create an `uf2` file that can be 'dragged-and-dropped' to the Pi Pico using a standard
file browser. The Pi Pico acts as an USB drive and will automatically install the `uf2`
file.

* Use `cargo build` to build the binary
* Convert the `ELF` file to an `uf2` file with `elf2uf2-rs
  ./target/thumbv6m-none-eabi/debug/rustpicoblink`. An `uf2`-file will be created in the
  same folder.
* Boot the Raspberry Pi (RP2040) into USB Bootloader mode. On the Raspberry Pi, this is
  done by holding the BOOTSEL button pressed when powering on the chip.
* Mount the drive and open the Pi Pico in a file explorer.
* drag-and-drop the `uf2` file to the Pi Pico

## Minimal implementation

A minimal implementation that runs on the Pi Pico is very similar to the "standard"
minimal
[implementation](https://docs.rust-embedded.org/book/start/qemu.html#program-overview),
but there are a few specifics. The details are specified on the [RP2040 Project
Template](https://github.com/rp-rs/rp2040-project-template) page, but the most important
parts are as follows:

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

## GPIO

To turn on a LED, the corresponding GPIO pin needs to be configured. The needed operations
for the configuration is listed below. Details can be found in the[RP2040
  datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf).

* The `IO Bank0 peripheral` needs to be enabled (disabled at startup). This
  is done by accessing the `RESETS` registers and writing a 0 to the `IO_BANK0`
  bit. We also need to wait for the reset to take effect. The `RESET_DONE` register
  can be read to check the status. See section *2.14. Subsystem Resets* for details.
* The function (`FUNCSEL`) for the GPIO must be set as `SIO` (F5) using the GPIO's `CTRL`
  register to control the GPIO using the single-cycle IO (SIO) block. See section *2.19.2.
  Function Select* for details.
* Enable output for the PIN. Use the SIO registers `GPIO_OE` to control the
  configuration. See *2.3.1.2. GPIO Control* for details.

When the pin has beed configured, the output can be controlled using the SIO registers
`GPIO_OUT_SET` and `GPIO_OUT_CLR`.

## SysTick

The `SysTick` exception is available in the `cortex-m-rt` crate and exposed in the `RP2040
PAC` under `CorePeripherals`. Configure using the following:

```rs
let cp = CorePeripherals::take().unwrap();
let mut syst = cp.SYST;

// Use either Core or External source
syst.set_clock_source(SystClkSource::Core);

// Value dependent on the clock source
syst.set_reload(1_500_000);

// Start
syst.clear_current();
syst.enable_counter();
syst.enable_interrupt();
```

Handle the interrupt in an interrupt handler.

```rs
#[exception]
fn SysTick() {
  ...
}
```

For more information, see [Exceptions](https://doc.rust-lang.org/beta/embedded-book/start/exceptions.html)
in "The Embedded Rust Book".

## Sharing data between IRQ and main

In order for the timer interrupt to do something interesting, it must be able to address
the same data as the data in the main function. The "C" way is to creata a `static
volatile` global variable that can be read and written by both the IRQ and the main
function. This is also possible in Rust, but requires that these accesses are `unsafe`.
There are a few different ways to avoid this; see
[Concurrency](https://doc.rust-lang.org/beta/embedded-book/concurrency/index.html) for
an overview of the alternatives.

### Atomic access

An `AtomicBool` is used to allow access to a shared global variable without using unsafe
and without the need to disable interrupts. The `swap` method is not supported since
thumbv6m only provide `load` and `store` operations. No support for CAS (Compare and
Swap) operations, such as `swap`, etc.

`static LED_ON: AtomicBool = AtomicBool::new(true);`

In the IRQ, the following can be used. The complete write is not atomic, but in this
scenario that is not important since the main function only reads the data.

`LED_ON.store(!LED_ON.load(Ordering::Relaxed), Ordering::Relaxed);`

See [Atomics](https://doc.rust-lang.org/nomicon/atomics.html) for more information.

### Mutex

An alternative to the atomic access is to use a Mutex. This will wrap the boolean in a
Mutex that uses a critical section whenever the value is to be accessed.

```rs
 // Use a Mutex to protect the shared variable
 static LED_ON: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

#[exception]
fn SysTick() {
    interrupt::free(|cs| LED_ON.borrow(cs).set(!LED_ON.borrow(cs).get()));
}

```

See [Mutexes](https://doc.rust-lang.org/book/ch16-03-shared-state.html) and [Embedded
Mutexes](https://doc.rust-lang.org/beta/embedded-book/concurrency/index.html#mutexes) for
more information about Mutexes.

## Documentation

* [Rust embedded book](https://docs.rust-embedded.org/book/)
* [The Embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)
* [The Rustonomicon](https://doc.rust-lang.org/nomicon/intro.html)
* [rp-rs](https://github.com/rp-rs)
* [rp-hal](https://github.com/rp-rs/rp-hal/)
* [pico microcontrollers](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html)
* [RP2040 datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
* [GPIO setup](https://embedded-rust-101.wyliodrin.com/docs/lab/02)

