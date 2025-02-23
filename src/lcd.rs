#![no_std]

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;

/// Minimal driver for an ST7789-based 240×240 display (e.g. Waveshare RP2040‑LCD‑1.28)
pub struct LCD<SPI, DC, RST>
where
    SPI: Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    spi: SPI,
    dc: DC,
    rst: RST,
    width: u16,
    height: u16,
}

impl<SPI, DC, RST> LCD<SPI, DC, RST>
where
    SPI: Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Create a new LCD driver instance.
    /// For the RP2040‑LCD‑1.28, width and height are both 240.
    pub fn new(spi: SPI, dc: DC, rst: RST, width: u16, height: u16) -> Self {
        Self {
            spi,
            dc,
            rst,
            width,
            height,
        }
    }

    /// Initialize the display with a minimal command sequence.
    ///
    /// This sequence:
    ///  1. Toggles the reset pin
    ///  2. Sends a software reset (0x01)
    ///  3. Sends Sleep Out (0x11)
    ///  4. Sets MADCTL (0x36) with parameter 0x70
    ///  5. Sets color mode to 16-bit (0x3A with parameter 0x05)
    ///  6. Turns the display on (0x29)
    pub fn init<D: DelayMs<u8>>(&mut self, delay: &mut D) -> Result<(), SPI::Error> {
        // Hardware reset.
        self.rst.set_high().ok();
        delay.delay_ms(100);
        self.rst.set_low().ok();
        delay.delay_ms(100);
        self.rst.set_high().ok();
        delay.delay_ms(100);

        // Software reset.
        self.write_command(0x01)?;
        delay.delay_ms(150);

        // Sleep Out.
        self.write_command(0x11)?;
        delay.delay_ms(150);

        // MADCTL – set orientation; parameter 0x70 as per known‑working code.
        self.write_command(0x36)?;
        self.write_data(&[0x70])?;

        // Color mode – 16-bit color.
        self.write_command(0x3A)?;
        self.write_data(&[0x05])?;

        // Display On.
        self.write_command(0x29)?;
        delay.delay_ms(100);

        Ok(())
    }

    /// Clear the entire display with the given 16-bit RGB565 color.
    pub fn clear(&mut self, color: u16) -> Result<(), SPI::Error> {
        // Set column address (0x2A)
        self.write_command(0x2A)?;
        let col_data = [
            0x00,
            0x00,
            ((self.width - 1) >> 8) as u8,
            ((self.width - 1) & 0xFF) as u8,
        ];
        self.write_data(&col_data)?;

        // Set row address (0x2B)
        self.write_command(0x2B)?;
        let row_data = [
            0x00,
            0x00,
            ((self.height - 1) >> 8) as u8,
            ((self.height - 1) & 0xFF) as u8,
        ];
        self.write_data(&row_data)?;

        // Write Memory (0x2C)
        self.write_command(0x2C)?;

        let hi = (color >> 8) as u8;
        let lo = (color & 0xFF) as u8;
        let total_pixels = (self.width as usize) * (self.height as usize);
        const CHUNK_SIZE: usize = 128; // pixels per chunk

        // Build a buffer for CHUNK_SIZE pixels (each pixel is 2 bytes)
        let mut buf = [0u8; CHUNK_SIZE * 2];
        for i in 0..CHUNK_SIZE {
            buf[i * 2] = hi;
            buf[i * 2 + 1] = lo;
        }

        let mut sent = 0;
        while sent < total_pixels {
            self.write_data(&buf)?;
            sent += CHUNK_SIZE;
        }
        Ok(())
    }

    /// Draw a single pixel at (x, y) with the given 16-bit RGB565 color.
    pub fn draw_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), SPI::Error> {
        if x >= self.width || y >= self.height {
            return Ok(()); // or return an error
        }

        // Set column address to x.
        self.write_command(0x2A)?;
        let col_data = [
            0x00,
            0x00,
            (x >> 8) as u8,
            (x & 0xFF) as u8,
        ];
        self.write_data(&col_data)?;

        // Set row address to y.
        self.write_command(0x2B)?;
        let row_data = [
            0x00,
            0x00,
            (y >> 8) as u8,
            (y & 0xFF) as u8,
        ];
        self.write_data(&row_data)?;

        // Memory Write command.
        self.write_command(0x2C)?;
        // Send pixel data.
        let hi = (color >> 8) as u8;
        let lo = (color & 0xFF) as u8;
        self.write_data(&[hi, lo])
    }

    /// Draw multiple pixels specified as an iterator over (x, y, color) tuples.
    pub fn draw_pixels<I>(&mut self, pixels: I) -> Result<(), SPI::Error>
    where
        I: IntoIterator<Item = (u16, u16, u16)>,
    {
        for (x, y, color) in pixels.into_iter() {
            self.draw_pixel(x, y, color)?;
        }
        Ok(())
    }

    /// Write a command byte to the display.
    fn write_command(&mut self, command: u8) -> Result<(), SPI::Error> {
        self.dc.set_low().ok();
        self.spi.write(&[command])
    }

    /// Write data bytes to the display.
    fn write_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.dc.set_high().ok();
        self.spi.write(data)
    }
}
