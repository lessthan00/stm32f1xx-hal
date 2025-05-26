#![no_std]
#![no_main]

// 使用 panic_halt 作为 panic 处理程序，当发生 panic 时直接停止程序
use panic_halt as _;

// 导入 cortex_m_semihosting 的 hprintln 宏，用于通过调试器输出信息
use cortex_m_semihosting::hprintln;

// 从 core 库导入 MaybeUninit，用于安全地初始化静态变量
use core::mem::MaybeUninit;
// 从 core 库导入原子操作相关的类型和方法
use core::sync::atomic::{AtomicI16, Ordering};

// 导入 cortex_m_rt 的 entry 宏，定义程序入口点
use cortex_m_rt::entry;
// 导入 pac 模块的 interrupt 属性宏，用于定义中断处理函数
use pac::interrupt;

// 导入 stm32f1xx_hal 的 GPIO 相关类型和特性
use stm32f1xx_hal::gpio::*;
// 导入 stm32f1xx_hal 的 PAC (Peripheral Access Crate) 和预导入模块
use stm32f1xx_hal::{pac, prelude::*};
// 导入 stm32f1xx_hal 的定时器模块
use stm32f1xx_hal::timer::Timer;
// 导入 nb 库的 block 宏，用于阻塞等待异步操作完成
use nb::block;

// LED 引脚 PA0（推挽输出模式）
// 使用 MaybeUninit 是因为 GPIO 引脚必须运行时初始化，而 Rust 静态变量需编译期初始化
// 必须用 unsafe 来访问和修改
static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA0<Output<PushPull>>> =
    MaybeUninit::uninit();
static mut INT_PIN1: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA2<Input<Floating>>> =
    MaybeUninit::uninit();
static mut INT_PIN2: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA3<Input<Floating>>> =
    MaybeUninit::uninit();
// 使用 AtomicI16 来保证跨上下文访问的安全
static ENCODER_COUNT: AtomicI16 = AtomicI16::new(0);

#[interrupt]
fn EXTI2() {
    // 在中断中安全地访问LED和INT_PIN
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin1 = unsafe { &mut *INT_PIN1.as_mut_ptr() };
    let int_pin2 = unsafe { &mut *INT_PIN2.as_mut_ptr() };
    if int_pin1.check_interrupt() {
        if int_pin1.is_low(){
            if int_pin2.is_low() { //A 下降沿 + B 低电平 → 正转
                led.toggle();  // 切换LED状态
                ENCODER_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        // 如果不清除这个标志位，中断会无限触发
        int_pin1.clear_interrupt_pending_bit();
    }
}

#[interrupt]
fn EXTI3() {
    // 在中断中安全地访问LED和INT_PIN
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin1 = unsafe { &mut *INT_PIN1.as_mut_ptr() };
    let int_pin2 = unsafe { &mut *INT_PIN2.as_mut_ptr() };
    if int_pin2.check_interrupt() {
        if int_pin2.is_low(){
            if int_pin1.is_low() { //A 低沿 + B 下降沿 → 反转
                led.toggle();  // 切换LED状态
                ENCODER_COUNT.fetch_sub(1, Ordering::Relaxed);
            }
        }
        // 如果不清除这个标志位，中断会无限触发
        int_pin2.clear_interrupt_pending_bit();
    }
}

#[entry]
fn main() -> ! {
    // 初始化阶段
    let mut dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut nvic = cp.NVIC;

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioc = dp.GPIOC.split();
    let mut led_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();
    {
        // 这个作用域确保在第一个ISR执行前，int_pin的引用会被释放
        
        let mut gpioa = dp.GPIOA.split();
        let mut afio = dp.AFIO.constrain();

        // 初始化LED引脚(PC13)为推挽输出
        
        let led = unsafe { &mut *LED.as_mut_ptr() };
        *led = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);

        // 初始化中断引脚(PA7)为浮空输入
        let int_pin1 = unsafe { &mut *INT_PIN1.as_mut_ptr() };
        *int_pin1 = gpioa.pa2.into_floating_input(&mut gpioa.crl);
        // 配置PA7为中断源
        int_pin1.make_interrupt_source(&mut afio);
        // 设置中断触发方式为上升沿和下降沿
        int_pin1.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);
        // 使能中断
        int_pin1.enable_interrupt(&mut dp.EXTI);

        let int_pin2 = unsafe { &mut *INT_PIN2.as_mut_ptr() };
        *int_pin2 = gpioa.pa3.into_floating_input(&mut gpioa.crl);
        // 配置PA7为中断源
        int_pin2.make_interrupt_source(&mut afio);
        // 设置中断触发方式为上升沿和下降沿
        int_pin2.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);
        // 使能中断
        int_pin2.enable_interrupt(&mut dp.EXTI);
    } // 初始化阶段结束

    // 启用 NVIC 中断
    // 在启用中断前设置它们的优先级
    unsafe {
        // 设置EXTI2的优先级为1（数值越低，优先级越高）
        nvic.set_priority(pac::Interrupt::EXTI2, 1 << 4); 
        // 设置EXTI3的优先级为2
        nvic.set_priority(pac::Interrupt::EXTI3, 2 << 4);
        
        // 启用中断
        pac::NVIC::unmask(pac::Interrupt::EXTI2);
        pac::NVIC::unmask(pac::Interrupt::EXTI3);
    }
    // 创建一个1Hz的系统定时器
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();
    loop {
        block!(timer.wait()).unwrap();  // 等待定时器触发
        led_pc13.toggle();
        // 获取自上次调用以来的增量值，并重置计数器
        let count = ENCODER_COUNT.swap(0, Ordering::Relaxed);
        hprintln!("coder count: {}", count);  // 通过半主机输出中断计数
        
    }
}
// cargo check --features "stm32f103" --example my_gpio_exit_encoder
// cargo run --features "stm32f103" --example my_gpio_exit_encoder