use crate::types::{InstanceData, RenderBatch};

pub struct SpriteBatcher<'a> {
    instances: Vec<InstanceData>,
    batches: Vec<RenderBatch<'a>>,
    current_bind_group: Option<&'a wgpu::BindGroup>,
    current_batch_start: u32,
}

impl<'a> SpriteBatcher<'a> {
    pub fn new() -> Self {
        Self {
            // Резервуємо пам'ять одразу, щоб уникнути частих реаллокацій під час гри
            instances: Vec::with_capacity(2048),
            batches: Vec::with_capacity(128),
            current_bind_group: None,
            current_batch_start: 0,
        }
    }

    /// Очищує батчер перед початком нового кадру
    pub fn begin(&mut self) {
        self.instances.clear();
        self.batches.clear();
        self.current_bind_group = None;
        self.current_batch_start = 0;
    }

    /// Додає один спрайт у батчер.
    #[inline] // Важлива оптимізація, бо ця функція викликатиметься тисячі разів за кадр
    pub fn push(&mut self, bind_group: &'a wgpu::BindGroup, instance: InstanceData) {
        let start_new_batch = match self.current_bind_group {
            Some(current_bg) => !std::ptr::eq(current_bg, bind_group),
            None => true,
        };

        if start_new_batch {
            // Зберігаємо попередній батч, якщо він був
            if let Some(bg) = self.current_bind_group {
                let end = self.instances.len() as u32;
                if self.current_batch_start < end {
                    self.batches.push(RenderBatch {
                        bind_group: bg,
                        range: self.current_batch_start..end,
                    });
                }
            }
            // Починаємо новий
            self.current_bind_group = Some(bind_group);
            self.current_batch_start = self.instances.len() as u32;
        }

        self.instances.push(instance);
    }

    /// Закриває останній батч (викликається автоматично в Renderer)
    pub fn finish(&mut self) {
        if let Some(bg) = self.current_bind_group {
            let end = self.instances.len() as u32;
            if self.current_batch_start < end {
                self.batches.push(RenderBatch {
                    bind_group: bg,
                    range: self.current_batch_start..end,
                });
            }
            self.current_bind_group = None;
        }
    }

    // Внутрішні методи для рендерера
    pub(crate) fn instances(&self) -> &[InstanceData] {
        &self.instances
    }
    pub(crate) fn batches(&self) -> &[RenderBatch<'a>] {
        &self.batches
    }
}
