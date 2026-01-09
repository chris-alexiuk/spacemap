use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A min-heap with a fixed maximum capacity.
///
/// This data structure efficiently tracks the top N largest items by:
/// 1. Maintaining a min-heap of size N (the N largest items seen so far)
/// 2. For each new item, comparing it against the minimum (smallest of the top N)
/// 3. If the new item is larger, evicting the minimum and inserting the new item
///
/// Complexity:
/// - Push: O(log N) where N is the capacity
/// - Memory: O(N) instead of O(total items)
///
/// This is much more efficient than collecting all items and sorting:
/// - For 1M items with capacity 10: O(1M * log 10) vs O(1M * log 1M)
/// - Memory: 10 items vs 1M items
pub struct BoundedMinHeap<T: Ord> {
    heap: BinaryHeap<Reverse<T>>,
    capacity: usize,
}

impl<T: Ord> BoundedMinHeap<T> {
    /// Creates a new bounded min-heap with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity + 1),
            capacity,
        }
    }

    /// Attempts to push an item into the heap.
    ///
    /// If the heap is not full, the item is added.
    /// If the heap is full and the item is larger than the minimum,
    /// the minimum is removed and the item is added.
    /// Otherwise, the item is discarded.
    pub fn push(&mut self, item: T) {
        if self.heap.len() < self.capacity {
            self.heap.push(Reverse(item));
        } else if let Some(&Reverse(ref min)) = self.heap.peek() {
            if &item > min {
                self.heap.pop();
                self.heap.push(Reverse(item));
            }
        }
    }

    /// Consumes the heap and returns the items in descending order (largest first).
    pub fn into_sorted_vec(self) -> Vec<T> {
        let mut vec: Vec<_> = self.heap.into_iter().map(|Reverse(x)| x).collect();
        vec.sort_by(|a, b| b.cmp(a)); // Descending order
        vec
    }

    /// Returns the number of items currently in the heap.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns true if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_heap_basic() {
        let mut heap = BoundedMinHeap::new(3);

        heap.push(5);
        heap.push(2);
        heap.push(8);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, vec![8, 5, 2]);
    }

    #[test]
    fn test_bounded_heap_eviction() {
        let mut heap = BoundedMinHeap::new(3);

        // Fill heap with [5, 2, 8]
        heap.push(5);
        heap.push(2);
        heap.push(8);

        // Push 10 - should evict 2 (smallest)
        heap.push(10);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, vec![10, 8, 5]);
    }

    #[test]
    fn test_bounded_heap_no_eviction_if_smaller() {
        let mut heap = BoundedMinHeap::new(3);

        heap.push(5);
        heap.push(8);
        heap.push(10);

        // Push 1 - should be ignored (smaller than min)
        heap.push(1);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, vec![10, 8, 5]);
    }

    #[test]
    fn test_bounded_heap_capacity() {
        let mut heap = BoundedMinHeap::new(5);

        for i in 0..100 {
            heap.push(i);
        }

        assert_eq!(heap.len(), 5);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, vec![99, 98, 97, 96, 95]);
    }

    #[test]
    fn test_bounded_heap_empty() {
        let heap: BoundedMinHeap<i32> = BoundedMinHeap::new(10);
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, Vec::<i32>::new());
    }

    #[test]
    fn test_bounded_heap_single_item() {
        let mut heap = BoundedMinHeap::new(5);
        heap.push(42);

        assert_eq!(heap.len(), 1);

        let sorted = heap.into_sorted_vec();
        assert_eq!(sorted, vec![42]);
    }
}
