pub struct Arena<T> {
    storage: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn allocate(&mut self, value: T) -> &T {
        self.storage.push(value);
        &self.storage[self.storage.len() - 1]
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    pub fn reset(&mut self) {
        self.storage.clear();
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_single_allocation() {
        let mut arena = Arena::new();

        let value = arena.allocate(42);

        assert_eq!(*value, 42);
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut arena = Arena::new();

        {
            let first = arena.allocate(10);
            assert_eq!(*first, 10);
        }

        {
            let second = arena.allocate(20);
            assert_eq!(*second, 20);
        }

        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_reset_clears() {
        let mut arena = Arena::new();

        arena.allocate(1);
        arena.allocate(2);
        arena.allocate(3);

        let capacity_before = arena.capacity();

        arena.reset();

        assert_eq!(arena.len(), 0);
        assert_eq!(arena.capacity(), capacity_before);
    }

    #[test]
    fn test_capacity_reuse() {
        let mut arena = Arena::new();

        for i in 0..16 {
            arena.allocate(i);
        }

        let capacity_before = arena.capacity();

        arena.reset();

        for i in 100..116 {
            arena.allocate(i);
        }

        assert_eq!(arena.capacity(), capacity_before);
    }

    #[test]
    fn test_arena_drop_cascades() {
        let drop_count = Cell::new(0);
        {
            let mut arena = Arena::new();
            arena.allocate(TraceDrop {
                counter: &drop_count,
            });

            arena.allocate(TraceDrop {
                counter: &drop_count,
            });

            assert_eq!(arena.len(), 2);
            assert_eq!(drop_count.get(), 0);
        }
        assert_eq!(drop_count.get(), 2);
    }

    struct TraceDrop<'a> {
        counter: &'a Cell<usize>,
    }

    impl<'a> Drop for TraceDrop<'a> {
        fn drop(&mut self) {
            self.counter.set(self.counter.get() + 1);
        }
    }
}
