//! Native WebView window using the OS's built-in browser engine:
//!   macOS  — WKWebView
//!   Windows — WebView2
//!   Linux   — WebKitGTK
//!
//! The WASM binary and JS glue are embedded at compile time.
//! Run `make wasm` before `make webview` to generate the `pkg/` assets.
//!
//! # Threading model
//!
//! `WebView` is not `Send`. All WebView interaction must happen on the main
//! thread inside the event loop handler.
//!
//! Host → WebView: background threads send `UserEvent::ScriptToRun(script)`
//! via `EventLoopProxy`. The event loop calls `webview.evaluate_script()`.
//!
//! WebView → Host: JS calls `window.ipc.postMessage("...")`, wry fires the
//! `with_ipc_handler` closure on the main thread.

use std::borrow::Cow;
use std::thread;
use std::time::Duration;
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

/// Events sent from background threads to the main event loop.
/// The event loop calls `webview.evaluate_script()` for each one.
#[derive(Debug)]
pub enum UserEvent {
    /// Run an arbitrary JS snippet in the WebView.
    ScriptToRun(String),
}

pub fn run() {
    let event_loop = EventLoop::<UserEvent>::with_user_event();
    let proxy = event_loop.create_proxy();

    // Proof-of-concept background thread: sends a JS ping to the WebView
    // every 2 seconds. Replace with real serial / board logic in Step 5.
    thread::spawn(move || {
        let mut n = 0u32;
        loop {
            thread::sleep(Duration::from_secs(2));
            let script = format!("console.log('host ping {n}')");
            if proxy.send_event(UserEvent::ScriptToRun(script)).is_err() {
                break; // event loop exited
            }
            n += 1;
        }
    });

    let window = WindowBuilder::new()
        .with_title("lilbud")
        .with_inner_size(wry::application::dpi::LogicalSize::new(280u32, 280u32))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let webview = WebViewBuilder::new(window)
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
        .with_ipc_handler(|_window, msg| {
            // WebView → Host: JS called window.ipc.postMessage("...")
            eprintln!("[ipc] received from JS: {msg}");
        })
        .with_url("app://localhost/")
        .unwrap()
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::ScriptToRun(script)) => {
                // Host → WebView: run JS on the main thread
                if let Err(e) = webview.evaluate_script(&script) {
                    eprintln!("[webview] evaluate_script error: {e}");
                }
            }
            _ => {}
        }
    });
}
