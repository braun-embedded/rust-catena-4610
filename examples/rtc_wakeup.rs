#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;
use stm32l0xx_hal::{
    exti::{
        self,
        Exti,
    },
    pac,
    prelude::*,
    pwr::PWR,
    rcc,
    rtc::{self, Instant, RTC},
};

#[entry]
fn main() -> ! {
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(rcc::Config::hsi16());

    // Initialize all the GPIO we need
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = gpiob.pb2.into_push_pull_output();
    let button = gpiob.pb5.into_pull_down_input(); // external button

    // Enable LED to signal that MCU is running
    led.set_high().unwrap();

    let mut scb    = cp.SCB;
    let mut exti   = Exti::new(dp.EXTI);
    let mut pwr    = PWR::new(dp.PWR, &mut rcc);

    let instant = Instant::new()
        .set_year(19)
        .set_month(8)
        .set_day(12)
        .set_hour(12)
        .set_minute(55)
        .set_second(0);

    let mut rtc = RTC::new(dp.RTC, &mut rcc, &mut pwr, instant);

    rtc.enable_interrupts(rtc::Interrupts {
        wakeup_timer: true,
        ..rtc::Interrupts::default()
    });
    exti.listen_configurable(
        exti::ConfigurableLine::RtcWakeup,
        exti::TriggerEdge::Rising,
    );

    while button.is_low().unwrap() {}

    rtc.wakeup_timer().start(1u32);

    exti.wait_for_irq(
        exti::ConfigurableLine::RtcWakeup,
        pwr.standby_mode(&mut scb),
    );

    // Waking up from Standby mode resets the microcontroller, so we should
    // never reach this point.
    loop {
        led.set_high().unwrap();
    }
}
