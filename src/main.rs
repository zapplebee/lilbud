#![no_std]
#![no_main]

mod get_faces;
mod ui;

use panic_halt as _;
use rand_xorshift::XorShiftRng;
use rp_pico as bsp;

use bsp::entry;
use fugit::RateExtU32;

use display_interface_spi::SPIInterface;
use embedded_graphics::{prelude::*, primitives::Rectangle};

use rand_core::SeedableRng;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::FunctionSpi,
    pac, pwm,
    sio::Sio,
    watchdog::Watchdog,
    Spi,
};

#[entry]
fn main() -> ! {
    let mut rng = XorShiftRng::from_seed([0; 16]);
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sclk = pins.gpio10.into_function::<FunctionSpi>();
    let mosi = pins.gpio11.into_function::<FunctionSpi>();

    let spi_cs = pins.gpio9.into_push_pull_output();

    let spi: Spi<_, _, _, 8> = Spi::new(pac.SPI1, (mosi, sclk));

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc_pin = pins.gpio8.into_push_pull_output();
    let rst_pin = pins.gpio12.into_push_pull_output();

    let spi_interface = SPIInterface::new(spi, dc_pin, spi_cs);

    // initialize PWM for backlight
    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM6
    let mut pwm = pwm_slices.pwm4;
    pwm.set_ph_correct();
    pwm.enable();

    let mut channel = pwm.channel_b;
    // pins.led corresponds to gpio25, the backlight on the board.
    channel.output_to(pins.led);

    // Create display driver
    let mut display = gc9a01a::GC9A01A::new(spi_interface, rst_pin, channel);
    // Bring out of reset
    display.reset(&mut delay).unwrap();
    // Turn on backlight
    display.set_backlight(55000);
    // Initialize registers
    display.initialize(&mut delay).unwrap();
    let area = Rectangle::new(Point::zero(), Size::new(240, 240));

    loop {
        let buffer = ui::draw_ui(&mut rng);
        let _ = display.fill_contiguous(&area, buffer.iter().copied());
    }
}
