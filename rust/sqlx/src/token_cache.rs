use crate::{token, Result};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

const TOKEN_DURATION_SECS: u64 = 900; // 15 minutes
const REFRESH_BEFORE_EXPIRY_SECS: u64 = 300; // 5 minutes (refresh at 10 min mark)

#[derive(Clone)]
struct CachedToken {
    token: String,
    expires_at: SystemTime,
}

#[derive(Clone)]
pub struct TokenCache {
    cache: Arc<RwLock<Option<CachedToken>>>,
    host: String,
    region: String,
    profile: Option<String>,
}

impl TokenCache {
    pub fn new(host: String, region: String, profile: Option<String>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            host,
            region,
            profile,
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                let now = SystemTime::now();
                let refresh_at =
                    cached.expires_at - Duration::from_secs(REFRESH_BEFORE_EXPIRY_SECS);

                if now < refresh_at {
                    return Ok(cached.token.clone());
                }
            }
        }

        let mut cache = self.cache.write().await;

        // Double-check after acquiring write lock
        if let Some(cached) = cache.as_ref() {
            let now = SystemTime::now();
            let refresh_at = cached.expires_at - Duration::from_secs(REFRESH_BEFORE_EXPIRY_SECS);

            if now < refresh_at {
                return Ok(cached.token.clone());
            }
        }

        let token = token::generate_token(&self.host, &self.region, self.profile.as_deref()).await?;
        let expires_at = SystemTime::now() + Duration::from_secs(TOKEN_DURATION_SECS);

        *cache = Some(CachedToken {
            token: token.clone(),
            expires_at,
        });

        Ok(token)
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}
