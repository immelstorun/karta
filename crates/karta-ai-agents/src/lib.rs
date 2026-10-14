//! # karta-ai-agents
//!
//! AI-агенты для построения ниток маршрутов из текстовых описаний.
//!
//! Пайплайн:
//! 1. [`toponym_extractor::ToponymExtractor`] — извлечение топонимов из текста
//! 2. Геокодирование топонимов → координаты
//! 3. [`route_thread_builder::RouteThreadBuilder`] — построение нитки маршрута
//! 4. [`validator::RouteValidator`] — валидация человеком
//!
//! Планируется:
//! - NER (Named Entity Recognition) для извлечения топонимов
//! - Геокодирование через онлайн-сервисы (Nominatim и др.)
//! - LLM-интеграция для улучшения качества извлечения

pub mod toponym_extractor;
pub mod route_thread_builder;
pub mod validator;

/// Конфигурация AI-агентов.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Включить использование LLM для улучшения точности
    pub use_llm: bool,
    /// Минимальная длина топонима для извлечения
    pub min_toponym_length: usize,
    /// Максимальное количество топонимов в одном тексте
    pub max_toponyms: usize,
    /// Включить геокодирование онлайн-сервисами
    pub use_online_geocoding: bool,
    /// Включить кэширование результатов геокодирования
    pub cache_geocoding_results: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            use_llm: false,
            min_toponym_length: 3,
            max_toponyms: 50,
            use_online_geocoding: false,
            cache_geocoding_results: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert!(!config.use_llm);
        assert_eq!(config.min_toponym_length, 3);
        assert_eq!(config.max_toponyms, 50);
    }
}
