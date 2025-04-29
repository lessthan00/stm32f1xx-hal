//! ADC 接口使用 DMA 接收数据传输测试程序

#![allow(clippy::empty_loop)] // 允许空循环（防止编译器警告）
#![no_main] // 不使用标准入口函数 main()
#![no_std]  // 不使用标准库

// 导入 panic_halt，当发生 panic 时停机
use panic_halt as _;

// 使用 cortex_m 提供的汇编指令和单例机制
use cortex_m::{asm, singleton};

// 使用 cortex-m-rt 的 entry 宏来定义程序入口点
use cortex_m_rt::entry;

// 引入 STM32F1xx HAL 库的相关模块
use stm32f1xx_hal::{adc, pac, prelude::*};

// 使用 entry 宏标记 main 函数为程序入口
#[entry]
fn main() -> ! {
    // 获取底层外设权限（take() 只能调用一次）
    let p = pac::Peripherals::take().unwrap();

    // 获取并配置 Flash 控制器
    let mut flash = p.FLASH.constrain();

    // 获取并配置 RCC（系统时钟控制）
    let rcc = p.RCC.constrain();

    // 设置系统时钟，并配置 ADC 时钟源
    // 将 ADC 时钟设置为 2MHz，通过 PCLK2 分频获得（PCLK2 / 6，因为 STM32F1 ADC 最高支持 14MHz）
    let clocks = rcc.cfgr.adcclk(2.MHz()).freeze(&mut flash.acr);

    // 获取 DMA1 通道 1 的所有权（用于 ADC DMA 传输）
    let dma_ch1 = p.DMA1.split().1;

    // 初始化 ADC1 外设
    let adc1 = adc::Adc::new(p.ADC1, &clocks);

    // 初始化 GPIOA 外设
    let mut gpioa = p.GPIOA.split();

    // 配置 PA0 为模拟输入引脚（用于 ADC 采样）
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    // 将 ADC 配置为使用 DMA 模式进行采样，通道为 ch0，DMA 通道为 ch1
    let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);

    // 创建一个大小为 8 的缓冲区，用于存储 ADC 采集的数据（类型为 u16）
    // 使用 singleton! 确保该缓冲区在内存中是唯一的、静态分配的
    let buf = singleton!(: [u16; 8] = [0; 8]).unwrap();

    // 启动 ADC 和 DMA 转换，开始读取数据到 buf 中
    // read() 方法返回一个 RxDma 结构体
    // wait() 方法会阻塞等待直到整个 DMA 传输完成
    // 返回值 (_buf, adc_dma) 包括：更新后的缓冲区以及 ADC DMA 的结构体
    let (_buf, adc_dma) = adc_dma.read(buf).wait();

    // 此处插入断点，可用于调试查看数据是否已正确填充到 _buf 中
    asm::bkpt();

    // 拆分 AdcDma 结构体，恢复 ADC 到非 DMA 模式
    // 返回值：
    // - _adc1: ADC1 控制器（正常模式）
    // - _adc_ch0: 原来的 ADC 通道 0（PA0）
    // - _dma_ch1: 原来的 DMA 通道 1
    let (_adc1, _adc_ch0, _dma_ch1) = adc_dma.split();

    // 再次插入断点，可用于调试确认拆分过程是否成功
    asm::bkpt();

    // 主循环为空，程序在此死循环
    loop {}
}