//! Blinks several LEDs stored in an array
//! PC13->LED
//! PB10->buzzer
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIO peripherals
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap();

    let mut led_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();
    let mut buzzer = gpiob.pb10.into_push_pull_output(&mut gpiob.crh).erase();

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led_pc13.toggle();
        buzzer.toggle();
    }
}

// cargo check --features stm32f103 --example my_gpio_buzzer
// cargo run --features stm32f103 --example my_gpio_buzzer
