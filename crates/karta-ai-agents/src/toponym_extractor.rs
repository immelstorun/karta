//! Извлечение топонимов из текста.
//!
//! Топонимы — географические названия: горы, перевалы, реки, озёра, посёлки и т.д.
//!
//! Текущая реализация — заглушка, возвращающая пустой список.
//!
//! TODO: NLP-подход или LLM-интеграция:
//! - NER-модели для русского языка
//! - Контекстное определение типа топонима
//! - Разрешение неоднозначностей (например, "Кубань" — река или регион)
//! - Словарь известных топонимов для региона

use serde::{Deserialize, Serialize};

/// Извлечённый топоним с указанием позиции в тексте.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedToponym {
    /// Сам топоним (название географического объекта)
    pub name: String,
    /// Тип топонима (гора, река, перевал и т.д.)
    pub kind: ToponymKind,
    /// Позиция в тексте (в символах)
    pub position: usize,
    /// Контекст (окружающие слова) для устранения неоднозначности
    pub context: Option<String>,
}

impl ExtractedToponym {
    /// Создаёт новый извлечённый топоним.
    pub fn new(name: impl Into<String>, kind: ToponymKind, position: usize) -> Self {
        Self {
            name: name.into(),
            kind,
            position,
            context: None,
        }
    }

    /// Задаёт контекст топонима и возвращает self.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Тип географического объекта (топонима).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToponymKind {
    /// Гора, вершина, пик
    Peak,
    /// Перевал
    Pass,
    /// Река, ручей
    River,
    /// Озеро
    Lake,
    /// Город, посёлок, деревня
    Settlement,
    /// Хребет, горная система
    Ridge,
    /// Ледник
    Glacier,
    /// Базовый лагерь, приют, изба
    Camp,
    /// Долина, ущелье
    Valley,
    /// Прочее
    Other,
}

impl ToponymKind {
    /// Возвращает русское название типа топонима.
    pub fn display_name(&self) -> &'static str {
        match self {
            ToponymKind::Peak => "вершина",
            ToponymKind::Pass => "перевал",
            ToponymKind::River => "река",
            ToponymKind::Lake => "озеро",
            ToponymKind::Settlement => "населённый пункт",
            ToponymKind::Ridge => "хребет",
            ToponymKind::Glacier => "ледник",
            ToponymKind::Camp => "приют/базовый лагерь",
            ToponymKind::Valley => "долина",
            ToponymKind::Other => "прочее",
        }
    }
}

/// Извлекатель топонимов из текста.
pub struct ToponymExtractor {
    /// Конфигурация агента
    config: crate::AgentConfig,
}

impl ToponymExtractor {
    /// Создаёт новый извлекатель топонимов с конфигурацией по умолчанию.
    pub fn new() -> Self {
        Self {
            config: crate::AgentConfig::default(),
        }
    }

    /// Создаёт извлекатель с кастомной конфигурацией.
    pub fn with_config(config: crate::AgentConfig) -> Self {
        Self { config }
    }

    /// Извлекает топонимы из текста.
    ///
    /// TODO: реализация (NER или LLM).
    /// Пока возвращает пустой список.
    pub fn extract(&self, _text: &str) -> Vec<ExtractedToponym> {
        // TODO: NER-based extraction
        Vec::new()
    }

    /// Возвращает конфигурацию.
    pub fn config(&self) -> &crate::AgentConfig {
        &self.config
    }
}

impl Default for ToponymExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TEXT: &str = r#"
        Маршрут начинался от посёлка Тюнгур, далее вверх по реке Катунь
        до устья реки Актру. Оттуда подъём на перевал Кузуюм,
        затем траверс хребта Белуха. Финальная точка — озеро Мультинское.
    "#;

    #[test]
    fn test_extract_returns_empty_for_stub() {
        let extractor = ToponymExtractor::new();
        let toponyms = extractor.extract(SAMPLE_TEXT);
        assert!(toponyms.is_empty());
    }

    #[test]
    fn test_toponym_kind_display_names() {
        assert_eq!(ToponymKind::Peak.display_name(), "вершина");
        assert_eq!(ToponymKind::Pass.display_name(), "перевал");
        assert_eq!(ToponymKind::River.display_name(), "река");
        assert_eq!(ToponymKind::Lake.display_name(), "озеро");
        assert_eq!(ToponymKind::Settlement.display_name(), "населённый пункт");
    }

    #[test]
    fn test_extracted_toponym_construction() {
        let toponym = ExtractedToponym::new("Белуха", ToponymKind::Peak, 42)
            .with_context("траверс хребта Белуха");
        assert_eq!(toponym.name, "Белуха");
        assert_eq!(toponym.kind, ToponymKind::Peak);
        assert_eq!(toponym.position, 42);
        assert!(toponym.context.is_some());
    }

    #[test]
    fn test_extractor_with_custom_config() {
        let mut config = crate::AgentConfig::default();
        config.min_toponym_length = 5;
        let extractor = ToponymExtractor::with_config(config);
        assert_eq!(extractor.config().min_toponym_length, 5);
    }
}
