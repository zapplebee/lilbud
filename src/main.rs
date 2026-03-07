#![cfg_attr(feature = "embedded", no_std)]
#![cfg_attr(feature = "embedded", no_main)]

mod config;
mod get_faces;
mod ui;

#[cfg(feature = "embedded")]
mod face_data;

#[cfg(feature = "desktop")]
mod sdl2_display;

#[cfg(feature = "embedded")]
mod gc9a01a_display;

// ── Desktop entry point ───────────────────────────────────────────────────────

#[cfg(feature = "desktop")]
fn main() -> Result<(), String> {
    use embedded_graphics::pixelcolor::Rgb565;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2_display::SDL2Display;
    use std::time::{Duration, Instant};

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut display = SDL2Display::new(&video_subsystem);

    let mut last_face_change = Instant::now();
    let mut buffer = [Rgb565::CSS_BLACK; config::WIDTH * config::HEIGHT];

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        if last_face_change.elapsed() >= Duration::from_secs(3) {
            ui::set_face();
            last_face_change = Instant::now();
        }

        ui::tick();
        ui::draw_ui(&mut buffer);
        display.flush(&buffer);

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

// ── Embedded entry point ──────────────────────────────────────────────────────

#[cfg(feature = "embedded")]
use panic_halt as _;

#[cfg(feature = "embedded")]
use rp2040_hal as hal;

#[cfg(feature = "embedded")]
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[cfg(feature = "embedded")]
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Static framebuffer — keeps 115KB off the stack.
#[cfg(feature = "embedded")]
static mut FRAMEBUFFER: core::mem::MaybeUninit<
    [embedded_graphics::pixelcolor::Rgb565; config::WIDTH * config::HEIGHT],
> = core::mem::MaybeUninit::uninit();

#[cfg(feature = "embedded")]
#[cortex_m_rt::entry]
fn main() -> ! {
    use gc9a01a_display::GC9A01ADisplay;
    use hal::clocks::Clock;
    use hal::gpio::FunctionSpi;
    use hal::pac;
    use hal::Timer;

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
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
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut display = GC9A01ADisplay::new(
        pac.SPI1,
        pins.gpio10.into_function::<FunctionSpi>(), // SCK
        pins.gpio11.into_function::<FunctionSpi>(), // MOSI
        pins.gpio12.into_push_pull_output(),        // RST
        pins.gpio8.into_push_pull_output(),         // DC
        pins.gpio9.into_push_pull_output(),         // CS
        pins.gpio25.into_push_pull_output(),        // BL
        &mut pac.RESETS,
        &clocks,
        &mut delay,
    );

    crate::get_faces::seed_rng(timer.get_counter().ticks() as u32);
    ui::set_face();

    let mut last_face_change = timer.get_counter();

    loop {
        let now = timer.get_counter();
        if now.checked_duration_since(last_face_change)
            .map(|d| d.to_secs() >= 3)
            .unwrap_or(false)
        {
            ui::set_face();
            last_face_change = now;
        }

        ui::tick();
        unsafe {
            ui::draw_ui(FRAMEBUFFER.assume_init_mut());
            display.flush(FRAMEBUFFER.assume_init_ref());
        }
    }
}
