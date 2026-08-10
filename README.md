# Pi Zero 2 W + Waveshare 1.44-inch LCD HAT

This example displays a live input tester on Waveshare's 128×128 ST7735S LCD
HAT. Move or press the joystick, or press any of the three buttons, and the
control name appears on the screen. Multiple simultaneous inputs are supported.

## Pin mapping

The HAT connects directly to the Pi's 40-pin header. These are BCM GPIO numbers,
not physical header positions:

| LCD signal | BCM GPIO | Physical pin |
| --- | ---: | ---: |
| SCLK | 11 | 23 |
| MOSI | 10 | 19 |
| CS / CE0 | 8 | 24 |
| DC | 25 | 22 |
| RESET | 27 | 13 |
| Backlight | 24 | 18 |

The HAT's controls use these active-low inputs (with internal pull-ups enabled):

| Control | BCM GPIO | Physical pin |
| --- | ---: | ---: |
| Joystick up | 6 | 31 |
| Joystick down | 19 | 35 |
| Joystick left | 5 | 29 |
| Joystick right | 26 | 37 |
| Joystick press | 13 | 33 |
| Button 1 | 21 | 40 |
| Button 2 | 20 | 38 |
| Button 3 | 16 | 36 |

## Raspberry Pi setup

Enable SPI once, then reboot:

```sh
sudo raspi-config
# Interface Options -> SPI -> Yes
sudo reboot
```

Confirm that the kernel SPI device exists:

```sh
ls -l /dev/spidev0.0
```

Install Rust if needed, then run the example from this directory:

```sh
cargo run --release
```

The tester runs continuously; press `Ctrl-C` to stop it.

GPIO and SPI access depend on the Raspberry Pi OS user/group configuration. If
you get `Permission denied`, try `sudo -E cargo run --release` as a quick test.

## Troubleshooting

- A lit but blank screen usually means SPI is disabled or the DC/reset pins do
  not match the board.
- If the image is shifted by a pixel, adjust `display.set_offset(1, 0)` in
  `src/main.rs`; controller/panel revisions can differ.
- If colors look swapped, change the first boolean passed to `ST7735::new` from
  `true` (RGB) to `false` (BGR).
- This code targets the Waveshare **1.44inch LCD HAT**, 128×128, ST7735S. Other
  Waveshare 1.44-inch modules can use different GPIO pins.
