//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use rtt_target::{rprintln, rtt_init_print};
use embedded_hal::pwm::SetDutyCycle;

use cortex_m_rt::entry;
use panic_halt as _;

use hal::pac;
use rp2040_hal as hal;
use rp2040_hal::Clock;


use rp2040_boot2;

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// Min and max value for the PWM value
const LOW: u16 = 0;
const HIGH: u16 = u16::MAX;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = hal::clocks::init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Init PWMs
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM 3A (See RP2040 datasheet for details)
    let pwm = &mut pwm_slices.pwm3;
    pwm.set_ph_correct();
    pwm.enable();

    // Output channel A on PWM3 to the LED pin
    let channel = &mut pwm.channel_a;
    channel.output_to(pins.gpio22);

    rprintln!("Max duty cycle: {}", channel.max_duty_cycle());

    loop {
        rprintln!("Ramp up!");
        for i in (LOW..=HIGH).step_by(10) {
            delay.delay_us(1);
            let _ = channel.set_duty_cycle(i);
        }

        rprintln!("Ramp down!");
        for i in (LOW..=HIGH).rev().step_by(8) {
            delay.delay_us(2);
            let _ = channel.set_duty_cycle(i);
        }

        delay.delay_ms(400);
    }
}
