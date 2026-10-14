//! # karta-route-db
//!
//! База туристских маршрутов. Интеграция с tlib.ru (библиотека отчётов о походах).
//! Планируется:
//! - Парсинг отчётов tlib.ru
//! - Хранение маршрутов в SQLite
//! - AI-построение ниток маршрутов из текстовых описаний
//! - Community-база маршрутов с рейтингами

pub mod route_model;
pub mod tlib_client;

pub use route_model::{Difficulty, Route, RouteSource};
pub use tlib_client::{TlibClient, TlibReport};

use std::path::PathBuf;

/// Точка входа для работы с базой маршрутов.
pub struct RouteDb {
    /// SQLite database file path
    db_path: PathBuf,
}

impl RouteDb {
    /// Создаёт новую базу маршрутов по указанному пути.
    ///
    /// TODO: инициализация SQLite схемы (таблицы routes, waypoints, ratings).
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }

    /// Возвращает путь к файлу базы данных.
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_db_creation() {
        let db = RouteDb::new("/tmp/test-routes.sqlite");
        assert_eq!(
            db.db_path(),
            std::path::Path::new("/tmp/test-routes.sqlite")
        );
    }
}
