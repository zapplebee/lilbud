#![no_std]
#![no_main]

use rp_pico as bsp;
use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionSpi, Pins},
    pac,
    sio::Sio,
    spi::Spi,
    watchdog::Watchdog,
    Timer,
};

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use fugit::RateExtU32;
use panic_halt as _; // Panic handler

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use st7789::ST7789;

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
    )
    .ok()
    .unwrap();
    let mut timer = Timer::new(peripherals.TIMER, &mut peripherals.RESETS, &clocks);
    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    // #define LCD_DC_PIN 8
    // #define LCD_CS_PIN 9
    // #define LCD_CLK_PIN 10
    // #define LCD_MISO_PIN 12
    // #define LCD_MOSI_PIN 11
    // #define LCD_RST_PIN 12
    // #define LCD_BL_PIN 25

    // #define DEV_SDA_PIN     (6)
    // #define DEV_SCL_PIN     (7)

    // #define BAT_ADC_PIN     (29)
    // #define BAR_CHANNEL     (A3)

    let sck = pins.gpio10.into_function::<FunctionSpi>();
    let mosi = pins.gpio11.into_function::<FunctionSpi>();
    let dc = pins.gpio8.into_push_pull_output();
    let cs = pins.gpio9.into_push_pull_output();
    let rst = pins.gpio12.into_push_pull_output();
    let backlight = pins.gpio25.into_push_pull_output();



// Initialize SPI (using SPI1 in this example)
let spi: Spi<_, _, _, 8> = Spi::new(peripherals.SPI1, (mosi, sck));
let spi = spi.init(
    &mut peripherals.RESETS,
    8_000_000u32.Hz(), // Or whatever frequency works best
    8_000_000u32.Hz(),
    embedded_hal::spi::MODE_0,
);

// Create the SPI interface with CS.
let di = SPIInterface::new(spi, dc, cs);

// Now create your display instance. (Adjust the constructor as needed.)
let mut display = ST7789::new(di, Some(rst), Some(backlight), 240, 240);
    timer.delay_ms(500_u32);
    display.init(&mut timer).unwrap();
    timer.delay_ms(500_u32);

    display.clear(Rgb565::BLUE).unwrap();

    let red_style = PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build();
    // Create a red square, then convert it to a styled primitive.
    let red_square = Rectangle::new(
        Point::new(20, 20),
        Size::new(100, 200),
    )
    .into_styled(red_style);


    let green_style = PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build();
    // Create a red square, then convert it to a styled primitive.
    let green_square = Rectangle::new(
        Point::new(0, 0),
        Size::new(240, 200),
    )
    .into_styled(green_style);


    loop {
        timer.delay_ms(1000_u32);
        red_square.draw(&mut display).unwrap();
        timer.delay_ms(1000_u32);
        green_square.draw(&mut display).unwrap();
    }
}
