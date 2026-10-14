//! # karta-render
//!
//! Рендеринг карты. Планируется:
//! - wgpu-based рендеринг тайлов (растровые и векторные)
//! - GPU-ускорение для плавного зума/пана
//! - Композиция слоёв (base map + overlays + vector data)

pub mod layer_compositor;
pub mod tile_renderer;

/// Конфигурация рендерера
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Включить вертикальную синхронизацию
    pub vsync: bool,
    /// Максимальное количество тайлов в GPU-кэше
    pub gpu_tile_cache_size: usize,
    /// Фоновый цвет (тёмная тема)
    pub background_color: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            gpu_tile_cache_size: 512,
            background_color: [0.1, 0.1, 0.1, 1.0],
        }
    }
}
