use js_resized_event_channel::{JsResizeEventChannel, ResizeEventChannel};
use pollster::FutureExt;
use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureUsages,
};
use winit::window::Window;

pub struct Screen<'a> {
    pub window: &'a Window,
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub resized_event_channel: JsResizeEventChannel,
}

#[derive(Default)]
pub struct ScreenDescriptor {
    pub power_preference: PowerPreference,
    pub memory_hints: MemoryHints,
}

impl<'a> Screen<'a> {
    pub fn new(window: &'a Window, screen_desc: ScreenDescriptor) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: screen_desc.power_preference,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        let mut limits = Limits::downlevel_webgl2_defaults();
                        limits.max_texture_dimension_1d = 4096;
                        limits.max_texture_dimension_2d = 4096;
                        limits.max_texture_dimension_3d = 256;
                        limits.max_uniform_buffer_binding_size = 16 * 1024;
                        limits.using_resolution(adapter.limits())
                    } else {
                        let mut limits = Limits::default().using_resolution(adapter.limits());
                        limits.max_uniform_buffer_binding_size = 16 * 1024;
                        limits
                    },
                    memory_hints: screen_desc.memory_hints,
                },
                None, //trace_path
            )
            .block_on()
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        {
            size.width = size.width.max(1);
            size.height = size.height.max(1);
        }
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        let resized_event_channel = JsResizeEventChannel::init(&window);
        Self {
            window: &window,
            surface,
            device,
            queue,
            config,
            resized_event_channel,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width / 16;
        self.config.height = height / 16;
        self.surface.configure(&self.device, &self.config);
        #[cfg(target_arch = "wasm32")]
        let _result = self.window.request_inner_size(size);
    }

    pub fn try_resize_by_js_event(&mut self) {
        if let Some(size) = self.resized_event_channel.try_recv_resized_event() {
            self.resize(size.width as u32, size.height as u32);
        }
    }
}
