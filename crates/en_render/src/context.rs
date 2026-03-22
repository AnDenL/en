use std::sync::Arc;
use winit::window::Window;
use crate::config::{RenderSettings, PowerPreference};

pub struct RenderContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    
    // Surface та Config є Option, бо в редакторі ми малюємо не у вікно, 
    // а у віртуальну текстуру (TextureView).
    pub surface: Option<wgpu::Surface<'static>>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    
    pub render_format: wgpu::TextureFormat,
}

impl RenderContext {
    /// Ініціалізація для самостійної гри (з вікном)
    pub async fn new(window: Arc<Window>, settings: &RenderSettings) -> Result<Self, &'static str> {
        let size = window.inner_size();
        
        // 1. Створюємо інстанс WGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. Створюємо поверхню для малювання на основі вікна
        let surface = instance.create_surface(window.clone()).map_err(|_| "Failed to create surface")?;

        // 3. Шукаємо відеокарту
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: match settings.power_preference {
                PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
                PowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
            },
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        // Тепер це Result, тому обробляємо помилку через map_err.
        // Додаємо .to_string(), щоб уникнути помилки "cannot infer type".
        .map_err(|_| "Failed to find an appropriate adapter".to_string()).unwrap();

        // 4. Отримуємо логічний пристрій та чергу команд
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("EnEngine Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            }
        )
        .await
        // Аналогічно перетворюємо помилку в String
        .map_err(|_| "Failed to create device".to_string()).unwrap();
        // 5. Налаштовуємо поверхню
        let surface_caps = surface.get_capabilities(&adapter);
        
        // Шукаємо sRGB формат, щоб кольори були правильними
        let render_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if settings.vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width.max(1), // Запобігаємо крашу при нульовому розмірі вікна
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: Some(surface),
            config: Some(config),
            render_format,
        })
    }

    /// Ініціалізація для редактора (без власного вікна). 
    /// Редактор (через en_core) сам створить Device і передасть його нам.
    pub fn new_headless(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        render_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            device,
            queue,
            surface: None,
            config: None,
            render_format,
        }
    }

    /// Оновлення розміру поверхні при ресайзі вікна
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let (Some(config), Some(surface)) = (&mut self.config, &self.surface) {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, config);
            }
        }
    }

    /// Увімкнення/вимкнення VSync "на льоту"
    pub fn set_vsync(&mut self, vsync: bool) {
        if let (Some(config), Some(surface)) = (&mut self.config, &self.surface) {
            let mode = if vsync { wgpu::PresentMode::Fifo } else { wgpu::PresentMode::AutoNoVsync };
            if config.present_mode != mode {
                config.present_mode = mode;
                surface.configure(&self.device, config);
            }
        }
    }
}