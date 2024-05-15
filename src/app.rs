#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use tracing::info;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Debug, Default)]
pub struct App {
    window: Option<Window>,
    width: u32,
    height: u32,
    title: String,
}

impl App {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }
}

/// @link https://docs.rs/winit/latest/winit/index.html
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

        let window = event_loop.create_window(window_attributes).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            let _ = window.request_inner_size(PhysicalSize::new(self.width, self.height));

            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let container = doc.get_element_by_id("wgpu-wasm")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    let canvas = canvas
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .map_err(|_| ())
                        .unwrap();

                    canvas.set_width(self.width);
                    canvas.set_height(self.height);

                    container.append_child(&canvas).ok()?;

                    Some(())
                })
                .expect("Couldn't append canvas to element!");
        }

        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("Close requested, stopping …");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let KeyEvent {
                    physical_key,
                    logical_key,
                    text,
                    location: _,
                    state,
                    repeat: _,
                    ..
                } = event;

                info!(
                    "KeyBoard> key_code: {:?} | logical_key {:?} | text: {:?} | state {:?}",
                    physical_key, logical_key, text, state
                );
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                info!("MouseButton> state: {:?} | button: {:?}", state, button);
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
            } => {
                info!("MouseWheel> delta: {:?} | touchphase: {:?}", delta, phase);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                info!("MouseCursor> position: {:?}", position);
            }
            _ => (),
        }
    }
}
