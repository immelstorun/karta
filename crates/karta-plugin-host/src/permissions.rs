//! Система разрешений (permissions) для плагинов.
//!
//! Каждый плагин получает набор разрешений, определяющих:
//! - Доступ к сети (список разрешённых URL patterns)
//! - Доступ к файловой системе (список разрешённых путей)
//! - Доступ к UI (список разрешённых UI-элементов)
//! - Доступ к GPS

use serde::{Deserialize, Serialize};

/// Разрешения плагина.
///
/// Все поля — белые списки (allowlists):
/// - Пустой список = полный запрет
/// - `["*"]` = полный доступ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    /// Разрешённые сетевые ресурсы (glob patterns, например `"api.openstreetmap.org/*"`)
    pub network: Vec<String>,
    /// Разрешённые пути файловой системы (glob patterns)
    pub filesystem: Vec<String>,
    /// Разрешённые UI-элементы (например `"toolbar"`, `"map-overlay"`)
    pub ui: Vec<String>,
    /// Доступ к GPS-данным
    pub gps: bool,
}

impl Permissions {
    /// Создаёт набор разрешений с полным доступом (для доверенных плагинов).
    pub fn full() -> Self {
        Self {
            network: vec!["*".to_string()],
            filesystem: vec!["*".to_string()],
            ui: vec!["*".to_string()],
            gps: true,
        }
    }

    /// Создаёт набор разрешений с минимальным доступом (никаких разрешений).
    pub fn none() -> Self {
        Self::default()
    }
}

/// Проверщик разрешений — решает, может ли плагин выполнить действие.
pub struct PermissionChecker {
    permissions: Permissions,
}

impl PermissionChecker {
    /// Создаёт новый проверщик разрешений.
    pub fn new(permissions: Permissions) -> Self {
        Self { permissions }
    }

    /// Проверяет, может ли плагин получить доступ к указанному URL.
    ///
    /// URL должен соответствовать хотя бы одному паттерну из
    /// `permissions.network`.
    pub fn can_access_url(&self, url: &str) -> bool {
        if url.contains("..") {
            return false;
        }
        self.permissions
            .network
            .iter()
            .any(|pattern| matches_pattern(pattern, url))
    }

    /// Проверяет, может ли плагин получить доступ к указанному пути.
    ///
    /// Путь должен соответствовать хотя бы одному паттерну из
    /// `permissions.filesystem`.
    pub fn can_access_path(&self, path: &str) -> bool {
        if path.contains("..") {
            return false;
        }
        self.permissions
            .filesystem
            .iter()
            .any(|pattern| matches_pattern(pattern, path))
    }

    /// Проверяет, может ли плагин использовать указанный UI-элемент.
    pub fn can_access_ui(&self, ui_element: &str) -> bool {
        self.permissions
            .ui
            .iter()
            .any(|pattern| matches_pattern(pattern, ui_element))
    }

    /// Проверяет, есть ли у плагина доступ к GPS.
    pub fn has_gps(&self) -> bool {
        self.permissions.gps
    }
}

/// Проверяет, соответствует ли строка glob-паттерну.
///
/// Поддерживаемые спецсимволы:
/// - `*` — любая последовательность символов (включая пустую)
/// - `?` — ровно один символ
pub fn matches_pattern(pattern: &str, s: &str) -> bool {
    fn helper(p: &[u8], s: &[u8]) -> bool {
        match (p.first(), s.first()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(b'*'), _) => helper(&p[1..], s) || (!s.is_empty() && helper(p, &s[1..])),
            (Some(b'?'), Some(_)) => helper(&p[1..], &s[1..]),
            (Some(pc), Some(sc)) if pc == sc => helper(&p[1..], &s[1..]),
            _ => false,
        }
    }
    helper(pattern.as_bytes(), s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("*.com/*", "https://example.com/path"));
        assert!(matches_pattern("/tmp/*", "/tmp/file.txt"));
        assert!(matches_pattern("a?c", "abc"));
        assert!(!matches_pattern("a?c", "abbc"));
        assert!(!matches_pattern("/tmp/*", "/home/user/file"));
    }

    #[test]
    fn test_can_access_url() {
        let mut perms = Permissions::none();
        perms.network = vec!["api.openstreetmap.org/*".to_string()];
        let checker = PermissionChecker::new(perms);

        assert!(checker.can_access_url("api.openstreetmap.org/0/0/0.png"));
        assert!(!checker.can_access_url("evil.com/exfiltrate"));
    }

    #[test]
    fn test_can_access_path() {
        let mut perms = Permissions::none();
        perms.filesystem = vec!["/home/user/maps/*".to_string()];
        let checker = PermissionChecker::new(perms);

        assert!(checker.can_access_path("/home/user/maps/file.gpx"));
        assert!(!checker.can_access_path("/etc/passwd"));
        assert!(!checker.can_access_path("/home/user/maps/../../etc/passwd"));
    }

    #[test]
    fn test_can_access_ui() {
        let mut perms = Permissions::none();
        perms.ui = vec!["toolbar".to_string(), "map-overlay".to_string()];
        let checker = PermissionChecker::new(perms);

        assert!(checker.can_access_ui("toolbar"));
        assert!(checker.can_access_ui("map-overlay"));
        assert!(!checker.can_access_ui("settings-panel"));
    }

    #[test]
    fn test_full_permissions() {
        let checker = PermissionChecker::new(Permissions::full());
        assert!(checker.can_access_url("https://example.com"));
        assert!(checker.can_access_path("/any/path"));
        assert!(checker.can_access_ui("any-ui"));
        assert!(checker.has_gps());
    }

    #[test]
    fn test_gps_permission() {
        let checker = PermissionChecker::new(Permissions::none());
        assert!(!checker.has_gps());
    }
}
