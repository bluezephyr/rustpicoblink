#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use panic_halt as _;
use rp2040_boot2;
use rp2040_pac::{CorePeripherals, Peripherals};
use rtt_target::{rprintln, rtt_init_print};

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// Use an AtomicBool to allow access to the shared variable without using unsafe and without
// disabling interrupts.
static LED_ON: AtomicBool = AtomicBool::new(true);

#[exception]
fn SysTick() {
    // Toggle the wanted LED state
    // The `swap` method is not supported since thumbv6m only provide `load` and `store`
    // operations. No support for CAS (Compare and Swap) operations, such as `swap`, etc.
    LED_ON.store(!LED_ON.load(Ordering::Relaxed), Ordering::Relaxed);
    rprintln!("Tick!");
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    const LED: usize = 22;
    let p = Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();

    // Create a variable for the RESETS registers and clear bit 5 (IO_BANK0). Then wait
    // for the reset to take effect.
    let reset = p.RESETS;
    reset
        .reset()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 5)) });
    while reset.reset_done().read().bits() & (1 << 5) == 0 {}

    // Write the value 5 (SIO) to the FUNCSEL to be able to control the GPIO using the
    // SIO block. (Can be improved by using read, modify, write sequence to avoid that
    // other pins are affected).
    let io_bank0 = p.IO_BANK0;
    io_bank0
        .gpio(LED)
        .gpio_ctrl()
        .write(|w| unsafe { w.bits(5) });

    // Enable output for the pin
    let sio = p.SIO;
    sio.gpio_oe_set().write(|w| unsafe { w.bits(1 << LED) });

    // Configure the SysTick
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(1_500_000);

    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    loop {
        if LED_ON.load(Ordering::Relaxed) {
            // Enable the LED
            sio.gpio_out_set().write(|w| unsafe { w.bits(1 << LED) });
        } else {
            // Disable the LED
            sio.gpio_out_clr().write(|w| unsafe { w.bits(1 << LED) })
        }
    }
}
