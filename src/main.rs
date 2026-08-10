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
use std::{thread, time::Duration};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

// Waveshare 1.44inch LCD HAT pin numbers use BCM numbering.
const DC_PIN: u8 = 25;
const RESET_PIN: u8 = 27;
const BACKLIGHT_PIN: u8 = 24;

// Joystick and button pins on the Waveshare 1.44inch LCD HAT (BCM numbering).
const JOYSTICK_UP_PIN: u8 = 6;
const JOYSTICK_DOWN_PIN: u8 = 19;
const JOYSTICK_LEFT_PIN: u8 = 5;
const JOYSTICK_RIGHT_PIN: u8 = 26;
const JOYSTICK_PRESS_PIN: u8 = 13;
const BUTTON_1_PIN: u8 = 21;
const BUTTON_2_PIN: u8 = 20;
const BUTTON_3_PIN: u8 = 16;

const BACKGROUND: Rgb565 = Rgb565::new(0, 4, 10);

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    // SPI0 CE0 maps to the HAT's SCLK=BCM11, MOSI=BCM10, CS=BCM8.
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
    let spi = SimpleHalSpiDevice::new(spi);

    let gpio = Gpio::new()?;
    let mut dc = gpio.get(DC_PIN)?.into_output_low();
    let mut reset = gpio.get(RESET_PIN)?.into_output_high();
    let mut backlight = gpio.get(BACKLIGHT_PIN)?.into_output_high();

    // The controls connect their GPIO to ground when pressed, so enable the
    // Pi's internal pull-up resistors and treat a low level as pressed.
    let inputs = [
        (
            "Joystick UP",
            gpio.get(JOYSTICK_UP_PIN)?.into_input_pullup(),
        ),
        (
            "Joystick DOWN",
            gpio.get(JOYSTICK_DOWN_PIN)?.into_input_pullup(),
        ),
        (
            "Joystick LEFT",
            gpio.get(JOYSTICK_LEFT_PIN)?.into_input_pullup(),
        ),
        (
            "Joystick RIGHT",
            gpio.get(JOYSTICK_RIGHT_PIN)?.into_input_pullup(),
        ),
        (
            "Joystick PRESS",
            gpio.get(JOYSTICK_PRESS_PIN)?.into_input_pullup(),
        ),
        ("Button 1", gpio.get(BUTTON_1_PIN)?.into_input_pullup()),
        ("Button 2", gpio.get(BUTTON_2_PIN)?.into_input_pullup()),
        ("Button 3", gpio.get(BUTTON_3_PIN)?.into_input_pullup()),
    ];

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
        .set_orientation(&Orientation::Portrait)
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

    let mut previous_state = u8::MAX;
    loop {
        let state = input_state(&inputs);
        if state != previous_state {
            draw_input_state(&mut display, &inputs, state, body)?;
            previous_state = state;
        }

        // A short poll interval also provides simple switch debounce without
        // making the display feel sluggish.
        thread::sleep(Duration::from_millis(30));
    }
}

fn input_state(inputs: &[(&'static str, InputPin)]) -> u8 {
    inputs
        .iter()
        .enumerate()
        .fold(0, |state, (index, (_, pin))| {
            state | ((pin.is_low() as u8) << index)
        })
}

fn draw_input_state<SPI, DC, RST>(
    display: &mut ST7735<SPI, DC, RST>,
    inputs: &[(&'static str, InputPin)],
    state: u8,
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

    if state == 0 {
        Text::with_baseline("None", Point::new(52, 75), style, Baseline::Top)
            .draw(display)
            .map_err(|_| "could not draw input message")?;
    } else {
        // Multiple controls can be shown at once, one per line.
        let mut y = 66;
        for (index, (name, _)) in inputs.iter().enumerate() {
            if state & (1 << index) != 0 {
                Text::with_baseline(name, Point::new(7, y), style, Baseline::Top)
                    .draw(display)
                    .map_err(|_| "could not draw input message")?;
                y += 12;
            }
        }
    }

    Ok(())
}
