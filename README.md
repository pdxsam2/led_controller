To talk about:
	- Demo video
# Author and License

Samuel Parker
### [License](./LICENSE.txt)

# An LED Controller in Rust

This LED Controller is a little different compared to your standard one because my LED strip uses common power for RGB on all the LEDs on the strip and then turns on R or G or B by grounding each one respectively. This controller cycles through four available modes using Pulse-Width Modulation (PWM) to change the behavior of the strip.

## Hardware
- STM32 Blue Pill (f103C8T6 chip)
- PN2222 Transistors
- Generic LED strip
- 12V power supply
- Generic Buttons

## Setup
Make sure the following are installed...

- The most recent version of [Rust](https://rustup.rs/)

- ARM instance of GDB that supports Rust: `rustup target install thumbv7m-none-eabi`

-  [openocd](http://openocd.org/)

  

Run this project by running `openocd` in the root directory of this project and then in a separate window in the same directory enter `cargo run`. This will flash board with the program and it will start running. Verify that the hardware is setup properly by copying the demonstration video linked at the bottom of this document.

  

## Testing

Since simple unit testing is not available the functionality of the modes is verified in the demonstration video at the end of this document.

## Modes

The controller cycles through 4 modes by pressing the 'MODE' button. NOTE: pressing the 'COLOR' button at any time will change the color regardless of the mode.

  

1.  <u> Pulse Colors</u>:

Cycles through all of the colors available by using PWM on the base of the transistors.

2.  <u> Pulse Color</u>:

Pulses one color using the same method as the latter mode.

3.  <u> Constant Colors</u>:

Maintains a constant color for 10 seconds and then switches to another.

4.  <u> Constant Color</u>:

Maintains one constant color.

## Global Variables and Unsafe Access

COLOR is a static Atomic variable. Its value may be read or written at anytime so it is important to understand how it is used. Since the strip uses RGB there are 8 possible color combinations (including no color) so the value of COLOR should only range from 0 - 7. The COLOR is then translated to the strip by checking which of the first three bits are on and then setting the PWM duty cycle for the R, G, or B lines. The COLOR button will increment the color value and each mode is implement to constantly check COLOR so that it adjusts which PWM channels are on.

MODE is also a static Atomic variable which may be read or written at anytime. Each mode is implemented so that it constantly checks the MODE value and once it changes it will return to the main loop which will decide which mode to engage next. The MODE value only changes when the MODE button is pressed and increments the value.

Lastly, the handlers for the interrupts for both the MODE and COLOR buttons need to be able to reset themselves which requires access to already-owned memory. Therefore, the GPIO types have been wrapped with 'MaybeUninit' (following this example from the creators of the Hardware Abstraction Library: https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/exti.rs). This does require `unsafe` access to these variables however these pins are guaranteed to be configured/initialized BEFORE the interrupts become available to the system. Therefore, access to them is only available after being initialized. I would also like to mention that it would have made more sense to wrap the GPIO types with 'Mutex' but I didn't read enough about this until the project was finished.

# Reflection

The original goals of the project were achieved and I'm satisified with the result. I had a very clear design for how this project was going to work from the beginning and stuck to that rather closely which is both good and bad. It is good because there were very few hiccups in the process of getting things working, with the biggest issue being handling the asynchronous nature of the interrupts. It was bad, however, because I designed this in a very 'C-like' way and would have liked to dive deeper into the design patterns that suit Rust best. 
	I would like to add more implementation in the future. First off, I want to add a potentiometer to the system which the board can read from. This can then be used in software to adjust various parameters within the modes. For instance, for the 'pulse colors' mode I would like to adjust the speed at which it pulses. Or, for the 'constant colors' mode I would like to adjust either the brightness or the duration between color changes. 
	On a more Rust-relevant front, however, I also want to abstract the functionality of the modes into a larger object (similar to this: https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/timer-interrupt-rtic.rs). I think this would get me a bit deeper into how Rust objects interact with eachother and overall be a more developer-friendly design. 
  
### [Demo Video](youtube.com)
###### Bug: As I was recording the demo the strip started changing modes after all of the colors were on... I spent a couple hours trying to solve this but couldn't figure it out so if you have any suggestions please let me know. 
