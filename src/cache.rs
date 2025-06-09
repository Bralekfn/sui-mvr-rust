use crate::error::{MvrError, MvrResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};

/// Cached resolution entry
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry {
    pub value: String,
    pub expires_at: Instant,
    pub hit_count: u64,
    pub last_accessed: Instant,
}

impl CacheEntry {
    pub fn new(value: String, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            expires_at: now + ttl,
            hit_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    pub fn access(&mut self) -> String {
        self.hit_count += 1;
        self.last_accessed = Instant::now();
        self.value.clone()
    }
}

/// In-memory cache for MVR resolutions
#[derive(Debug, Clone)]
pub(crate) struct MvrCache {
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
    max_size: usize,
}

impl MvrCache {
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))
            .ok()?;

        if let Some(entry) = entries.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.access());
            } else {
                // Remove expired entry
                entries.remove(key);
            }
        }
        None
    }

    pub fn insert(&self, key: String, value: String) -> MvrResult<()> {
        self.insert_with_ttl(key, value, self.default_ttl)
    }

    pub fn insert_with_ttl(&self, key: String, value: String, ttl: Duration) -> MvrResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))?;

        // Check if we need to evict entries
        if entries.len() >= self.max_size {
            self.evict_lru(&mut entries);
        }

        let entry = CacheEntry::new(value, ttl);
        entries.insert(key, entry);
        Ok(())
    }

    pub fn remove(&self, key: &str) -> MvrResult<Option<String>> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))?;

        Ok(entries.remove(key).map(|entry| entry.value))
    }

    pub fn clear(&self) -> MvrResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))?;

        entries.clear();
        Ok(())
    }

    pub fn stats(&self) -> MvrResult<CacheStats> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))?;

        let total_entries = entries.len();
        let expired_entries = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .count();

        let total_hits: u64 = entries.iter().map(|(_, entry)| entry.hit_count).sum();

        Ok(CacheStats {
            total_entries,
            expired_entries,
            valid_entries: total_entries - expired_entries,
            total_hits,
            max_size: self.max_size,
        })
    }

    pub fn cleanup_expired(&self) -> MvrResult<usize> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| MvrError::CacheError("Failed to acquire cache lock".to_string()))?;

        let initial_size = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        Ok(initial_size - entries.len())
    }

    fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>) {
        if entries.is_empty() {
            return;
        }

        // Find the least recently used entry
        let lru_key = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            entries.remove(&key);
        }
    }

    /// Create cache key for package resolution
    pub fn package_key(package_name: &str) -> String {
        format!("pkg:{}", package_name)
    }

    /// Create cache key for type resolution
    pub fn type_key(type_name: &str) -> String {
        format!("type:{}", type_name)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
    pub total_hits: u64,
    pub max_size: usize,
}

impl CacheStats {
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            self.total_entries as f64 / self.max_size as f64
        }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_hits == 0 {
            0.0
        } else {
            // Fixed: Convert total_entries to u64 to match total_hits type
            self.total_hits as f64 / (self.total_hits + self.total_entries as u64) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = MvrCache::new(Duration::from_secs(1), 10);

        // Test insertion and retrieval
        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Test non-existent key
        assert_eq!(cache.get("nonexistent"), None);

        // Test removal
        let removed = cache.remove("key1").unwrap();
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(cache.get("key1"), None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MvrCache::new(Duration::from_millis(100), 10);

        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1"), None);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let cache = MvrCache::new(Duration::from_secs(10), 2);

        // Fill cache to capacity
        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();
        cache
            .insert("key2".to_string(), "value2".to_string())
            .unwrap();

        // Access key1 to make it more recently used
        cache.get("key1");

        // Insert key3, should evict key2 (LRU)
        cache
            .insert("key3".to_string(), "value3".to_string())
            .unwrap();

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let cache = MvrCache::new(Duration::from_secs(1), 10);

        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();
        cache
            .insert("key2".to_string(), "value2".to_string())
            .unwrap();

        // Access key1 multiple times
        cache.get("key1");
        cache.get("key1");

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);
        assert!(stats.total_hits >= 2);
    }

    #[test]
    fn test_cache_key_functions() {
        assert_eq!(MvrCache::package_key("@test/pkg"), "pkg:@test/pkg");
        assert_eq!(
            MvrCache::type_key("@test/pkg::Type"),
            "type:@test/pkg::Type"
        );
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = MvrCache::new(Duration::from_millis(50), 10);

        // Insert entries with short TTL
        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();
        cache
            .insert("key2".to_string(), "value2".to_string())
            .unwrap();

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        let removed = cache.cleanup_expired().unwrap();
        assert_eq!(removed, 2);

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_cache_clone() {
        let cache = MvrCache::new(Duration::from_secs(1), 10);
        let cloned_cache = cache.clone();

        // Insert in original
        cache
            .insert("key1".to_string(), "value1".to_string())
            .unwrap();

        // Should be accessible from clone (shared Arc)
        assert_eq!(cloned_cache.get("key1"), Some("value1".to_string()));
    }
}