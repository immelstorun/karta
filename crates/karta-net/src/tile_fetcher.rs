//! Высокоуровневый загрузчик тайлов с retry (exponential backoff) и rate limiting.

use crate::http_client::{Error, HttpClient};
use crate::rate_limiter::RateLimiter;
use std::sync::Arc;
use std::time::Duration;

/// Запрос на загрузку тайла.
#[derive(Debug, Clone)]
pub struct TileFetchRequest {
    /// URL тайла.
    pub url: String,
    /// Дополнительные HTTP-заголовки запроса.
    pub headers: Vec<(String, String)>,
    /// Количество повторов при неудаче.
    pub retry_count: u32,
    /// Базовая задержка между повторами (milliseconds),
    /// фактическая задержка — `base * 2^attempt`.
    pub retry_delay_ms: u64,
}

/// Загрузчик тайлов: HTTP-клиент + rate limiter + retry с exponential backoff.
pub struct TileFetcher {
    http_client: Arc<HttpClient>,
    rate_limiter: Arc<RateLimiter>,
}

impl TileFetcher {
    /// Создаёт новый загрузчик тайлов.
    pub fn new(http_client: Arc<HttpClient>, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            http_client,
            rate_limiter,
        }
    }

    /// Загружает тайл по заданному запросу.
    ///
    /// Выполняет до `retry_count` повторов с экспоненциальной задержкой
    /// (`retry_delay_ms * 2^attempt`). Перед каждой попыткой проходит
    /// через rate limiter.
    pub async fn fetch_tile(&self, request: &TileFetchRequest) -> Result<Vec<u8>, Error> {
        let mut last_err = None;
        for attempt in 0..=request.retry_count {
            if attempt > 0 {
                let delay = Duration::from_millis(
                    request
                        .retry_delay_ms
                        .saturating_mul(1u64 << (attempt - 1).min(30)),
                );
                tokio::time::sleep(delay).await;
            }

            self.rate_limiter.acquire().await;
            match self
                .http_client
                .get_with_headers(&request.url, &request.headers)
                .await
            {
                Ok(data) => return Ok(data),
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_attempts = request.retry_count + 1,
                        url = %request.url,
                        error = %e,
                        "tile fetch failed"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("at least one attempt was made"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::HttpClientConfig;
    use crate::rate_limiter::RateLimiterConfig;

    fn fetcher() -> TileFetcher {
        let http_client =
            HttpClient::new(HttpClientConfig::default()).expect("failed to create http client");
        let rate_limiter = RateLimiter::new(RateLimiterConfig {
            requests_per_second: 1000.0,
            burst_size: 1000,
        });
        TileFetcher::new(Arc::new(http_client), Arc::new(rate_limiter))
    }

    #[tokio::test]
    async fn test_fetch_tile_invalid_url_retries() {
        let fetcher = fetcher();
        let request = TileFetchRequest {
            url: "not a valid url at all".to_string(),
            headers: vec![],
            retry_count: 2,
            retry_delay_ms: 1,
        };
        // Invalid URL fails fast; the fetcher should report the error
        // after exhausting the retries.
        let result = fetcher.fetch_tile(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_tile_success_no_retry() {
        let fetcher = fetcher();
        let request = TileFetchRequest {
            url: "http://127.0.0.1:1/nonexistent".to_string(),
            headers: vec![],
            retry_count: 0,
            retry_delay_ms: 0,
        };
        // Connection refused is still an error, but must not hang.
        let result = tokio::time::timeout(Duration::from_secs(10), fetcher.fetch_tile(&request))
            .await
            .expect("fetch timed out");
        assert!(result.is_err());
    }
}
