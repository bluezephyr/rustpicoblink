#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rp2040_boot2;
use rp2040_pac::Peripherals;
use rtt_target::{rprintln, rtt_init_print};

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    const LED: usize = 22;
    let p = Peripherals::take().unwrap();

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

    let mut on = true;
    loop {
        if on {
            // Enable the LED
            sio.gpio_out_set().write(|w| unsafe { w.bits(1 << LED) });
        } else {
            // Disable the LED
            sio.gpio_out_clr().write(|w| unsafe { w.bits(1 << LED) })
        }

        // Wait for a while
        for _ in 0..10_000 {
            nop();
        }

        // Toggle the wanted LED state
        on = !on;
        rprintln!("Loop!");
    }
}
