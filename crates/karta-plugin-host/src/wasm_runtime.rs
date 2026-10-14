//! WASM runtime для плагинов karta.
//!
//! TODO: host functions для взаимодействия с karta:
//! - `karta_log` — логирование из плагина
//! - `karta_http_fetch` — HTTP-запросы (с проверкой permissions)
//! - `karta_read_file` / `karta_write_file` — доступ к ФС (с проверкой permissions)
//! - `karta_add_layer` — добавление слоя на карту
//! - `kara_get_gps` — доступ к GPS-данным (с проверкой permissions)
//!
//! TODO: WASI preview2 support, plugin lifecycle management.

use crate::permissions::{PermissionChecker, Permissions};

/// WASM runtime — обёртка над wasmtime.
///
/// Каждый плагин выполняется в изолированном WASM-модуле
/// с ограниченным доступом к ресурсам хоста.
pub struct WasmRuntime {
    engine: wasmtime::Engine,
}

impl WasmRuntime {
    /// Создаёт новый WASM runtime.
    pub fn new() -> Result<Self, Error> {
        let engine = wasmtime::Engine::default();
        Ok(Self { engine })
    }

    /// Загружает WASM-плагин из байтов.
    ///
    /// TODO: инициализация host functions, вызов `_start` или
    /// экспортированной функции `init`.
    pub fn load_plugin(&mut self, wasm_bytes: &[u8]) -> Result<PluginInstance, Error> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        Ok(PluginInstance {
            module,
            permissions: Permissions::none(),
        })
    }

    /// Возвращает engine wasmtime.
    pub fn engine(&self) -> &wasmtime::Engine {
        &self.engine
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("failed to create wasmtime engine")
    }
}

/// Загруженный WASM-плагин.
pub struct PluginInstance {
    module: wasmtime::Module,
    permissions: Permissions,
}

impl PluginInstance {
    /// Возвращает модуль wasmtime.
    pub fn module(&self) -> &wasmtime::Module {
        &self.module
    }

    /// Возвращает permissions плагина.
    pub fn permissions(&self) -> &Permissions {
        &self.permissions
    }

    /// Возвращает checker permissions для этого плагина.
    pub fn permission_checker(&self) -> PermissionChecker {
        PermissionChecker::new(self.permissions.clone())
    }
}

/// Ошибки WASM runtime.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Ошибка компиляции WASM-модуля
    #[error("WASM compile error: {0}")]
    Compile(#[from] wasmtime::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_load_invalid_plugin() {
        let mut runtime = WasmRuntime::new().expect("runtime should be created");
        let result = runtime.load_plugin(b"not a wasm module");
        assert!(result.is_err());
    }
}
