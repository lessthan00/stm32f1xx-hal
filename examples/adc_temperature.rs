#![deny(unsafe_code)]      // 禁用所有不安全代码
#![no_main]                // 不使用标准 main 函数入口
#![no_std]                 // 不使用标准库（适用于嵌入式环境）

// 导入依赖库
use panic_semihosting as _; // 使用 semihosting 处理 panic 信息输出

use cortex_m_rt::entry;     // 提供 entry 宏来定义入口函数
use stm32f1xx_hal::{pac, prelude::*}; // HAL 库和外设访问模块

use cortex_m_semihosting::hprintln; // 用于通过 semihosting 输出日志信息

#[entry]                    // 标记 main 函数为程序入口点
fn main() -> ! {
    // 获取外设访问权限
    let p = pac::Peripherals::take().unwrap(); // 获取对寄存器的独占访问权
    let mut flash = p.FLASH.constrain();        // 转换 FLASH 外设为配置结构体
    let rcc = p.RCC.constrain();                // 转换 RCC 外设为配置结构体

    // 配置系统时钟
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())       // 使用外部晶振 HSE，频率为 8MHz
        .sysclk(56.MHz())       // 设置系统主频为 56MHz
        .pclk1(28.MHz())        // 设置 APB1 总线频率为 28MHz
        .adcclk(14.MHz())       // 设置 ADC 时钟为 14MHz
        .freeze(&mut flash.acr); // 冻结配置并应用到硬件（需要FLASH访问控制）

    /* 
    // 可选：手动设置 PLL 倍频分频参数
    let clocks = rcc.cfgr.freeze_with_config(rcc::Config {
        hse: Some(8_000_000),   // HSE 晶振频率 8 MHz
        pllmul: Some(7),         // PLL 倍频系数 7 (8 * 7 = 56 MHz)
        hpre: rcc::HPre::DIV1,   // AHB 分频为 1
        ppre1: rcc::PPre::DIV2,  // APB1 分频为 2 (56 / 2 = 28 MHz)
        ppre2: rcc::PPre::DIV1,  // APB2 分频为 1
        usbpre: rcc::UsbPre::DIV1_5, // USB 分频为 1.5 (56 / 1.5 = 37.33 MHz)
        adcpre: rcc::AdcPre::DIV2,   // ADC 分频为 2 (56 / 2 / 2 = 14 MHz)
    }, &mut flash.acr);
    */

    // 输出当前时钟信息
    hprintln!("sysclk freq: {}", clocks.sysclk()); // 打印系统主频
    hprintln!("adc freq: {}", clocks.adcclk());     // 打印 ADC 时钟频率

    // 初始化 ADC
    let mut adc = p.ADC1.adc(&clocks); // 启动 ADC1 并传入当前时钟配置

    // 循环读取温度传感器数据
    loop {
        let temp = adc.read_temp(); // 读取内部温度传感器值

        hprintln!("temp: {}", temp); // 打印温度传感器原始值
    }
}