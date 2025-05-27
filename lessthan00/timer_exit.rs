//! blinky timer using interrupts on TIM2
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Please note according to RM0008:
//! "Due to the fact that the switch only sinks a limited amount of current (3 mA), the use of
//! GPIOs PC13 to PC15 in output mode is restricted: the speed has to be limited to 2MHz with
//! a maximum load of 30pF and these IOs must not be used as a current source (e.g. to drive a LED)"

#![no_main]
#![no_std]

use panic_halt as _;

use stm32f1xx_hal as hal;

use crate::hal::{
    gpio::{gpioc, Output, PinState, PushPull},
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::{CounterMs, Event},
};

use core::cell::RefCell;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;

// NOTE You can uncomment 'hprintln' here and in the code below for a bit more
// verbosity at runtime, at the cost of throwing off the timing of the blink
// (using 'semihosting' for printing debug info anywhere slows program
// execution down)
//use cortex_m_semihosting::hprintln;

// Mutex: 在Rust中，Mutex通常用于多线程环境下的数据保护，但在单线程的嵌入式环境中，它同样可以用来防止中断服务例程（ISR）与主程序之间发生竞争条件。这里的Mutex是由cortex-m库提供的，专门用于嵌入式系统。
// RefCell: RefCell提供了一种在运行时检查借用规则的方法，允许你在可变性和借用上拥有更大的灵活性。这对于需要在编译时无法确定借用关系的情况下非常有用，比如在这个场景中ISR需要动态地获取对某些资源的可变引用。
// A type definition for the GPIO pin to be used for our LED
type LedPin = gpioc::PC13<Output<PushPull>>;

// Make LED pin globally available
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<CounterMs<TIM2>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer TIM2 times out
#[interrupt]
fn TIM2() {
    static mut LED: Option<LedPin> = None;
    static mut TIM: Option<CounterMs<TIM2>> = None;
// get_or_insert_with: 这个方法用于尝试获取一个值，如果该值不存在，则通过给定的闭包生成一个新的值并插入。
// 这保证了即使多个线程（在这里主要是中断和主线程）同时尝试获取这个值，也只会有一个实际创建新值的过程。
// cortex_m::interrupt::free: 这个函数的作用是在其作用域内禁止所有中断，从而确保在执行临界区代码时不会被中断打断。
// 这样，在获取或者设置全局变量时，可以避免竞态条件的发生。
// G_LED.borrow(cs): 借助于Mutex提供的上下文(cs)，我们可以安全地从全局变量中借出引用。
// 这里borrow(cs)会返回一个RefMut类型的引用，允许我们对内部的数据进行修改。
    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_LED.borrow(cs).replace(None).unwrap()
        })
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    let _ = led.toggle();
    let _ = tim.wait();
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.MHz())
        .pclk1(8.MHz())
        .freeze(&mut flash.acr);

    // Configure PC13 pin to blink LED
    let mut gpioc = dp.GPIOC.split();
    let led = Output::new(gpioc.pc13, &mut gpioc.crh, PinState::High);
    //or
    // let led = gpioc.pc13.into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);
    // 同样是利用了cortex_m::interrupt::free来确保在没有中断干扰的情况下，安全地将新的led或timer实例赋值给全局变量。
    // borrow_mut(): 获取一个可变引用，允许我们改变全局变量的内容。
    // Some(led) 或 Some(timer): 将具体的硬件资源封装进Option类型，并存储到全局变量中，以便后续在ISR中能够访问。
    // Move the pin into our global storage
    cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

    // Set up a timer expiring after 1s
    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer.start(1.secs()).unwrap();

    // Generate an interrupt when the timer expires
    timer.listen(Event::Update);

    // Move the timer into our global storage
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    // 最后进入无限循环，使用wfi();指令等待中断的发生。这种方法可以节省电力，因为在等待期间CPU处于低功耗状态直到下一个中断到来。
    loop {
        wfi();
    }
}
