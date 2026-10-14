//! Композитор слоёв.
//!
//! TODO: композиция base map + overlays:
//! - Смешивание слоёв с учётом opacity
//! - Управление порядком отрисовки (z-order)
//! - Обработка видимости слоёв

/// Описание слоя для композиции
#[derive(Debug, Clone)]
pub struct Layer {
    /// Имя слоя
    pub name: String,
    /// Прозрачность слоя (0.0 — полностью прозрачно, 1.0 — непрозрачно)
    pub opacity: f32,
    /// Видимость слоя
    pub visible: bool,
}

/// Композитор слоёв — объединяет базовую карту, overlays и векторные данные.
pub struct LayerCompositor {
    /// Список слоёв для композиции
    layers: Vec<Layer>,
}

impl LayerCompositor {
    /// Создаёт новый композитор слоёв.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Добавляет слой в композицию.
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Возвращает список слоёв.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Возвращает количество видимых слоёв.
    pub fn visible_layer_count(&self) -> usize {
        self.layers.iter().filter(|l| l.visible).count()
    }
}

impl Default for LayerCompositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_compositor() {
        let mut compositor = LayerCompositor::new();
        compositor.add_layer(Layer {
            name: "base".to_string(),
            opacity: 1.0,
            visible: true,
        });
        compositor.add_layer(Layer {
            name: "overlay".to_string(),
            opacity: 0.5,
            visible: false,
        });
        assert_eq!(compositor.layers().len(), 2);
        assert_eq!(compositor.visible_layer_count(), 1);
    }
}
