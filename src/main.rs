#![no_std]
#![no_main]

use rp2040_boot2;
#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
use embedded_hal::blocking::delay::DelayMs;
use cortex_m_rt::entry;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionSpi, Pins},
    pac,
    sio::Sio,
    spi::Spi,
    watchdog::Watchdog,
    timer::Timer,
};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _; // Panic handler
use fugit::RateExtU32;

mod lcd;
use lcd::LCD;

#[entry]
fn main() -> ! {
    let mut peripherals = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
    let clocks = init_clocks_and_plls(
        12_000_000, 
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    ).ok().unwrap();
    let mut timer = Timer::new(peripherals.TIMER, &mut peripherals.RESETS, &clocks);
    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    // Use the wiring as per the manufacturer's documentation:
    // SCK: GPIO2, MOSI: GPIO3, DC: GPIO8, RST: GPIO12, Backlight: GPIO25
    let sck = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();
    let dc = pins.gpio8.into_push_pull_output();
    let rst = pins.gpio12.into_push_pull_output();
    let mut backlight = pins.gpio25.into_push_pull_output();

    // Turn on the backlight.
    backlight.set_high().unwrap();

    // Initialize SPI (using SPI0 in this example)
    let spi: Spi<_, _, _, 8> = Spi::new(peripherals.SPI0, (mosi, sck));
    let spi = spi.init(
        &mut peripherals.RESETS,
        8_000_000u32.Hz(), // SPI frequency (try lowering if needed)
        8_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );

    // Create our LCD driver (240×240).
    let mut lcd = LCD::new(spi, dc, rst, 240, 240);

    // Initialize the display.
    lcd.init(&mut timer).unwrap();
    timer.delay_ms(500_u32);
    // Clear the display with blue (0x001F in RGB565).
    lcd.clear(0x001F).unwrap();

    loop {

        lcd.draw_pixels([
            (100, 100, 0x07E0),  // green
            (120, 120, 0xF800),  // red
            (130, 130, 0x001F),  // blue
        ]).unwrap();
        // The display remains on with the cleared color.
    }
}
