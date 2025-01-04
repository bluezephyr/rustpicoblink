#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use embedded_hal::digital::{OutputPin, PinState};
use panic_halt as _;
use rp2040_boot2;
use rp2040_hal as hal;
use rtt_target::{rprintln, rtt_init_print};

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

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

    let mut p = hal::pac::Peripherals::take().unwrap();
    let cp = hal::pac::CorePeripherals::take().unwrap();

    let sio = hal::Sio::new(p.SIO);
    let pins = hal::gpio::Pins::new(p.IO_BANK0, p.PADS_BANK0, sio.gpio_bank0, &mut p.RESETS);
    let mut led = pins.gpio22.into_push_pull_output();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(p.WATCHDOG);

    // Configure the clocks
    let _ = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        p.XOSC,
        p.CLOCKS,
        p.PLL_SYS,
        p.PLL_USB,
        &mut p.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // Configure the SysTick
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::External);
    syst.set_reload(hal::pac::SYST::get_ticks_per_10ms() * 50);

    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    rprintln!("{} ticks per ms (times 50 modifier applied)", hal::pac::SYST::get_ticks_per_10ms());

    loop {
        let _ = led.set_state(PinState::from(LED_ON.load(Ordering::Relaxed)));
    }
}
