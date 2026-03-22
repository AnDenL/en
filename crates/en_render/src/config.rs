use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FilterMode { Nearest, Linear }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PowerPreference { LowPower, HighPerformance }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderSettings {
    pub clear_color: [f32; 4],
    pub vsync: bool,
    /// Згладжування (для піксель-арту зазвичай 1, для 3D/векторів - 4 або 8)
    pub msaa_samples: u32, 
    /// Дискретна чи інтегрована відеокарта
    pub power_preference: PowerPreference,
    /// Фільтрація за замовчуванням (Nearest ідеально для твоїх ігор)
    pub default_filter: FilterMode,
    /// Якщо true, рендер буде зберігати пропорції пікселів при розтягуванні вікна
    pub pixel_perfect: bool,
    /// Розмір внутрішнього буфера (корисно для ретро-ігор, щоб рендерити в меншій роздільній здатності, а потім скейлити)
    pub internal_resolution: Option<[u32; 2]>,
    /// Максимальна кількість екземплярів в одному батчі (для оптимізації пам'яті)
    pub max_batch_size: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            clear_color: [0.1, 0.1, 0.15, 1.0],
            vsync: true,
            msaa_samples: 1,
            power_preference: PowerPreference::HighPerformance,
            default_filter: FilterMode::Nearest,
            pixel_perfect: true,
            internal_resolution: None,
            max_batch_size: 10_000,
        }
    }
}