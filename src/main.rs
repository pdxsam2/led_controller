#![no_std]
#![no_main]

use panic_halt as _;
//TODO:
//  demo video
//  change 'mode' to 'current_mode'

use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};
use cortex_m_rt::entry;
use stm32f1xx_hal::{
    delay::Delay,
    gpio::*,
    pac,
    pac::{
        interrupt,
        Interrupt::{EXTI4, EXTI9_5},
        TIM2,
    },
    prelude::*,
    pwm::{PwmChannel, C1, C2, C3},
    time::U32Ext,
    timer::{Tim2NoRemap, Timer},
};

///Determines which mode the LED strip is in
static MODE: AtomicU8 = AtomicU8::new(0);
///Determines the color of the LED strip
static COLOR: AtomicU8 = AtomicU8::new(1);

///Interrupt pin used for changing the mode
static mut MODE_BUTTON: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA5<Input<Floating>>> =
    MaybeUninit::uninit();
///Interrupt pin used for changing the color
static mut COLOR_BUTTON: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA4<Input<Floating>>> =
    MaybeUninit::uninit();

///Object containing all three
type PwmChannels = (
    PwmChannel<TIM2, C1>,
    PwmChannel<TIM2, C2>,
    PwmChannel<TIM2, C3>,
);

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

    //*** PERIPHERAL INIT ***//
    let mut delay = Delay::new(cp.SYST, clocks);

    let mode_button = unsafe { &mut *MODE_BUTTON.as_mut_ptr() };
    *mode_button = gpioa.pa5.into_floating_input(&mut gpioa.crl);
    mode_button.make_interrupt_source(&mut afio);
    mode_button.trigger_on_edge(&p.EXTI, Edge::RISING);
    mode_button.enable_interrupt(&p.EXTI);

    let color_button = unsafe { &mut *COLOR_BUTTON.as_mut_ptr() };
    *color_button = gpioa.pa4.into_floating_input(&mut gpioa.crl);
    color_button.make_interrupt_source(&mut afio);
    color_button.trigger_on_edge(&p.EXTI, Edge::RISING);
    color_button.enable_interrupt(&p.EXTI);

    let pa0 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let pa1 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
    let pa2 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let pwm = Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
        (pa0, pa1, pa2),
        &mut afio.mapr,
        1.khz(),
    );

    //*** INTERRUPTS ***//
    let mut nvic = cp.NVIC;
    unsafe {
        nvic.set_priority(EXTI9_5, 1);
        nvic.set_priority(EXTI4, 1);
        cortex_m::peripheral::NVIC::unmask(EXTI9_5);
        cortex_m::peripheral::NVIC::unmask(EXTI4);
    }
    cortex_m::peripheral::NVIC::unpend(EXTI9_5);
    cortex_m::peripheral::NVIC::unpend(EXTI4);

    //*** PRIVATE VARS ***//
    let mut pwm_channels: PwmChannels = pwm.split();
    pwm_channels.0.enable();
    pwm_channels.1.enable();
    pwm_channels.2.enable();

    //*** LOOP ***//
    loop {
        let current_mode = MODE.load(Ordering::Relaxed);
        if current_mode == 3 {
            pulse_color(current_mode, &mut pwm_channels, &mut delay);
        } else if current_mode == 2 {
            pulse_colors(current_mode, &mut pwm_channels, &mut delay);
        } else if current_mode == 1 {
            const_color(current_mode, &mut pwm_channels);
        } else {
            const_colors(current_mode, &mut pwm_channels);
        }
    }
}

#[interrupt]
fn EXTI9_5() {
    let mut mode = MODE.load(Ordering::Relaxed);
    let mode_button = unsafe { &mut *MODE_BUTTON.as_mut_ptr() };

    if mode_button.check_interrupt() {
        mode = (mode + 1) % 4;
        MODE.store(mode, Ordering::Relaxed);
        mode_button.clear_interrupt_pending_bit();
    }
}

#[interrupt]
fn EXTI4() {
    let color_button = unsafe { &mut *COLOR_BUTTON.as_mut_ptr() };
    let mut color = COLOR.load(Ordering::Relaxed);

    if color_button.check_interrupt() {
        color = (color + 1) % 8;
        COLOR.store(color, Ordering::Relaxed);
        color_button.clear_interrupt_pending_bit();
    }
}
/// Pulses a single color, if the value of COLOR changes this will change
fn pulse_color(current_mode: u8, channels: &mut PwmChannels, delay: &mut Delay) {
    let max = channels.0.get_max_duty();
    let min = 0;
    let mut duty_cycle = min;
    let mut to_add = true;

    channels.0.set_duty(duty_cycle);
    channels.1.set_duty(duty_cycle);
    channels.2.set_duty(duty_cycle);

    while MODE.load(Ordering::Relaxed) == current_mode {
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
        } else {
            channels.0.set_duty(min);
        }
        if color & 2 == 2 {
            channels.1.set_duty(duty_cycle);
        } else {
            channels.1.set_duty(min);
        }
        if color & 4 == 4 {
            channels.2.set_duty(duty_cycle);
        } else {
            channels.2.set_duty(min);
        }
        delay.delay_ms(1_u16);
    }
    channels.0.set_duty(min);
    channels.1.set_duty(min);
    channels.2.set_duty(min);
}
/// Changes color after every pulse, changing the value of COLOR will change immediately
fn pulse_colors(current_mode: u8, channels: &mut PwmChannels, delay: &mut Delay) {
    let max = channels.0.get_max_duty();
    let min = 0;
    let mut duty_cycle = min;
    let mut to_add = true;

    channels.0.set_duty(duty_cycle);
    channels.1.set_duty(duty_cycle);
    channels.2.set_duty(duty_cycle);

    while MODE.load(Ordering::Relaxed) == current_mode {
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
        } else {
            channels.0.set_duty(min);
        }
        if color & 2 == 2 {
            channels.1.set_duty(duty_cycle);
        } else {
            channels.1.set_duty(min);
        }
        if color & 4 == 4 {
            channels.2.set_duty(duty_cycle);
        } else {
            channels.2.set_duty(min);
        }
        delay.delay_ms(1_u16);
    }
    channels.0.set_duty(min);
    channels.1.set_duty(min);
    channels.2.set_duty(min);
}

///Display a constant color which can be adjust by the color button
fn const_color(current_mode: u8, channels: &mut PwmChannels) {
    let max = channels.0.get_max_duty();
    let min = 0;
    while MODE.load(Ordering::Relaxed) == current_mode {
        let color = COLOR.load(Ordering::Relaxed);
        if color & 1 == 1 {
            channels.0.set_duty(max);
        } else {
            channels.0.set_duty(min);
        }
        if color & 2 == 2 {
            channels.1.set_duty(max);
        } else {
            channels.1.set_duty(min);
        }
        if color & 4 == 4 {
            channels.2.set_duty(max);
        } else {
            channels.2.set_duty(min);
        }
    }
}

///Display a color for approximately 10 seconds and then switch to another
fn const_colors(current_mode: u8, channels: &mut PwmChannels) {
    let max = channels.0.get_max_duty();
    let min = 0;
    let mut ticks: u32 = 0;
    let approx_second = 15000; //I timed this by hand, and I agree that it's a bad way of doing it

    while MODE.load(Ordering::Relaxed) == current_mode {
        while ticks < (10 * approx_second) && MODE.load(Ordering::Relaxed) == current_mode {
            let color = COLOR.load(Ordering::Relaxed);
            if color & 1 == 1 {
                channels.0.set_duty(max);
            } else {
                channels.0.set_duty(min);
            }
            if color & 2 == 2 {
                channels.1.set_duty(max);
            } else {
                channels.1.set_duty(min);
            }
            if color & 4 == 4 {
                channels.2.set_duty(max);
            } else {
                channels.2.set_duty(min);
            }
            ticks += 1;
        }
        ticks = 0;
        let mut color = COLOR.load(Ordering::Relaxed);
        color = (color + 1) % 8;
        if color == 0 {
            color += 1;
        }
        COLOR.store(color, Ordering::Relaxed);
    }
}
