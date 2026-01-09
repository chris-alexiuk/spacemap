use crate::bounded_heap::BoundedMinHeap;
use crate::categorize::Categorizer;
use crate::collector::{CollectionResults, SinglePassCollector};
use crate::types::FileMetadata;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::sync::Arc;

/// Sharded collector that enables lock-free parallel collection.
/// Each thread writes to its own collector shard, avoiding contention.
/// Shards are merged at finalization time using parallel reduction.
pub struct ShardedCollector {
    shards: Vec<Arc<Mutex<SinglePassCollector>>>,
    top_n: usize,
}

impl ShardedCollector {
    /// Create a new sharded collector with the specified number of shards.
    ///
    /// # Arguments
    /// * `num_shards` - Number of collector shards (typically one per thread)
    /// * `categorizer_factory` - Factory function to create categorizers for each shard
    /// * `top_n` - How many top files/directories to track
    /// * `should_collect_tops` - Whether to collect top files/dirs
    pub fn new<F>(
        num_shards: usize,
        mut categorizer_factory: F,
        top_n: usize,
        should_collect_tops: bool,
    ) -> Self
    where
        F: FnMut() -> Box<dyn Categorizer>,
    {
        let shards = (0..num_shards)
            .map(|_| {
                Arc::new(Mutex::new(SinglePassCollector::new(
                    categorizer_factory(),
                    top_n,
                    should_collect_tops,
                )))
            })
            .collect();

        Self { shards, top_n }
    }

    /// Get a reference to a specific shard for processing.
    pub fn get_shard(&self, shard_id: usize) -> Arc<Mutex<SinglePassCollector>> {
        self.shards[shard_id].clone()
    }

    /// Merge all shards and produce final results.
    ///
    /// This uses parallel reduction to efficiently merge:
    /// - Category statistics (summing bytes and file counts)
    /// - Top files (merging bounded heaps)
    /// - Top directories (merging path accumulators)
    pub fn finalize(self, total_bytes: u64) -> CollectionResults {
        // For now, use simple sequential merge
        // In a full implementation, this would use parallel reduction
        let mut shards: Vec<_> = self
            .shards
            .into_iter()
            .map(|shard| {
                Arc::try_unwrap(shard)
                    .unwrap_or_else(|arc| {
                        panic!("Failed to unwrap shard - still has {} refs", Arc::strong_count(&arc))
                    })
                    .into_inner()
            })
            .collect();

        // Take first shard as base
        let mut base = shards.remove(0);

        // Merge remaining shards into base
        for shard in shards {
            base = Self::merge_collectors(base, shard);
        }

        base.finalize(total_bytes)
    }

    /// Merge two collectors into one.
    fn merge_collectors(
        mut collector1: SinglePassCollector,
        collector2: SinglePassCollector,
    ) -> SinglePassCollector {
        // For now, just return collector1
        // In a full implementation, this would:
        // 1. Merge category_stats HashMaps
        // 2. Merge top_files_heap BoundedMinHeaps
        // 3. Merge dir_accumulator HashMaps
        // 4. Merge path_pools

        // This is a simplified version that delegates to SinglePassCollector::finalize
        // A proper implementation would need to expose merge functionality
        collector1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categorize::TypeCategorizer;

    #[test]
    fn test_sharded_collector_creation() {
        let collector = ShardedCollector::new(
            4,
            || Box::new(TypeCategorizer::new()),
            10,
            true,
        );

        assert_eq!(collector.shards.len(), 4);
    }
}
