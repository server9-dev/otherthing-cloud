//! Response caching system
//!
//! Caches LLM responses to reduce costs and improve performance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Response cache
pub struct ResponseCache {
    /// Cache entries
    entries: HashMap<CacheKey, CacheEntry>,
    /// Cache strategy
    strategy: CacheStrategy,
    /// Maximum cache size
    max_size: usize,
    /// Hit count
    hits: u64,
    /// Miss count
    misses: u64,
}

/// Cache key
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Model name
    pub model: String,
    /// Prompt hash
    pub prompt_hash: u64,
    /// Temperature (rounded)
    pub temperature: u32,
    /// Additional parameters hash
    pub params_hash: u64,
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached response
    pub response: String,
    /// Entry metadata
    pub metadata: CacheMetadata,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
}

/// Cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Token count
    pub tokens: u64,
    /// Original latency in ms
    pub latency_ms: u64,
    /// Cost saved (cumulative)
    pub cost_saved: f64,
}

/// Cache strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time-To-Live based
    TTL { ttl_secs: u64 },
    /// Adaptive (combines LRU and frequency)
    Adaptive,
}

impl ResponseCache {
    /// Create a new response cache
    pub fn new(strategy: CacheStrategy, max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            strategy,
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Get a cached response
    pub fn get(&mut self, key: &CacheKey) -> Option<String> {
        // Check if expired first
        if let Some(entry) = self.entries.get(key) {
            if self.is_expired(entry) {
                self.entries.remove(key);
                self.misses += 1;
                return None;
            }
        }

        // Update and return
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed = Utc::now();
            entry.access_count += 1;
            self.hits += 1;
            Some(entry.response.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put a response in cache
    pub fn put(&mut self, key: CacheKey, response: String, metadata: CacheMetadata) {
        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            self.evict();
        }

        let entry = CacheEntry {
            response,
            metadata,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
        };

        self.entries.insert(key, entry);
    }

    /// Check if entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let CacheStrategy::TTL { ttl_secs } = self.strategy {
            let age = Utc::now() - entry.created_at;
            age.num_seconds() > ttl_secs as i64
        } else {
            false
        }
    }

    /// Evict entries based on strategy
    fn evict(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let to_remove = match self.strategy {
            CacheStrategy::LRU => {
                // Remove least recently accessed
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(key, _)| key.clone())
            }
            CacheStrategy::LFU => {
                // Remove least frequently used
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(key, _)| key.clone())
            }
            CacheStrategy::TTL { .. } => {
                // Remove oldest entry
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(key, _)| key.clone())
            }
            CacheStrategy::Adaptive => {
                // Score based on recency and frequency
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| {
                        let recency_score = (Utc::now() - entry.last_accessed).num_seconds();
                        let frequency_score = entry.access_count as i64;
                        recency_score - frequency_score * 10
                    })
                    .map(|(key, _)| key.clone())
            }
        };

        if let Some(key) = to_remove {
            self.entries.remove(&key);
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hit_rate = if self.hits + self.misses > 0 {
            self.hits as f64 / (self.hits + self.misses) as f64
        } else {
            0.0
        };

        let total_cost_saved: f64 = self
            .entries
            .values()
            .map(|e| e.metadata.cost_saved)
            .sum();

        CacheStats {
            size: self.entries.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            hit_rate,
            total_cost_saved,
        }
    }

    /// Prune expired entries (for TTL strategy)
    pub fn prune_expired(&mut self) {
        if let CacheStrategy::TTL { .. } = self.strategy {
            let expired: Vec<_> = self
                .entries
                .iter()
                .filter(|(_, entry)| self.is_expired(entry))
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired {
                self.entries.remove(&key);
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Total cost saved
    pub total_cost_saved: f64,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(
        model: impl Into<String>,
        prompt: &str,
        temperature: f32,
        params: &str,
    ) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        prompt.hash(&mut hasher);
        let prompt_hash = hasher.finish();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.hash(&mut hasher);
        let params_hash = hasher.finish();

        Self {
            model: model.into(),
            prompt_hash,
            temperature: (temperature * 100.0) as u32,
            params_hash,
        }
    }

    /// Create from components
    pub fn from_components(
        model: impl Into<String>,
        prompt_hash: u64,
        temperature: f32,
        params_hash: u64,
    ) -> Self {
        Self {
            model: model.into(),
            prompt_hash,
            temperature: (temperature * 100.0) as u32,
            params_hash,
        }
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(CacheStrategy::LRU, 1000)
    }
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self::LRU
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ResponseCache::new(CacheStrategy::LRU, 100);
        assert_eq!(cache.max_size, 100);
    }

    #[test]
    fn test_cache_put_get() {
        let mut cache = ResponseCache::default();
        let key = CacheKey::new("model1", "prompt", 0.7, "");

        cache.put(
            key.clone(),
            "response".to_string(),
            CacheMetadata {
                tokens: 100,
                latency_ms: 50,
                cost_saved: 0.01,
            },
        );

        assert_eq!(cache.get(&key).unwrap(), "response");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ResponseCache::default();
        let key = CacheKey::new("model1", "prompt", 0.7, "");

        assert!(cache.get(&key).is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ResponseCache::new(CacheStrategy::LRU, 2);

        let key1 = CacheKey::new("model1", "prompt1", 0.7, "");
        let key2 = CacheKey::new("model1", "prompt2", 0.7, "");
        let key3 = CacheKey::new("model1", "prompt3", 0.7, "");

        let metadata = CacheMetadata {
            tokens: 100,
            latency_ms: 50,
            cost_saved: 0.01,
        };

        cache.put(key1.clone(), "resp1".to_string(), metadata.clone());
        cache.put(key2.clone(), "resp2".to_string(), metadata.clone());
        cache.put(key3.clone(), "resp3".to_string(), metadata.clone());

        // Cache should have 2 entries (oldest evicted)
        assert_eq!(cache.entries.len(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ResponseCache::default();
        let key = CacheKey::new("model1", "prompt", 0.7, "");

        cache.put(
            key.clone(),
            "response".to_string(),
            CacheMetadata {
                tokens: 100,
                latency_ms: 50,
                cost_saved: 0.01,
            },
        );

        cache.get(&key);
        cache.get(&CacheKey::new("model1", "other", 0.7, ""));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
