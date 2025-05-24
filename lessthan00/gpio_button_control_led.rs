//! Blinks several LEDs stored in an array
//! PC13->LED
//! PB11->button
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap();

    let mut led_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase(); // 持续闪烁的LED
    let mut led_pa0 = gpioa.pa0.into_push_pull_output(&mut gpioa.crl).erase();   // 受按钮控制的LED
    let mut button = gpiob.pb11.into_pull_up_input(&mut gpiob.crh).erase();      // 按钮（按下=低电平）

    // 消抖相关变量
    let mut button_state = false;      // 当前稳定状态
    let mut last_button_state = false; // 上次物理状态
    let mut stable_count: u8 = 0;      // 稳定周期计数
    const DEBOUNCE_CYCLES: u8 = 1;     // 需要稳定的周期数（200ms × 1 = 200ms）

    loop {
        block!(timer.wait()).unwrap(); // 每200ms触发一次
        led_pc13.toggle();             // PC13 LED闪烁

        //--- 按钮消抖逻辑 ---//
        let current_state = button.is_low();

        // 状态变化时重置计数器
        if current_state != last_button_state {
            stable_count = 0;
        } else {
            stable_count += 1; // 状态稳定时累加
        }

        // 检测稳定状态（持续200ms×N）
        if stable_count >= DEBOUNCE_CYCLES {
            if current_state != button_state {
                button_state = current_state;
                if button_state { // 按钮稳定按下时
                    led_pa0.toggle();
                }
            }
        }

        last_button_state = current_state;
    }
}

// cargo check --features stm32f103 --example my_gpio_button_control_led
// cargo run --features stm32f103 --example my_gpio_button_control_led
