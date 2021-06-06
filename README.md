To talk about:
    - How does color work?
    - How does the mode work?

# An LED Controller in Rust

This LED Controller is a little different compared to your standard one because the LED strip I'm using uses common power for RGB on all the LEDs on the strip and then turns on R or G or B by grounding each one respectively.

## Hardware

- STM32 Blue Pill (f103C8T6 chip)

- PN2222 Transistors

- Generic LED strip

- 12V power supply

- Generic Buttons

## Modes

The controller cycles through 4 modes by pressing one of the buttons. NOTE: pressing the 'COLOR' button at any time will change the color regardless of the mode. 

1. <u> Pulse Colors</u>: 
	Cycles through all of the colors available by using PWM on the base of the transistors. 
	
2. <u> Pulse Color</u>: 
	Pulses one color using the same method as the latter mode. 
	
3. <u> Constant Colors</u>:
	Maintains a constant color for 10 seconds and then switches to another. 
	
4. <u> Constant Color</u>:
	Maintains one constant color. 
	
## Color Explanation
COLOR is a static Atomic variable. Its value may be read or written at anytime so it is important to understand how it is used. Since the strip uses RGB there are 8 possible color combinations (including no color) so the value of COLOR should only range from 0 - 7. The COLOR is then translated to the strip by checking which of the first three bits are on and then setting the PWM duty cycle for the R, G, or B lines. The COLOR button will increment the color value and each mode is implement to constantly check COLOR so that it adjusts which PWM channels are on. 

## Mode Explanation
MODE is also a static Atomic variable which may be read or written at anytime. Each mode is implemented so that it constantly checks the MODE value and once it changes it will return to the main loop which will decide which mode to engage next. The MODE value only changes when the MODE button is pressed and increments the value. 

## Next Steps...
Potentiometer!

## Docs
