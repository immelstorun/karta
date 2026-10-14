//! # karta-plugin-host
//!
//! Плагинная система karta:
//! - WASM-плагины (песочница, кроссплатформенность)
//! - Декларативные слои (TOML)
//! - Система permissions (network, filesystem, ui, gps)

pub mod declarative_layers;
pub mod permissions;
pub mod wasm_runtime;

/// Plugin API version (for semantic versioning of plugin interface).
///
/// Плагины compiled against API version X can run in host with
/// API version Y if X <= Y (backward compatibility).
pub const PLUGIN_API_VERSION: u32 = 1;

/// Идентификатор плагина (уникальное имя в реестре).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(String);

impl PluginId {
    /// Создаёт новый идентификатор плагина.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Возвращает имя плагина.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_api_version() {
        assert_eq!(PLUGIN_API_VERSION, 1);
    }

    #[test]
    fn test_plugin_id() {
        let id = PluginId::new("my-plugin");
        assert_eq!(id.name(), "my-plugin");
        assert_eq!(id.to_string(), "my-plugin");
        assert_eq!(id, PluginId::new("my-plugin"));
        assert_ne!(id, PluginId::new("other-plugin"));
    }
}
