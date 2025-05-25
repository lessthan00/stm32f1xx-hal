//! i2C
//! PB6->SCL
//! PB7->SDA

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
    timer::Timer,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};



#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();
    let mut dwt = cp.DWT;
    dwt.enable_cycle_counter();

    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    let mut scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let mut sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    


    // let mut i2c = dp.I2C1.i2c(
    //     (scl, sda), 
    //     Mode::Fast {
    //         frequency: 400.kHz(),
    //         duty_cycle: DutyCycle::Ratio16to9,
    //     }, 
    //     &clocks);
    
    // let mut i2c = BlockingI2c::new(
    // dp.I2C1,
    // (scl, sda),
    //  Mode::Fast {
    //         frequency: 400.kHz(),
    //         duty_cycle: DutyCycle::Ratio16to9,
    //     },
    // &clocks,
    // 1000,   // start timeout
    // 10,     // retries
    // 1000,   // addr timeout
    // 1000    // data timeout
    // );

    // let mut i2c = BlockingI2c::new(
    //     dp.I2C1,
    //     (scl,sda),
    //     Mode::Fast {
    //         frequency: 400.kHz(),
    //         duty_cycle: DutyCycle::Ratio2to1,
    //     },
    //     &clocks,
    //     1000,
    //     10,
    //     1000,
    //     1000,
    // );

    let mut i2c = dp
    .I2C1
    //.remap(&mut afio.mapr) // add this if want to use PB8, PB9 instead
    .blocking_i2c(
        (scl, sda),
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio16to9,
        },
        &clocks,
        1000,
        10,
        1000,
        1000,
    );
    hprintln!("done setup()");
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    hprintln!("done interface");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    hprintln!("done Hello world!");

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    hprintln!("enter loop {}");
    loop {}
}

//cargo check --features "stm32f103,bme280,rtic" --example my_i2c
//cargo run --features "stm32f103,bme280,rtic" --example my_i2c