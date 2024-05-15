mod app;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use cfg_if::cfg_if;
use tracing::{error, info, warn};

use winit::event_loop::{ControlFlow, EventLoop};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run_app() {
    let log_level = tracing::Level::WARN;

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            // tracing for all log levels
            // tracing_wasm::set_as_global_default();

            let wasm_layer_config = tracing_wasm::WASMLayerConfigBuilder::new().set_max_level(log_level).build();
            tracing_wasm::set_as_global_default_with_config(wasm_layer_config);
        } else {
            // tracing for all log levels
            // tracing_subscriber::fmt::init();

            use tracing_subscriber::FmtSubscriber;
            let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
            tracing::subscriber::set_global_default(subscriber).expect("setting default tracing subscriber failed!");
        }
    }

    info!("Example Info!");
    warn!("Example Warning!!");
    error!("Example Error!!!");

    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    #[allow(unused_mut)]
    let mut app = app::App::new("WebGPU Rendering", 500, 400);

    #[cfg(target_arch = "wasm32")]
    {
        // web import trait method EventLoopExtWebSys::spawn_app(app: App)
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run_app(&mut app).unwrap();
}
