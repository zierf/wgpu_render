mod state;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use tracing::info;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use state::State;

#[derive(Debug)]
pub struct App {
    width: u32,
    height: u32,
    title: String,
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl App {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            window: None,
            state: None,
        }
    }
}

/// @link https://docs.rs/winit/0.30.0/winit/index.html
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            // web import trait method WindowExtWebSys::canvas(&self)
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

        // FIXME requesting adapter and device return futures, but ApplicationHandler::resumed is synchronous
        // FIXME Check WASM Error "already borrowed: BorrowMutError" after closing Tab in Firefox (not on refresh!)
        let state = pollster::block_on(State::new(Arc::clone(&window)));

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        let state = self.state.as_mut().unwrap();

        // WindowEvent has a WindowId member. In multi-window environments, it should be compared
        // to the value returned by Window::id() to determine which Window dispatched the event.
        // https://docs.rs/winit/latest/winit/#event-handling
        if window_id != window.id() {
            return;
        }

        // If the method returns true, the main loop won't process the event any further.
        if state.input(&event) {
            return;
        }

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
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _, ..
            } => {
                state.resize(PhysicalSize::new(self.width, self.height));
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw
                let size = state.size();

                state.update();

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }

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
