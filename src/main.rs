#![no_std]
#![no_main]

use panic_halt as _;
//TODO:
//  clean up imports
//  README and docs
//  Organize things into functions?
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};
use cortex_m_rt::entry;
use pac::interrupt;
use stm32f1xx_hal::{
    delay::Delay,
    gpio::*,
    pac,
    pac::Interrupt::EXTI9_5,
    pac::TIM2,
    prelude::*,
    pwm::{PwmChannel, C1, C2, C3},
    time::U32Ext,
    timer::{Tim2NoRemap, Timer},
};

static MODE: AtomicU8 = AtomicU8::new(2);
static COLOR: AtomicU8 = AtomicU8::new(7);
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
    let pa1 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
    let pa2 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let pwm = Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
        (pa0, pa1, pa2),
        &mut afio.mapr,
        1.khz(),
    );
    //enable for debugging
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
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
    let mut pwm_channels = pwm.split();
    pwm_channels.0.enable();
    pwm_channels.1.enable();
    pwm_channels.2.enable();

    //*** LOOP ***//
    loop {
        let mode = MODE.load(Ordering::Relaxed);
        if mode == 3 {
            pulse_color(mode, &mut pwm_channels, &mut delay);
        } else if mode == 2 {
            pulse_colors(mode, &mut pwm_channels, &mut delay);
        } else if mode == 1 {
        } else {
        }
    }
}

#[interrupt]
fn EXTI9_5() {
    let mut mode = MODE.load(Ordering::Relaxed);
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };

    if int_pin.check_interrupt() {
        led.toggle().unwrap();
        mode = (mode + 1) % 4;
        MODE.store(mode, Ordering::Relaxed);
        int_pin.clear_interrupt_pending_bit();
    }
}
/// Pulses a single color, if the value of COLOR changes this will change
fn pulse_color(
    mode: u8,
    channels: &mut (
        PwmChannel<TIM2, C1>,
        PwmChannel<TIM2, C2>,
        PwmChannel<TIM2, C3>,
    ),
    delay: &mut Delay,
) {
    let max = channels.0.get_max_duty() / 4;
    let min = 0;
    let mut duty_cycle = min;
    let mut to_add = true;

    channels.0.set_duty(duty_cycle);
    channels.1.set_duty(duty_cycle);
    channels.2.set_duty(duty_cycle);

    while MODE.load(Ordering::Relaxed) == mode {
        let color = COLOR.load(Ordering::Relaxed);
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
        
        if color & 1 == 1 {
            channels.0.set_duty(duty_cycle);
        }
        else {
            channels.0.set_duty(min);
        }
        if color & 2 == 2{
            channels.1.set_duty(duty_cycle);
        }
        else {
            channels.1.set_duty(min);
        }
        if color & 4 == 4 {
            channels.2.set_duty(duty_cycle);
        }
        else {
            channels.2.set_duty(min);
        }
        delay.delay_ms(1_u16);
    }
}
/// Changes color after every pulse, changing the value of COLOR will change immediately
fn pulse_colors(
    mode: u8,
    channels: &mut (
        PwmChannel<TIM2, C1>,
        PwmChannel<TIM2, C2>,
        PwmChannel<TIM2, C3>,
    ),
    delay: &mut Delay,
) {
    let max = channels.0.get_max_duty() / 4;
    let min = 0;
    let mut duty_cycle = min;
    let mut to_add = true;

    channels.0.set_duty(duty_cycle);
    channels.1.set_duty(duty_cycle);
    channels.2.set_duty(duty_cycle);

    while MODE.load(Ordering::Relaxed) == mode {
        let mut color = COLOR.load(Ordering::Relaxed);
        if to_add {
            duty_cycle += 1;
            if duty_cycle == max {
                to_add = false;
            }
        } else {
            duty_cycle -= 1;
            if duty_cycle == min {
                color = (color + 1) % 8;
                if color == 0 {
                    color += 1;
                }
                COLOR.store(color, Ordering::Relaxed);
                to_add = true;
            }
        }
        
        if color & 1 == 1 {
            channels.0.set_duty(duty_cycle);
        }
        else {
            channels.0.set_duty(min);
        }
        if color & 2 == 2{
            channels.1.set_duty(duty_cycle);
        }
        else {
            channels.1.set_duty(min);
        }
        if color & 4 == 4 {
            channels.2.set_duty(duty_cycle);
        }
        else {
            channels.2.set_duty(min);
        }
        // delay.delay_ms(1_u16);
    }
}
