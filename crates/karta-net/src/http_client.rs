//! HTTP-клиент с connection pooling, настройкой user-agent, timeout и proxy.

use std::time::Duration;

/// Ошибки HTTP-клиента.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Ошибка сетевого взаимодействия (reqwest).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Некорректный URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Конфигурация HTTP-клиента.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Значение заголовка User-Agent.
    pub user_agent: String,
    /// Таймаут запроса (seconds).
    pub timeout_secs: u64,
    /// Максимальное количество одновременных соединений в пуле.
    pub max_connections: u32,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("karta/{}", env!("CARGO_PKG_VERSION")),
            timeout_secs: 30,
            max_connections: 16,
        }
    }
}

/// Обёртка над [`reqwest::Client`] с connection pooling.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpClient {
    /// Создаёт новый HTTP-клиент с заданной конфигурацией.
    pub fn new(config: HttpClientConfig) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_connections as usize)
            .build()?;
        Ok(Self { client, config })
    }

    /// Выполняет GET-запрос и возвращает тело ответа в виде байтов.
    pub async fn get(&self, url: &str) -> Result<Vec<u8>, Error> {
        self.get_with_headers(url, &[]).await
    }

    /// Выполняет GET-запрос с дополнительными заголовками
    /// (например, Referer или API-токены) и возвращает тело ответа.
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<Vec<u8>, Error> {
        let mut request = self.client.get(url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = request.send().await?;
        let response = response.error_for_status()?;
        Ok(response.bytes().await?.to_vec())
    }

    /// Возвращает текущую конфигурацию клиента.
    pub fn config(&self) -> &HttpClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let config = HttpClientConfig {
            user_agent: "test-agent".to_string(),
            timeout_secs: 5,
            max_connections: 2,
        };
        let client = HttpClient::new(config).expect("failed to create http client");
        assert_eq!(client.config().user_agent, "test-agent");
        assert_eq!(client.config().timeout_secs, 5);
        assert_eq!(client.config().max_connections, 2);
    }

    #[test]
    fn test_default_config() {
        let config = HttpClientConfig::default();
        assert!(config.user_agent.starts_with("karta/"));
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_connections, 16);
    }
}
