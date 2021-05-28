#![no_std]
#![no_main]

use panic_halt as _;

// use embedded_hal::digital::v2::OutputPin; //used for DEBUGGING
use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use pac::interrupt;
use stm32f1xx_hal::{
    delay::Delay,
    gpio::*,
    pac,
    pac::Interrupt::EXTI9_5,
    prelude::*,
    time::U32Ext,
    timer::{Tim2NoRemap, Timer},
};

static mut MODE: MaybeUninit<u8> = MaybeUninit::new(1);
// static mut COLOR: MaybeUninit<u8> = MaybeUninit::uninit();
static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioc::PC13<Output<PushPull>>> =
    MaybeUninit::uninit();
static mut INT_PIN: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA7<Input<Floating>>> =
    MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    //*** CHIP INIT ***//
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

    //*** PERIPHERALS ***//
    let mut delay = Delay::new(cp.SYST, clocks);

    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };
    *int_pin = gpioa.pa7.into_floating_input(&mut gpioa.crl);
    int_pin.make_interrupt_source(&mut afio);
    int_pin.trigger_on_edge(&p.EXTI, Edge::RISING);
    int_pin.enable_interrupt(&p.EXTI);

    let pa0 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let pwm = Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
        pa0,
        &mut afio.mapr,
        1.khz(),
    );
    //enable for debugging
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
    // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let led = unsafe { &mut *LED.as_mut_ptr() };
    *led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    //*** INTERRUPTS ***//

    let mut nvic = cp.NVIC;
    unsafe {
        nvic.set_priority(EXTI9_5, 1);
        cortex_m::peripheral::NVIC::unmask(EXTI9_5);
    }
    cortex_m::peripheral::NVIC::unpend(EXTI9_5);

    //*** VARS ***//
    let max = pwm.get_max_duty() / 4;
    let min = 0;

    let mut pwm_channel = pwm.split();
    pwm_channel.enable();

    let mut duty_cycle = min;
    let mut to_add = true;
    pwm_channel.set_duty(duty_cycle);

    //*** LOOP ***//
    loop {
        let mode = unsafe { &mut *MODE.as_mut_ptr() };
        if *mode == 1 {
            if to_add {
                duty_cycle += 1;
                if duty_cycle == max {
                    to_add = false;
                }
            } else {
                duty_cycle -= 1;
                if duty_cycle == min {
                    to_add = true;
                }
            }
            delay.delay_ms(1_u16);
            pwm_channel.set_duty(duty_cycle);
        } else if *mode == 0 {
            // delay.delay_ms(100_u16);
            // led.set_high().unwrap();
            // delay.delay_ms(100_u16);
            // led.set_low().unwrap();
        }
    }
}

#[interrupt]
fn EXTI9_5() {
    let mode = unsafe { &mut *MODE.as_mut_ptr() };
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };

    if int_pin.check_interrupt() {
        led.toggle().unwrap();

        if *mode == 1 {
            *mode = 0;
        } else {
            *mode = 1;
        }
        int_pin.clear_interrupt_pending_bit();
    }
}
