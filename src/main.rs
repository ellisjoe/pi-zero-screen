use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Baseline, Text},
};
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use st7735_lcd::{Orientation, ST7735};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

// Waveshare 1.44inch LCD HAT pin numbers use BCM numbering.
const DC_PIN: u8 = 25;
const RESET_PIN: u8 = 27;
const BACKLIGHT_PIN: u8 = 24;

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    // SPI0 CE0 maps to the HAT's SCLK=BCM11, MOSI=BCM10, CS=BCM8.
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
    let spi = SimpleHalSpiDevice::new(spi);

    let gpio = Gpio::new()?;
    let mut dc = gpio.get(DC_PIN)?.into_output_low();
    let mut reset = gpio.get(RESET_PIN)?.into_output_high();
    let mut backlight = gpio.get(BACKLIGHT_PIN)?.into_output_high();

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
        .clear(Rgb565::new(0, 4, 10))
        .map_err(|_| "could not clear display")?;

    let heading = MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN);
    let body = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::with_baseline("Pi Zero 2 W", Point::new(27, 35), heading, Baseline::Top)
        .draw(&mut display)
        .map_err(|_| "could not draw heading")?;
    Text::with_baseline("Hello from Rust!", Point::new(16, 60), body, Baseline::Top)
        .draw(&mut display)
        .map_err(|_| "could not draw text")?;

    println!("Text drawn to the Waveshare LCD.");
    Ok(())
}
