//! Blinks an LED
//! LED 闪烁
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//! 这假设一个LED连接到pc13，就像在蓝色药丸板上一样。
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.
//! 注意：如果没有额外的硬件，PC13不应用于驱动LED，请参阅参考手册的第5.1.2页以获取解释。这不是蓝色药丸的问题。

#![deny(unsafe_code)]   // 禁止使用不安全代码
#![no_std]              // 不使用标准库
#![no_main]             // 不使用main函数

use panic_halt as _;    // 使用panic_halt库

use nb::block;          // 使用nb库的block函数

use cortex_m_rt::entry; // 使用cortex_m_rt库的entry宏
use stm32f1xx_hal::{pac, prelude::*, timer::Timer}; // 使用stm32f1xx_hal库的pac、prelude和timer模块

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    // 获取来自cortex-m crate的核心外设的访问权限
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    // 从外设访问crate获取对设备特定外设的访问权限
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // 获取原始flash和rcc设备的所有权，并将它们转换为相应的
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // 释放系统中所有时钟的配置，并将冻结的频率存储在
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    // 获取GPIOC外设
    let mut gpioc = dp.GPIOC.split();

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    // 将gpio C引脚13配置为推挽输出。将crh寄存器传递给函数以配置端口。对于引脚0-7，应改为传递crl。
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    // 将syst定时器配置为每秒触发一次更新
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    // Wait for the timer to trigger an update and change the state of the LED
    // 等待定时器触发更新并更改LED的状态
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    }
}