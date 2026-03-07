//! Native WebView window using the OS's built-in browser engine:
//!   macOS  — WKWebView
//!   Windows — WebView2
//!   Linux   — WebKitGTK
//!
//! The WASM binary and JS glue are embedded at compile time.
//! Run `make wasm` before `make webview` to generate the `pkg/` assets.

use std::borrow::Cow;
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    http::header::CONTENT_TYPE,
    webview::WebViewBuilder,
};

// Embedded at compile time. `cargo build --features webview` will fail
// with a clear error if `make wasm` has not been run first.
const INDEX_HTML:  &[u8] = include_bytes!("../index.html");
const LILBUD_JS:   &[u8] = include_bytes!("../pkg/lilbud.js");
const LILBUD_WASM: &[u8] = include_bytes!("../pkg/lilbud_bg.wasm");

pub fn run() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("lilbud")
        .with_inner_size(wry::application::dpi::LogicalSize::new(280u32, 280u32))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let _webview = WebViewBuilder::new(window)
        .unwrap()
        .with_custom_protocol("app".into(), |request| {
            let path = request.uri().path();
            let path = if path == "/" { "/index.html" } else { path };

            let (body, mime): (&[u8], &str) = match path {
                "/index.html"        => (INDEX_HTML,  "text/html; charset=utf-8"),
                "/pkg/lilbud.js"     => (LILBUD_JS,   "text/javascript"),
                "/pkg/lilbud_bg.wasm"=> (LILBUD_WASM, "application/wasm"),
                _                    => (b"Not Found", "text/plain"),
            };

            wry::http::Response::builder()
                .header(CONTENT_TYPE, mime)
                .body(Cow::Borrowed(body))
                .map_err(Into::into)
        })
        .with_url("app://localhost/")
        .unwrap()
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit;
        }
    });
}
