//! Turns the user LED on
//!
//! Listens for interrupts on the pa7 pin. On any rising or falling edge, toggles
//! the pc13 pin (which is connected to the LED on the blue pill board, hence the `led` name).

#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

// 使用 panic_halt 作为 panic 处理程序，当发生 panic 时直接停止程序
use panic_halt as _;

// 导入 cortex_m_semihosting 的 hprintln 宏，用于通过调试器输出信息
use cortex_m_semihosting::hprintln;

// 从 core 库导入 MaybeUninit，用于安全地初始化静态变量
use core::mem::MaybeUninit;
// 从 core 库导入原子操作相关的类型和方法
use core::sync::atomic::{AtomicU32, Ordering};

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

// These two are owned by the ISR. main() may only access them during the initialization phase,
// where the interrupt is not yet enabled (i.e. no concurrent accesses can occur).
// After enabling the interrupt, main() may not have any references to these objects any more.
// For the sake of minimalism, we do not use RTIC here, which would be the better way.
// 这些变量由中断服务程序(ISR)拥有。main()函数只能在初始化阶段访问它们，
// 此时中断尚未启用(即不会发生并发访问)。
// 启用中断后，main()不能再持有这些对象的任何引用。
// 为了简单起见，我们没有使用RTIC框架(这会是更好的方式)。

// LED 引脚 PA0（推挽输出模式）
// 使用 MaybeUninit 是因为 GPIO 引脚必须运行时初始化，而 Rust 静态变量需编译期初始化
// 必须用 unsafe 来访问和修改
static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA0<Output<PushPull>>> =
    MaybeUninit::uninit();

// 外部中断引脚 PA7（浮空输入模式）
// 同样需要延迟初始化，且电平由外部电路决定
static mut INT_PIN: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA7<Input<Floating>>> =
    MaybeUninit::uninit();

// 中断计数器，主循环与 ISR 共享
// 使用 AtomicU32 是为了保证跨上下文访问的原子性，防止数据竞争
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

#[interrupt]
fn EXTI9_5() {
    // 在中断中安全地访问LED和INT_PIN
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };

    if int_pin.check_interrupt() {
        led.toggle();  // 切换LED状态
        INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);  // 原子地增加中断计数器
        // 如果不清除这个标志位，中断会无限触发
        int_pin.clear_interrupt_pending_bit();
    }
}

#[entry]
fn main() -> ! {
    // 初始化阶段
    let mut dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

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
        let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };
        *int_pin = gpioa.pa7.into_floating_input(&mut gpioa.crl);
        // 配置PA7为中断源
        int_pin.make_interrupt_source(&mut afio);
        // 设置中断触发方式为上升沿和下降沿
        int_pin.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);
        // 使能中断
        int_pin.enable_interrupt(&mut dp.EXTI);
    } // 初始化阶段结束

    // 启用EXTI9_5中断
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    // 创建一个5Hz的系统定时器
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap();
    
    loop {
        block!(timer.wait()).unwrap();  // 等待定时器触发
        led_pc13.toggle();
        // 原子地读取当前中断计数
        let count = INTERRUPT_COUNT.load(Ordering::Relaxed);
        hprintln!("Interrupt count: {}", count);  // 通过半主机输出中断计数
    }
}
// cargo check --features "stm32f103" --example my_gpio_exit_counter
// cargo run --features "stm32f103" --example my_gpio_exit_counter