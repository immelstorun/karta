//! Клиент tlib.ru — библиотеки туристских отчётов.
//!
//! TODO: реализация HTTP-клиента для tlib.ru:
//! - Поиск отчётов по региону/тегам
//! - Скачивание и парсинг полных отчётов
//! - Извлечение ниток маршрутов и точек

use serde::{Deserialize, Serialize};

/// Базовый URL tlib.ru.
pub const TLIB_BASE_URL: &str = "https://tlib.ru";

/// Отчёт tlib.ru.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlibReport {
    /// Уникальный идентификатор отчёта
    pub id: String,
    /// Заголовок отчёта
    pub title: String,
    /// URL отчёта
    pub url: String,
    /// Регион похода
    pub region: Option<String>,
    /// Год похода
    pub year: Option<u32>,
    /// Текстовое описание маршрута
    pub route_description: Option<String>,
}

/// Клиент tlib.ru.
pub struct TlibClient {
    /// Базовый URL tlib.ru
    base_url: String,
}

impl TlibClient {
    /// Создаёт новый клиент tlib.ru.
    pub fn new() -> Self {
        Self {
            base_url: TLIB_BASE_URL.to_string(),
        }
    }

    /// Создаёт клиент с кастомным базовым URL (для тестов).
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }

    /// Возвращает базовый URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Поиск отчётов по запросу.
    ///
    /// TODO: реализация через tlib.ru HTML API.
    pub async fn search_reports(&self, _query: &str) -> Result<Vec<TlibReport>, Error> {
        Err(Error::NotImplemented)
    }

    /// Получение полного отчёта по ID.
    ///
    /// TODO: реализация через tlib.ru HTML API.
    pub async fn get_report(&self, _report_id: &str) -> Result<TlibReport, Error> {
        Err(Error::NotImplemented)
    }
}

impl Default for TlibClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Ошибки tlib-клиента.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Метод ещё не реализован
    #[error("Not implemented")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_model::{Difficulty, Route, RouteSource};
    use chrono::Utc;

    #[tokio::test]
    async fn test_search_reports_not_implemented() {
        let client = TlibClient::new();
        assert!(matches!(client.search_reports("Алтай").await, Err(Error::NotImplemented)));
    }

    #[tokio::test]
    async fn test_get_report_not_implemented() {
        let client = TlibClient::new();
        assert!(matches!(client.get_report("123").await, Err(Error::NotImplemented)));
    }

    #[test]
    fn test_tlib_client_base_url() {
        let client = TlibClient::new();
        assert_eq!(client.base_url(), "https://tlib.ru");

        let custom = TlibClient::with_base_url("https://test.tlib.ru");
        assert_eq!(custom.base_url(), "https://test.tlib.ru");
    }

    #[test]
    fn test_report_to_route_conversion() {
        // Verify that we can create a Route from a TlibReport (full flow test)
        let report = TlibReport {
            id: "123".to_string(),
            title: "Поход по Алтаю".to_string(),
            url: "https://tlib.ru/report/123".to_string(),
            region: Some("Алтай".to_string()),
            year: Some(2024),
            route_description: Some("Маршрут от посёлка ...".to_string()),
        };

        let route = Route {
            id: report.id.clone(),
            name: report.title.clone(),
            description: report.route_description.clone(),
            points: vec![],
            source: RouteSource::Tlib,
            difficulty: Some(Difficulty::Moderate),
            created_at: Utc::now(),
            author: None,
            tags: vec!["tlib".to_string()],
        };
        assert_eq!(route.source, RouteSource::Tlib);
    }
}
