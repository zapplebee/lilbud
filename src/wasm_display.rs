//! WASM display: renders the RGB565 framebuffer to a `<canvas id="canvas">` element
//! via the Canvas 2D API, driven by `requestAnimationFrame`.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

use crate::config::{HEIGHT, WIDTH};
use crate::ui;

fn window() -> Window {
    web_sys::window().expect("no global window exists")
}

fn request_animation_frame(closure: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .unwrap();
}

fn canvas_ctx() -> CanvasRenderingContext2d {
    window()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .expect("no element with id=\"canvas\"")
        .dyn_into::<HtmlCanvasElement>()
        .unwrap()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap()
}

/// Convert the big-endian RGB565 framebuffer to RGBA and blit it to the canvas.
fn flush(buffer: &[u8; WIDTH * HEIGHT * 2]) {
    let ctx = canvas_ctx();
    let mut rgba = vec![0u8; WIDTH * HEIGHT * 4];

    for (i, chunk) in buffer.chunks_exact(2).enumerate() {
        let raw = ((chunk[0] as u16) << 8) | chunk[1] as u16;
        let r5 = ((raw >> 11) & 0x1F) as u8;
        let g6 = ((raw >>  5) & 0x3F) as u8;
        let b5 = ( raw        & 0x1F) as u8;
        // Expand to 8 bits
        rgba[i * 4]     = (r5 << 3) | (r5 >> 2);
        rgba[i * 4 + 1] = (g6 << 2) | (g6 >> 4);
        rgba[i * 4 + 2] = (b5 << 3) | (b5 >> 2);
        rgba[i * 4 + 3] = 255;
    }

    let image_data = ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&rgba),
        WIDTH as u32,
        HEIGHT as u32,
    )
    .unwrap();

    ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
}

/// Start the animation loop. Called once from `#[wasm_bindgen(start)]`.
pub fn animate() {
    ui::set_face();

    // Heap-allocate the framebuffer — 115,200 bytes is too large for the WASM stack.
    let mut buffer = Box::new([0u8; WIDTH * HEIGHT * 2]);
    let mut last_face_ms = js_sys::Date::now();

    // Standard Rust/WASM recursive-closure pattern for requestAnimationFrame.
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = js_sys::Date::now();
        if now - last_face_ms >= 3000.0 {
            ui::set_face();
            last_face_ms = now;
        }

        ui::tick();
        ui::draw_ui(&mut buffer);
        flush(&buffer);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
