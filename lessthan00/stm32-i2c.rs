//! Get state and start/stop motor on a Jrk with a STM32f1xx
//!
//! In this example, the Jrk is connected on I2C:
//! SCL is on PB8, SDA on PB9
//!
//! PA9 & PA10 are used as a serial monitor

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use nb::block;
use panic_halt as _;
use stm32f1xx_hal::{i2c, pac, prelude::*, serial, timer::Timer};

use jrk_g2_rs::{JrkG2, JrkG2BlockingI2c};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Initialize USART1 on PA9 & PA10 for monitoring
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = serial::Serial::usart1(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        serial::Config::default().baudrate(115_200.bps()),
        clocks,
        &mut rcc.apb2,
    );
    let (mut tx, _rx) = serial.split();
    writeln!(tx, "serial monitor initialized").unwrap();

    // Initialize connexion to the Jrk: I2C1 on PB8 & PB9 and USART3 on PB10 & PB11
    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    let jrk_tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let jrk_rx = gpiob.pb11;

    let i2c = i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        i2c::Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: i2c::DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        10000,
        100,
        10000,
        10000,
    );

    let jrk_ser = serial::Serial::usart3(
        dp.USART3,
        (jrk_tx, jrk_rx),
        &mut afio.mapr,
        serial::Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcc.apb1,
    );

    let (_jrk_tx, _jrk_rx) = jrk_ser.split();

    let mut jrk = JrkG2BlockingI2c::new(i2c);
    writeln!(tx, "Jrk initialized on stm32 by i2c").unwrap();

    loop {
        if let Err(e) = jrk.stop_motor() {
            write!(tx, "I2cError: {:?}", e).ok();
        }
        block!(timer.wait()).unwrap();
        jrk.show_vars(&mut tx).ok();

        if let Err(e) = jrk.set_target(1400) {
            write!(tx, "I2cError: {:?}", e).ok();
        }
        block!(timer.wait()).unwrap();
        jrk.show_vars(&mut tx).ok();
    }
}
