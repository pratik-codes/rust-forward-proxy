//! Certificate caching system with Redis and in-memory backends

use crate::tls::CertificateData;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn, error};

#[cfg(feature = "redis-support")]
use redis::{Commands, Connection};

/// Certificate cache entry with expiration
#[derive(Debug, Clone)]
struct CachedCertificate {
    cert_data: CertificateData,
    created_at: u64,
    expires_at: u64,
}

/// Certificate cache backend trait
pub trait CertificateCache: Send + Sync {
    fn get(&self, domain: &str) -> Result<Option<CertificateData>>;
    fn set(&self, domain: &str, cert_data: CertificateData, ttl_seconds: u64) -> Result<()>;
    fn remove(&self, domain: &str) -> Result<()>;
    fn clear(&self) -> Result<()>;
    fn cache_info(&self) -> String;
}

/// In-memory certificate cache implementation
pub struct MemoryCache {
    cache: Arc<Mutex<HashMap<String, CachedCertificate>>>,
    max_entries: usize,
}

impl MemoryCache {
    pub fn new(max_entries: usize) -> Self {
        debug!("Creating in-memory certificate cache (max_entries: {})", max_entries);
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }

    fn cleanup_expired(&self) {
        let now = Self::current_timestamp();
        let mut cache = self.cache.lock().unwrap();
        let initial_count = cache.len();
        
        cache.retain(|domain, entry| {
            let is_valid = now < entry.expires_at;
            if !is_valid {
                debug!("Removing expired certificate for domain: {}", domain);
            }
            is_valid
        });
        
        let removed = initial_count - cache.len();
        if removed > 0 {
            debug!("Cleaned up {} expired certificates", removed);
        }
    }

    fn enforce_size_limit(&self) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() > self.max_entries {
            // Remove oldest entries (simple LRU approximation)
            let mut entries: Vec<(String, u64)> = cache
                .iter()
                .map(|(domain, entry)| (domain.clone(), entry.created_at))
                .collect();
            
            entries.sort_by_key(|(_, created_at)| *created_at);
            
            let to_remove = cache.len() - self.max_entries;
            for (domain, _) in entries.iter().take(to_remove) {
                cache.remove(domain);
                debug!("Removed old certificate for domain: {}", domain);
            }
            
            debug!("Enforced cache size limit: removed {} entries", to_remove);
        }
    }
}

impl CertificateCache for MemoryCache {
    fn get(&self, domain: &str) -> Result<Option<CertificateData>> {
        self.cleanup_expired();
        
        let cache = self.cache.lock().unwrap();
        match cache.get(domain) {
            Some(entry) => {
                let now = Self::current_timestamp();
                if now < entry.expires_at {
                    debug!("Cache HIT for domain: {} (expires in {} seconds)", domain, entry.expires_at - now);
                    Ok(Some(entry.cert_data.clone()))
                } else {
                    debug!("Cache entry expired for domain: {}", domain);
                    Ok(None)
                }
            }
            None => {
                debug!("Cache MISS for domain: {}", domain);
                Ok(None)
            }
        }
    }

    fn set(&self, domain: &str, cert_data: CertificateData, ttl_seconds: u64) -> Result<()> {
        let now = Self::current_timestamp();
        let expires_at = now + ttl_seconds;
        
        let entry = CachedCertificate {
            cert_data,
            created_at: now,
            expires_at,
        };
        
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(domain.to_string(), entry);
        }
        
        self.enforce_size_limit();
        
        debug!("Cached certificate for domain: {} (TTL: {} seconds)", domain, ttl_seconds);
        Ok(())
    }

    fn remove(&self, domain: &str) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        if cache.remove(domain).is_some() {
            debug!("Removed certificate from cache for domain: {}", domain);
        }
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let count = cache.len();
        cache.clear();
        info!("Cleared certificate cache ({} entries removed)", count);
        Ok(())
    }

    fn cache_info(&self) -> String {
        let cache = self.cache.lock().unwrap();
        format!("Memory cache: {}/{} entries", cache.len(), self.max_entries)
    }
}

/// Redis certificate cache implementation
#[cfg(feature = "redis-support")]
pub struct RedisCache {
    connection: Arc<Mutex<Connection>>,
    key_prefix: String,
}

#[cfg(feature = "redis-support")]
impl RedisCache {
    pub fn new(redis_url: &str, key_prefix: Option<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
            
        let connection = client.get_connection()
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;
            
        let key_prefix = key_prefix.unwrap_or_else(|| "proxy:cert:".to_string());
        
        info!("Connected to Redis certificate cache (prefix: {})", key_prefix);
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            key_prefix,
        })
    }

    fn make_key(&self, domain: &str) -> String {
        format!("{}{}", self.key_prefix, domain)
    }
}

#[cfg(feature = "redis-support")]
impl CertificateCache for RedisCache {
    fn get(&self, domain: &str) -> Result<Option<CertificateData>> {
        let key = self.make_key(domain);
        let mut conn = self.connection.lock().unwrap();
        
        match conn.get::<_, Option<Vec<u8>>>(&key) {
            Ok(Some(data)) => {
                match bincode::deserialize::<CertificateData>(&data) {
                    Ok(cert_data) => {
                        debug!("Redis cache HIT for domain: {}", domain);
                        Ok(Some(cert_data))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached certificate for {}: {}", domain, e);
                        // Remove corrupted data
                        let _ = conn.del::<_, ()>(&key);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Redis cache MISS for domain: {}", domain);
                Ok(None)
            }
            Err(e) => {
                error!("Redis get error for domain {}: {}", domain, e);
                Ok(None) // Graceful degradation - don't fail the request
            }
        }
    }

    fn set(&self, domain: &str, cert_data: CertificateData, ttl_seconds: u64) -> Result<()> {
        let key = self.make_key(domain);
        let data = bincode::serialize(&cert_data)
            .map_err(|e| anyhow!("Failed to serialize certificate data: {}", e))?;
            
        let mut conn = self.connection.lock().unwrap();
        
        match conn.set_ex::<_, _, ()>(&key, data, ttl_seconds.try_into().unwrap()) {
            Ok(_) => {
                debug!("Cached certificate in Redis for domain: {} (TTL: {} seconds)", domain, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                error!("Failed to cache certificate in Redis for domain {}: {}", domain, e);
                Err(anyhow!("Redis set error: {}", e))
            }
        }
    }

    fn remove(&self, domain: &str) -> Result<()> {
        let key = self.make_key(domain);
        let mut conn = self.connection.lock().unwrap();
        
        match conn.del::<_, i32>(&key) {
            Ok(count) => {
                if count > 0 {
                    debug!("Removed certificate from Redis cache for domain: {}", domain);
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove certificate from Redis for domain {}: {}", domain, e);
                Err(anyhow!("Redis delete error: {}", e))
            }
        }
    }

    fn clear(&self) -> Result<()> {
        let pattern = format!("{}*", self.key_prefix);
        let mut conn = self.connection.lock().unwrap();
        
        // Get all keys with the prefix
        let keys: Vec<String> = conn.keys(&pattern)
            .map_err(|e| anyhow!("Failed to get Redis keys: {}", e))?;
            
        if !keys.is_empty() {
            let count: i32 = conn.del(&keys)
                .map_err(|e| anyhow!("Failed to delete Redis keys: {}", e))?;
            info!("Cleared Redis certificate cache ({} entries removed)", count);
        }
        
        Ok(())
    }

    fn cache_info(&self) -> String {
        let mut conn = self.connection.lock().unwrap();
        let pattern = format!("{}*", self.key_prefix);
        
        match conn.keys::<_, Vec<String>>(&pattern) {
            Ok(keys) => format!("Redis cache: {} entries (prefix: {})", keys.len(), self.key_prefix),
            Err(_) => format!("Redis cache: unknown entries (prefix: {})", self.key_prefix),
        }
    }
}

/// Certificate cache factory
pub struct CertificateManager {
    cache: Box<dyn CertificateCache>,
    default_ttl: u64,
}

impl CertificateManager {
    /// Create a new certificate manager with configuration-based backend selection
    pub fn with_config(tls_config: &crate::config::settings::TlsConfig, redis_config: &crate::config::settings::RedisConfig) -> Result<Self> {
        let default_ttl = 24 * 60 * 60; // 24 hours
        
        match tls_config.certificate_storage.as_str() {
            "redis" => {
                #[cfg(feature = "redis-support")]
                {
                    match RedisCache::new(&redis_config.url, Some("proxy:cert:".to_string())) {
                        Ok(redis_cache) => {
                            info!("🚀 Using Redis certificate cache (configured)");
                            Ok(Self {
                                cache: Box::new(redis_cache),
                                default_ttl,
                            })
                        }
                        Err(e) => {
                            error!("Failed to connect to Redis for certificate storage: {}", e);
                            return Err(anyhow!("Redis certificate storage configured but Redis connection failed: {}", e));
                        }
                    }
                }
                #[cfg(not(feature = "redis-support"))]
                {
                    return Err(anyhow!("Redis certificate storage configured but redis-support feature not enabled"));
                }
            }
            "cache" => {
                info!("🧠 Using in-memory certificate cache (configured)");
                Ok(Self {
                    cache: Box::new(MemoryCache::new(1000)), // Max 1000 certificates
                    default_ttl,
                })
            }
            storage_type => {
                return Err(anyhow!("Invalid certificate storage type '{}'. Must be 'cache' or 'redis'", storage_type));
            }
        }
    }

    /// Create a new certificate manager with automatic backend selection (deprecated)
    /// This method is kept for backward compatibility
    pub fn new() -> Self {
        let default_ttl = 24 * 60 * 60; // 24 hours
        
        // Try to use Redis if available and enabled
        #[cfg(feature = "redis-support")]
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            match RedisCache::new(&redis_url, Some("proxy:cert:".to_string())) {
                Ok(redis_cache) => {
                    info!("🚀 Using Redis certificate cache");
                    return Self {
                        cache: Box::new(redis_cache),
                        default_ttl,
                    };
                }
                Err(e) => {
                    warn!("Failed to connect to Redis cache: {}", e);
                    info!("Falling back to in-memory certificate cache");
                }
            }
        }
        
        // Fallback to in-memory cache
        info!("🧠 Using in-memory certificate cache");
        Self {
            cache: Box::new(MemoryCache::new(1000)), // Max 1000 certificates
            default_ttl,
        }
    }

    /// Get certificate from cache
    pub fn get_certificate(&self, domain: &str) -> Result<Option<CertificateData>> {
        self.cache.get(domain)
    }

    /// Cache a certificate
    pub fn cache_certificate(&self, domain: &str, cert_data: CertificateData) -> Result<()> {
        self.cache.set(domain, cert_data, self.default_ttl)
    }

    /// Cache a certificate with custom TTL
    pub fn cache_certificate_with_ttl(&self, domain: &str, cert_data: CertificateData, ttl_seconds: u64) -> Result<()> {
        self.cache.set(domain, cert_data, ttl_seconds)
    }

    /// Remove certificate from cache
    pub fn remove_certificate(&self, domain: &str) -> Result<()> {
        self.cache.remove(domain)
    }

    /// Clear all cached certificates
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear()
    }

    /// Get cache information
    pub fn cache_info(&self) -> String {
        self.cache.cache_info()
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}

