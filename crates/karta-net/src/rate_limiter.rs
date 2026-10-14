//! Token bucket rate limiter для ограничения частоты запросов.

use std::time::Duration;
use tokio::sync::Mutex;

/// Конфигурация rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Разрешённое количество запросов в секунду (sustained rate).
    pub requests_per_second: f64,
    /// Максимальный размер всплеска (burst) в токенах.
    pub burst_size: u32,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 20,
        }
    }
}

/// Token bucket rate limiter.
///
/// Ведро пополняется со скоростью `requests_per_second` токенов в секунду
/// и вмещает не более `burst_size` токенов. [`RateLimiter::acquire`]
/// ожидает появления хотя бы одного токена.
pub struct RateLimiter {
    tokens: Mutex<f64>,
    config: RateLimiterConfig,
    last_refill: Mutex<std::time::Instant>,
}

impl RateLimiter {
    /// Создаёт новый rate limiter с заданной конфигурацией.
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            tokens: Mutex::new(config.burst_size as f64),
            config,
            last_refill: Mutex::new(std::time::Instant::now()),
        }
    }

    /// Ожидает разрешения на выполнение одного запроса.
    ///
    /// Если в ведре есть токены, метод возвращает управление немедленно,
    /// иначе — ждёт пополнения ведра.
    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut last_refill = self.last_refill.lock().await;
                let mut tokens = self.tokens.lock().await;

                let elapsed = last_refill.elapsed().as_secs_f64();
                *last_refill = std::time::Instant::now();
                *tokens = (*tokens + elapsed * self.config.requests_per_second)
                    .min(self.config.burst_size as f64);

                if *tokens >= 1.0 {
                    *tokens -= 1.0;
                    return;
                }

                // Time until a single token is refilled.
                Duration::from_secs_f64((1.0 - *tokens) / self.config.requests_per_second)
            };
            tokio::time::sleep(wait).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_from_full_bucket() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            requests_per_second: 100.0,
            burst_size: 5,
        });
        // Draining a full bucket should not block noticeably.
        for _ in 0..5 {
            limiter.acquire().await;
        }
    }

    #[tokio::test]
    async fn test_rate_limiting_delay() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            requests_per_second: 50.0,
            burst_size: 1,
        });
        limiter.acquire().await; // consume the only token

        let start = std::time::Instant::now();
        limiter.acquire().await; // should wait ~20ms for refill
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(15),
            "second acquire returned too fast: {elapsed:?}"
        );
    }
}
