//! Рендерер тайлов.
//!
//! TODO: wgpu pipeline:
//! - Создание GPU-устройства и очереди команд
//! - Загрузка текстур тайлов в GPU-память
//! - Шейдеры для отображения тайлов с учётом зума/пана
//! - LRU-выгрузка неиспользуемых текстур

use crate::RenderConfig;
use karta_core::tiles::TileId;

/// Рендерер тайлов — рисует тайлы на GPU через wgpu.
pub struct TileRenderer {
    /// Конфигурация рендеринга
    config: RenderConfig,
    /// Количество отрисованных тайлов (для статистики)
    rendered_count: u64,
}

impl TileRenderer {
    /// Создаёт новый рендерер тайлов с заданной конфигурацией.
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            rendered_count: 0,
        }
    }

    /// Рендерит тайл: загружает изображение как текстуру и добавляет в очередь отрисовки.
    ///
    /// TODO: реальная реализация с wgpu:
    /// - создание `wgpu::Texture` из пикселей
    /// - загрузка в GPU через `Queue::write_texture`
    /// - добавление в render pass
    pub fn render_tile(&mut self, tile: &TileId, image: &image::DynamicImage) {
        tracing::debug!(
            x = tile.x,
            y = tile.y,
            zoom = tile.zoom,
            width = image.width(),
            height = image.height(),
            "rendering tile"
        );
        self.rendered_count += 1;
    }

    /// Возвращает конфигурацию рендерера.
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Возвращает количество отрисованных тайлов.
    pub fn rendered_count(&self) -> u64 {
        self.rendered_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_renderer_creation() {
        let renderer = TileRenderer::new(RenderConfig::default());
        assert_eq!(renderer.rendered_count(), 0);
        assert!(renderer.config().vsync);
    }

    #[test]
    fn test_render_tile_increments_count() {
        let mut renderer = TileRenderer::new(RenderConfig::default());
        let tile = TileId { x: 1, y: 2, zoom: 3 };
        let img = image::DynamicImage::new_rgb8(256, 256);
        renderer.render_tile(&tile, &img);
        renderer.render_tile(&tile, &img);
        assert_eq!(renderer.rendered_count(), 2);
    }
}
