//! 作者 lessthan0
//! 代码源于ai
//! 硬件和接线参考[STM32入门教程-2023版 细致讲解 中文字幕](https://www.bilibili.com/video/BV1th411z7sn/?vd_source=188a5e02d520f745e2a0cd650b30aa4b)
//! 如果需要硬件,在淘宝购买即可,这里的模块都是很简单的,没有什么正版盗版的说法.
//! 江科大的所谓正版,你支持可以购买,但这并没有额外的好处(因为事实上没任何一个硬件是他生产的)
//! 除非你确实依赖版权过活(所以必须维护版权).

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

    // 初始化 GPIOA 并分离控制寄存器
    let mut gpioa = dp.GPIOA.split();

    // 将 PA0 到 PA7 配置为推挽输出
    let mut leds = [
        // GPIOA 0 推挽输出 0~7->crl 8~15->crh 多次访问同一个位 erase()檫除
        gpioa.pa0.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa3.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa4.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa5.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa6.into_push_pull_output(&mut gpioa.crl).erase(),
        gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase(),
    ];

    // 设置 SysTick 定时器，每 200ms 触发一次（5Hz）
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap(); // 5Hz = 每秒5次更新

    // 流水灯逻辑
    let mut current = 0;

    // Wait for the timer to trigger an update and change the state of the LED
    // 等待定时器触发更新并更改LED的状态
    loop {
        block!(timer.wait()).unwrap(); // 等待定时器触发

        // 关闭所有 LED
        for led in &mut leds {
            led.set_low();
        }
        // 打开当前 LED
        leds[current].set_high();
        // 更新索引
        current = (current + 1) % leds.len();
    }
}