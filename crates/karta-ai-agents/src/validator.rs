//! Валидация маршрута человеком.
//!
//! Процесс:
//! 1. Строится нитка маршрута (см. [`crate::route_thread_builder`])
//! 2. Человек проверяет маршрут в GUI
//! 3. Человек может исправить точки (drag-and-drop, удаление, добавление)
//! 4. Исправления сохраняются в базу для улучшения AI-моделей
//!
//! TODO: интеграция с GUI (desktop app)
//! TODO: подсистема сохранения и применения human feedback

use karta_core::coords::LatLon;
use serde::{Deserialize, Serialize};

/// Результат валидации маршрута человеком.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Прошёл ли маршрут валидацию без изменений
    pub is_valid: bool,
    /// Внесённые человеком правки
    pub corrections_made: Vec<Correction>,
    /// Уверенность в корректности маршрута (0.0 to 1.0)
    pub confidence: f64,
    /// Комментарий валидатора
    pub comment: Option<String>,
}

impl ValidationResult {
    /// Создаёт результат успешной валидации без правок.
    pub fn valid(confidence: f64) -> Self {
        Self {
            is_valid: true,
            corrections_made: Vec::new(),
            confidence,
            comment: None,
        }
    }

    /// Создаёт результат валидации с правками.
    pub fn with_corrections(corrections: Vec<Correction>, confidence: f64) -> Self {
        Self {
            is_valid: true,
            corrections_made: corrections,
            confidence,
            comment: None,
        }
    }
}

/// Единичная правка маршрута, внесённая человеком.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    /// Индекс исправленной точки в исходной нитке маршрута
    pub original_index: usize,
    /// Новая (исправленная) точка
    pub corrected_point: LatLon,
    /// Причина правки (например, "точка попала в болото")
    pub reason: String,
    /// Время внесения правки
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Correction {
    /// Создаёт новую правку.
    pub fn new(original_index: usize, corrected_point: LatLon, reason: impl Into<String>) -> Self {
        Self {
            original_index,
            corrected_point,
            reason: reason.into(),
            timestamp: Some(chrono::Utc::now()),
        }
    }
}

/// Валидатор маршрута (человек).
pub struct RouteValidator {
    /// Идентификатор валидатора (например, username)
    validator_id: String,
    /// Список накопленных правок
    corrections: Vec<Correction>,
}

impl RouteValidator {
    /// Создаёт нового валидатора.
    pub fn new(validator_id: impl Into<String>) -> Self {
        Self {
            validator_id: validator_id.into(),
            corrections: Vec::new(),
        }
    }

    /// Возвращает идентификатор валидатора.
    pub fn validator_id(&self) -> &str {
        &self.validator_id
    }

    /// Возвращает все накопленные правки.
    pub fn corrections(&self) -> &[Correction] {
        &self.corrections
    }

    /// Добавляет правку точки.
    pub fn add_correction(&mut self, correction: Correction) {
        self.corrections.push(correction);
    }

    /// Применяет накопленные правки к маршруту (возвращает новый маршрут).
    pub fn apply_corrections(&self, route_points: &[LatLon]) -> Vec<LatLon> {
        let mut corrected = route_points.to_vec();
        for correction in &self.corrections {
            if correction.original_index < corrected.len() {
                corrected[correction.original_index] = correction.corrected_point;
            }
        }
        corrected
    }

    /// Финализирует валидацию, возвращая результат.
    pub fn validate(&mut self, _route_points: &[LatLon]) -> ValidationResult {
        let corrections = std::mem::take(&mut self.corrections);
        let confidence = if corrections.is_empty() { 1.0 } else { 0.7 };
        ValidationResult {
            is_valid: true,
            corrections_made: corrections,
            confidence,
            comment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moscow() -> LatLon {
        LatLon::new(55.7558, 37.6173)
    }

    fn spb() -> LatLon {
        LatLon::new(59.9343, 30.3351)
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid(0.95);
        assert!(result.is_valid);
        assert!(result.corrections_made.is_empty());
        assert!((result.confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_correction_construction() {
        let correction = Correction::new(
            0,
            LatLon::new(55.8000, 37.6200),
            "точка попала в реку",
        );
        assert_eq!(correction.original_index, 0);
        assert_eq!(correction.reason, "точка попала в реку");
        assert!(correction.timestamp.is_some());
    }

    #[test]
    fn test_validator_apply_corrections() {
        let mut validator = RouteValidator::new("test-user");
        validator.add_correction(Correction::new(
            0,
            LatLon::new(55.8000, 37.6200),
            "исправление точки 0",
        ));

        let original = vec![moscow(), spb()];
        let corrected = validator.apply_corrections(&original);
        assert_eq!(corrected.len(), 2);
        assert_eq!(corrected[0], LatLon::new(55.8000, 37.6200));
        assert_eq!(corrected[1], spb());
    }

    #[test]
    fn test_validator_validate() {
        let mut validator = RouteValidator::new("test-user");
        let result = validator.validate(&[moscow(), spb()]);
        assert!(result.is_valid);
        assert!(result.corrections_made.is_empty());
        assert!((result.confidence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_validator_validate_with_corrections() {
        let mut validator = RouteValidator::new("test-user");
        validator.add_correction(Correction::new(1, spb(), "исправление точки 1"));

        let result = validator.validate(&[moscow(), spb()]);
        assert!(result.is_valid);
        assert_eq!(result.corrections_made.len(), 1);
        assert!((result.confidence - 0.7).abs() < 1e-6);

        // Corrections should be consumed after validate
        assert!(validator.corrections().is_empty());
    }
}
