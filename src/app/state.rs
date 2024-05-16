use std::sync::Arc;

use wgpu::PresentMode;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

#[derive(Debug)]
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    #[cfg(target_arch = "wasm32")]
    /// reference to window for retrieving window and it's size in method `Self.resize(…)`
    window: Arc<Window>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // The surface needs to live as long as the window that created it.
        // https://github.com/rust-windowing/winit/issues/3626#issuecomment-2081794856
        // https://users.rust-lang.org/t/in-wgpu-how-do-i-reset-a-buffer-after-making-it-device-create-buffer-init/106391/13
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        // alternatively use instance.enumerate_adapters to manually check for a proper adapter
        // https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // use adapter.features() or device.features() to get a list of supported features
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    #[cfg(target_arch = "wasm32")]
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    #[cfg(not(target_arch = "wasm32"))]
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // shortened form to create a wgpu::ShaderModuleDescriptor
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            #[cfg(target_arch = "wasm32")]
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn window(&self) -> Arc<Window> {
        Arc::clone(&self.window)
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let surface_width;
        let surface_height;

        // A Desktop Window in Xwayland also wants the physical size as texture size.
        // Otherwise there will be a black border after scaling it down with a scale factor.
        // TODO check scaling in a X11 session and in a Wayland Desktop Window
        #[cfg(not(target_arch = "wasm32"))]
        {
            surface_width = new_size.width;
            surface_height = new_size.height;
        }

        // Web platform with a scale factor 2.0 would double the canvas size and
        // trigger further resize events until surface width and height are greater than
        // the maximum supported texture size (e.g. 2048x2048).
        // Keep the desired physical size, but scale the texture down to it's desired size.
        #[cfg(target_arch = "wasm32")]
        {
            let scale_factor = self.window().as_ref().scale_factor();
            surface_width = ((new_size.width as f64) / scale_factor) as u32;
            surface_height = ((new_size.height as f64) / scale_factor) as u32;
        }

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = surface_width;
            self.config.height = surface_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Returns a bool to indicate whether an event has been fully processed.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let _ = event;
        // We're just going to return false for now because we don't have any events we want to capture.
        false
    }

    pub fn update(&mut self) {
        // todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
