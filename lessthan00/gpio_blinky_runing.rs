//! Blinks several LEDs stored in an array
//! PC13->LED
//! PA1 ->LED2
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
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap();

    let mut led_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();

    // Create an array of LEDS to blink
    let mut leds = [
        gpioa.pa0.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa3.into_push_pull_output(&mut gpioa.crl).erase(),
        // gpioa.pa4.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa6.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase(),
        gpiob.pb0.into_push_pull_output(&mut gpiob.crl).erase(),
        gpiob.pb1.into_push_pull_output(&mut gpiob.crl).erase(),
    ];

    // Initialize all LEDs to off
    for led in leds.iter_mut() {
        led.set_low();
    }
    
    let mut current_led = 0;  // 当前 LED 索引（0~6，对应 PA1~PA7）

    loop {
        led_pc13.toggle();              
        block!(timer.wait()).unwrap();

        // 关闭所有 LED
        for led in leds.iter_mut() {
            led.set_low();
        }

        leds[current_led].set_high();

        current_led = (current_led + 1) % leds.len();
    }
}

// cargo check --features stm32f103 --example my_gpio_blinky_runing
// cargo run --features stm32f103 --example my_gpio_blinky_runing
