use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use rppal::{
    gpio::{Gpio, InputPin},
    hal::Delay,
    spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use st7735_lcd::{Orientation, ST7735};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{thread, time::Duration};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

// Waveshare 1.44inch LCD HAT pin numbers use BCM numbering.
const DC_PIN: u8 = 25;
const RESET_PIN: u8 = 27;
const BACKLIGHT_PIN: u8 = 24;

const BACKGROUND: Rgb565 = Rgb565::new(0, 4, 10);

type Error = Box<dyn std::error::Error>;

// Joystick and button pins on the Waveshare 1.44inch LCD HAT (BCM numbering).
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, EnumIter)]
#[repr(u8)]
enum GpioInput {
    JoystickUp = 6,
    JoystickDown = 19,
    JoystickLeft = 5,
    JoystickRight = 26,
    JoystickPress = 13,
    Button1 = 21,
    Button2 = 20,
    Button3 = 16,
}

impl GpioInput {
    fn init(&self, gpio: Gpio) -> InputPin {
        gpio.get(*self as u8).unwrap().into_input_pullup()
    }

    fn to_str(&self) -> &'static str {
        match self {
            GpioInput::JoystickUp => "Joystick Up",
            GpioInput::JoystickDown => "Joystick Down",
            GpioInput::JoystickLeft => "Joystick Left",
            GpioInput::JoystickRight => "Joystick Right",
            GpioInput::JoystickPress => "Joystick Press",
            GpioInput::Button1 => "Button 1",
            GpioInput::Button2 => "Button 2",
            GpioInput::Button3 => "Button 3",
        }
    }
}

impl Display for GpioInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

fn main() -> Result<(), Error> {
    // SPI0 CE0 maps to the HAT's SCLK=BCM11, MOSI=BCM10, CS=BCM8.
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
    let spi = SimpleHalSpiDevice::new(spi);

    let gpio = Gpio::new()?;
    let mut dc = gpio.get(DC_PIN)?.into_output_low();
    let mut reset = gpio.get(RESET_PIN)?.into_output_high();
    let mut backlight = gpio.get(BACKLIGHT_PIN)?.into_output_high();

    let inputs: HashMap<GpioInput, InputPin> = GpioInput::iter()
        .map(|input| (input.clone(), input.init(gpio)))
        .collect();

    // Keep the panel enabled after this short-lived program exits. By default,
    // rppal resets GPIO pins to inputs when their handles are dropped, which
    // releases RESET and BACKLIGHT and makes the freshly drawn image disappear.
    dc.set_reset_on_drop(false);
    reset.set_reset_on_drop(false);
    backlight.set_reset_on_drop(false);
    backlight.set_high();

    let mut delay = Delay::new();
    let mut display = ST7735::new(spi, dc, reset, true, false, WIDTH, HEIGHT);

    display
        .init(&mut delay)
        .map_err(|_| "ST7735 initialization failed")?;
    display
        .set_orientation(&Orientation::Landscape)
        .map_err(|_| "could not set display orientation")?;

    // The 1.44-inch panel starts one controller pixel to the right.
    display.set_offset(1, 0);
    display
        .clear(BACKGROUND)
        .map_err(|_| "could not clear display")?;

    let heading = MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN);
    let body = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::with_baseline("INPUT TEST", Point::new(34, 14), heading, Baseline::Top)
        .draw(&mut display)
        .map_err(|_| "could not draw heading")?;
    Text::with_baseline("Press a control", Point::new(19, 42), body, Baseline::Top)
        .draw(&mut display)
        .map_err(|_| "could not draw text")?;

    println!("Watching joystick and buttons. Press Ctrl-C to stop.");

    let mut previous_state = vec![];
    loop {
        let state: Vec<GpioInput> = inputs
            .iter()
            .filter(|(_, pin)| pin.is_low())
            .map(|(input, _)| input)
            .collect();

        if state != previous_state {
            draw_input_state(&mut display, &inputs, state, body)?;
            previous_state = state;
        }

        // A short poll interval also provides simple switch debounce without
        // making the display feel sluggish.
        thread::sleep(Duration::from_millis(30));
    }
}

fn draw_input_state<SPI, DC, RST>(
    display: &mut ST7735<SPI, DC, RST>,
    inputs: HashMap<GpioInput, InputPin>,
    state: Vec<GpioInput>,
    style: MonoTextStyle<'_, Rgb565>,
) -> Result<(), Error>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
    RST: embedded_hal::digital::OutputPin,
{
    Rectangle::new(Point::new(0, 63), Size::new(WIDTH, HEIGHT - 63))
        .into_styled(PrimitiveStyle::with_fill(BACKGROUND))
        .draw(display)
        .map_err(|_| "could not clear input message")?;

    if state.is_empty() {
        Text::with_baseline("None", Point::new(52, 75), style, Baseline::Top)
            .draw(display)
            .map_err(|_| "could not draw input message")?;
    } else {
        // Multiple controls can be shown at once, one per line.
        let mut y = 66;
        for input in state {
            Text::with_baseline(input.to_str(), Point::new(7, y), style, Baseline::Top)
                .draw(display)
                .map_err(|_| "could not draw input message")?;
            y += 12;
        }
    }

    Ok(())
}
