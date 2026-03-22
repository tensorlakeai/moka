//! Admission types for concurrent caches.
//!
//! These types are used for admission decisions in the cache eviction policy.

use crate::common::{concurrent::KeyHash, frequency_sketch::FrequencySketch, time::Instant};

use smallvec::SmallVec;

/// Counters for tracking cache eviction statistics.
pub(crate) struct EvictionCounters {
    pub(crate) entry_count: u64,
    pub(crate) weighted_size: u64,
    pub(crate) eviction_count: u64,
}

impl EvictionCounters {
    #[inline]
    pub(crate) fn new(entry_count: u64, weighted_size: u64) -> Self {
        Self {
            entry_count,
            weighted_size,
            eviction_count: 0,
        }
    }

    #[inline]
    pub(crate) fn saturating_add(&mut self, entry_count: u64, weight: u32) {
        self.entry_count += entry_count;
        let total = &mut self.weighted_size;
        *total = total.saturating_add(weight as u64);
    }

    #[inline]
    pub(crate) fn saturating_sub(&mut self, entry_count: u64, weight: u32) {
        self.entry_count -= entry_count;
        let total = &mut self.weighted_size;
        *total = total.saturating_sub(weight as u64);
    }

    #[inline]
    pub(crate) fn incr_eviction_count(&mut self) {
        let count = &mut self.eviction_count;
        *count = count.saturating_add(1);
    }
}

/// Aggregated metrics for admission decisions.
#[derive(Default)]
pub(crate) struct EntrySizeAndFrequency {
    pub(crate) policy_weight: u64,
    pub(crate) freq: u32,
}

impl EntrySizeAndFrequency {
    pub(crate) fn new(policy_weight: u32) -> Self {
        Self {
            policy_weight: policy_weight as u64,
            ..Default::default()
        }
    }

    pub(crate) fn add_policy_weight(&mut self, weight: u32) {
        self.policy_weight += weight as u64;
    }

    pub(crate) fn add_frequency(&mut self, freq: &FrequencySketch, hash: u64) {
        self.freq += freq.frequency(hash) as u32;
    }
}

/// Result of an admission decision.
///
/// NOTE: Clippy found that the `Admitted` variant contains at least a few hundred
/// bytes of data and the `Rejected` variant contains no data at all. It suggested to
/// box the `SmallVec`.
///
/// We ignore the suggestion because (1) the `SmallVec` is used to avoid heap
/// allocation as it will be used in a performance hot spot, and (2) this enum has a
/// very short lifetime and there will only one instance at a time.
#[allow(clippy::large_enum_variant)]
pub(crate) enum AdmissionResult<K> {
    Admitted {
        /// A vec of pairs of `KeyHash` and `last_accessed`.
        victim_keys: SmallVec<[(KeyHash<K>, Option<Instant>); 8]>,
    },
    Rejected,
}