#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

// use embedded_hal::digital::v2::OutputPin; //used for LEDS
use cortex_m_rt::entry;
use stm32f1xx_hal::{
	pac, 
	prelude::*, 
	time::U32Ext,
	timer::{Timer, Tim2NoRemap},
	delay::Delay
};

#[entry]
fn main() -> ! {
	let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

	let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    

    
	let c = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
	let pwm = Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
		c,
		&mut afio.mapr,
		1.khz(),
	);
	//	ENABLE THESE FOR DEBUGGING:
	// let mut gpioc = p.GPIOC.split(&mut rcc.apb2);
	//	let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

	let mut delay = Delay::new(cp.SYST, clocks);

	let max = pwm.get_max_duty() / 4;
	let min = 0;

	let mut pwm_channel = pwm.split();
	pwm_channel.enable();

	let mut duty_cycle= min;
	let mut to_add= true;
	pwm_channel.set_duty(duty_cycle);

    loop {
		if to_add == true {
			duty_cycle += 1;
			if duty_cycle == max {
				to_add = false;
			}
		}
		else {
			duty_cycle -= 1;
			if duty_cycle == min {
				to_add = true;
			}
		}
		delay.delay_ms(1_u16);
		pwm_channel.set_duty(duty_cycle);
		// delay.delay_ms(1000_u16);
		// pwm_channel.set_duty(max);
		// delay.delay_ms(1000_u16);
		// pwm_channel.set_duty(max/4);

		

        // block!(timer.wait()).unwrap();
        // led.set_high().unwrap();
		// block!(timer.wait()).unwrap();
        // led.set_low().unwrap();
    }
}
