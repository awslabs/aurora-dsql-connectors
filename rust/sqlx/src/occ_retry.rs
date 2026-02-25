use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OCCRetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
}

impl Default for OCCRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            jitter_factor: 0.25,
        }
    }
}

pub fn is_occ_error(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            let code_str = code.as_ref();
            if code_str == "40001" || code_str == "OC000" || code_str == "OC001" {
                return true;
            }
        }
    }
    false
}

pub fn calculate_backoff(config: &OCCRetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay_ms as f64;
    let delay = (base * 2_f64.powi(attempt as i32)).min(config.max_delay_ms as f64);
    let jitter = delay * rand::random::<f64>() * config.jitter_factor;

    Duration::from_millis((delay + jitter) as u64)
}
