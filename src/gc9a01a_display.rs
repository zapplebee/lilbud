//! GC9A01A display driver for the Waveshare RP2040-LCD-1.28
//!
//! Pinout (SPI1):
//!   RST  = GPIO12
//!   DC   = GPIO8
//!   CS   = GPIO9
//!   SCK  = GPIO10
//!   MOSI = GPIO11
//!   BL   = GPIO25  (backlight)

use crate::config::{HEIGHT, WIDTH};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::IntoStorage;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_1::spi::SpiBus;
use rp2040_hal as hal;
use hal::clocks::{Clock, ClocksManager};
use hal::fugit::RateExtU32;
use hal::pac::SPI1;
use hal::gpio::{
    bank0::{Gpio8, Gpio9, Gpio10, Gpio11, Gpio12, Gpio25},
    FunctionSio, FunctionSpi, Pin, PullDown, SioOutput,
};
use hal::spi::{Enabled, Spi};
use cortex_m::delay::Delay;

type RstPin  = Pin<Gpio12, FunctionSio<SioOutput>, PullDown>;
type DcPin   = Pin<Gpio8,  FunctionSio<SioOutput>, PullDown>;
type CsPin   = Pin<Gpio9,  FunctionSio<SioOutput>, PullDown>;
type BlPin   = Pin<Gpio25, FunctionSio<SioOutput>, PullDown>;
type MosiPin = Pin<Gpio11, FunctionSpi, PullDown>;
type SckPin  = Pin<Gpio10, FunctionSpi, PullDown>;
type DisplaySpi = Spi<Enabled, SPI1, (MosiPin, SckPin), 8>;

pub struct GC9A01ADisplay {
    spi: DisplaySpi,
    dc:  DcPin,
    _cs: CsPin,
    _bl: BlPin,
}

impl GC9A01ADisplay {
    pub fn new(
        spi_dev: SPI1,
        sck:     SckPin,
        mosi:    MosiPin,
        mut rst: RstPin,
        mut dc:  DcPin,
        mut cs:  CsPin,
        mut bl:  BlPin,
        resets:  &mut hal::pac::RESETS,
        clocks:  &ClocksManager,
        delay:   &mut Delay,
    ) -> Self {
        use embedded_hal::spi::MODE_0;
        let spi = Spi::<_, _, _, 8>::new(spi_dev, (mosi, sck)).init(
            resets,
            clocks.peripheral_clock.freq(),
            40_000_000u32.Hz(),
            MODE_0,
        );

        bl.set_high().unwrap();

        // Hardware reset: high → low → high, then CS permanently low
        rst.set_high().unwrap();
        delay.delay_ms(100);
        rst.set_low().unwrap();
        delay.delay_ms(100);
        rst.set_high().unwrap();
        cs.set_low().unwrap();
        delay.delay_ms(100);

        dc.set_low().unwrap();

        let mut disp = GC9A01ADisplay { spi, dc, _cs: cs, _bl: bl };
        disp.init(delay);
        disp
    }

    // CS stays permanently low; DC toggles to distinguish command vs data.
    fn cmd(&mut self, cmd: u8) {
        self.dc.set_low().unwrap();
        self.spi.write(&[cmd]).unwrap();
    }

    fn data(&mut self, data: &[u8]) {
        self.dc.set_high().unwrap();
        self.spi.write(data).unwrap();
    }

    fn cmd_data(&mut self, cmd: u8, data: &[u8]) {
        self.cmd(cmd);
        if !data.is_empty() {
            self.data(data);
        }
    }

    fn init(&mut self, delay: &mut Delay) {
        self.cmd(0x01); // Software reset
        delay.delay_ms(200);

        self.cmd(0x11); // Sleep out
        delay.delay_ms(200);

        self.cmd_data(0xEF, &[]);
        self.cmd_data(0xEB, &[0x14]);
        self.cmd_data(0xFE, &[]);
        self.cmd_data(0xEF, &[]);
        self.cmd_data(0xEB, &[0x14]);
        self.cmd_data(0x84, &[0x40]);
        self.cmd_data(0x85, &[0xFF]);
        self.cmd_data(0x86, &[0xFF]);
        self.cmd_data(0x87, &[0xFF]);
        self.cmd_data(0x88, &[0x0A]);
        self.cmd_data(0x89, &[0x21]);
        self.cmd_data(0x8A, &[0x00]);
        self.cmd_data(0x8B, &[0x80]);
        self.cmd_data(0x8C, &[0x01]);
        self.cmd_data(0x8D, &[0x01]);
        self.cmd_data(0x8E, &[0xFF]);
        self.cmd_data(0x8F, &[0xFF]);
        self.cmd_data(0xB6, &[0x00, 0x20]);
        self.cmd_data(0x36, &[0x08]); // MADCTL
        self.cmd_data(0x3A, &[0x05]); // Pixel format: RGB565
        self.cmd_data(0x90, &[0x08, 0x08, 0x08, 0x08]);
        self.cmd_data(0xBD, &[0x06]);
        self.cmd_data(0xBC, &[0x00]);
        self.cmd_data(0xFF, &[0x60, 0x01, 0x04]);
        self.cmd_data(0xC3, &[0x13]);
        self.cmd_data(0xC4, &[0x13]);
        self.cmd_data(0xC9, &[0x22]);
        self.cmd_data(0xBE, &[0x11]);
        self.cmd_data(0xE1, &[0x10, 0x0E]);
        self.cmd_data(0xDF, &[0x21, 0x0C, 0x02]);
        self.cmd_data(0xF0, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]);
        self.cmd_data(0xF1, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]);
        self.cmd_data(0xF2, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]);
        self.cmd_data(0xF3, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]);
        self.cmd_data(0xED, &[0x1B, 0x0B]);
        self.cmd_data(0xAE, &[0x77]);
        self.cmd_data(0xCD, &[0x63]);
        self.cmd_data(0x70, &[0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03]);
        self.cmd_data(0xE8, &[0x34]);
        self.cmd_data(0x62, &[0x18, 0x0D, 0x71, 0xED, 0x70, 0x70,
                               0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70]);
        self.cmd_data(0x63, &[0x18, 0x11, 0x71, 0xF1, 0x70, 0x70,
                               0x18, 0x13, 0x71, 0xF3, 0x70, 0x70]);
        self.cmd_data(0x64, &[0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07]);
        self.cmd_data(0x66, &[0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00]);
        self.cmd_data(0x67, &[0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98]);
        self.cmd_data(0x74, &[0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00]);
        self.cmd_data(0x98, &[0x3E, 0x07]);
        self.cmd_data(0x35, &[]);
        self.cmd_data(0x21, &[]);

        self.cmd(0x29); // Display on
        delay.delay_ms(20);
    }

    /// Fill the entire display with a single RGB565 color.
    pub fn fill(&mut self, color: u16) {
        let pixel = [(color >> 8) as u8, color as u8];

        self.cmd_data(0x2A, &[0x00, 0x00, 0x00, 0xEF]); // CASET 0..239
        self.cmd_data(0x2B, &[0x00, 0x00, 0x00, 0xEF]); // RASET 0..239
        self.cmd(0x2C);                                   // RAMWR

        self.dc.set_high().unwrap();
        for _ in 0..(WIDTH * HEIGHT) {
            self.spi.write(&pixel).unwrap();
        }
    }

    /// Write the full 240x240 Rgb565 framebuffer to the display.
    pub fn flush(&mut self, buffer: &[Rgb565; WIDTH * HEIGHT]) {
        self.cmd_data(0x2A, &[0x00, 0x00, 0x00, 0xEF]); // CASET 0..239
        self.cmd_data(0x2B, &[0x00, 0x00, 0x00, 0xEF]); // RASET 0..239
        self.cmd(0x2C);                                   // RAMWR

        self.dc.set_high().unwrap();
        let mut row = [0u8; WIDTH * 2];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let raw = buffer[y * WIDTH + x].into_storage();
                row[x * 2]     = (raw >> 8) as u8;
                row[x * 2 + 1] = raw as u8;
            }
            self.spi.write(&row).unwrap();
        }
    }
}
