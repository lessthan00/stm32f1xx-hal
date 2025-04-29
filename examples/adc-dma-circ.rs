//! ADC接口循环DMA接收传输测试

#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

// 当程序出现panic时，使用halt作为处理方式。
use panic_halt as _;

// 引入Cortex-M核心相关的功能，如汇编指令和单例创建
use cortex_m::{asm, singleton};

// 使用cortex-m-rt提供的entry宏定义入口函数
use cortex_m_rt::entry;
// 引入STM32F1xx硬件抽象库的相关模块，包括ADC、DMA、外设访问通道(PAC)和预导入模块
use stm32f1xx_hal::{adc, dma::Half, pac, prelude::*};

#[entry]
fn main() -> ! {
    // 获取设备的外围设备实例
    let p = pac::Peripherals::take().unwrap();
    // 获取FLASH控制寄存器并进行配置
    let mut flash = p.FLASH.constrain();
    // 获取RCC(复位与时钟控制)并进行配置
    let rcc = p.RCC.constrain();

    // 配置ADC使用的时钟频率。默认值是PCLK2/8。这里设置为2MHz，
    // 实际频率会根据支持的预分频值2/4/6/8自动调整以接近指定值。
    let clocks = rcc.cfgr.adcclk(2.MHz()).freeze(&mut flash.acr);

    // 分割DMA1通道，并获取第1个通道用于ADC数据传输
    let dma_ch1 = p.DMA1.split().1;

    // 创建一个新的ADC1实例
    let adc1 = adc::Adc::new(p.ADC1, &clocks);

    // 分割GPIOA，以便配置各个引脚的功能
    let mut gpioa = p.GPIOA.split();

    // 将PA0配置为模拟输入模式，用于ADC采样
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    // 将ADC通道与DMA通道绑定，启用DMA传输
    let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);
    // 创建一个双缓冲区（每个大小为8个u16元素），用于存储ADC采样结果
    let buf = singleton!(: [[u16; 8]; 2] = [[0; 8]; 2]).unwrap();

    // 启动循环读取模式，DMA会在两个缓冲区间交替填充数据
    let mut circ_buffer = adc_dma.circ_read(buf);

    // 等待第一个缓冲区被填满
    while circ_buffer.readable_half().unwrap() != Half::First {}

    // 读取第一个缓冲区的数据
    let _first_half = circ_buffer.peek(|half, _| *half).unwrap();

    // 等待第二个缓冲区被填满
    while circ_buffer.readable_half().unwrap() != Half::Second {}

    // 读取第二个缓冲区的数据
    let _second_half = circ_buffer.peek(|half, _| *half).unwrap();

    // 停止循环读取，并回收资源
    let (_buf, adc_dma) = circ_buffer.stop();
    let (_adc1, _adc_ch0, _dma_ch1) = adc_dma.split();
    // 插入断点，可用于调试
    asm::bkpt();

    // 进入无限循环
    loop {}
}