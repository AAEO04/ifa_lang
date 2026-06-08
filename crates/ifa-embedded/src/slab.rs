use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "alloc")]
extern crate alloc;

/// Slab size classes for the slab allocator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlabClass {
    Tiny = 0,   // 256 bytes
    Small = 1,  // 1 KB
    Medium = 2, // 4 KB
    Large = 3,  // 16 KB
    Huge = 4,   // 64 KB
    Giant = 5,  // 256 KB
    Mega = 6,   // 1 MB
}

impl SlabClass {
    pub const fn size(&self) -> u64 {
        match self {
            SlabClass::Tiny => 256,
            SlabClass::Small => 1024,
            SlabClass::Medium => 4 * 1024,
            SlabClass::Large => 16 * 1024,
            SlabClass::Huge => 64 * 1024,
            SlabClass::Giant => 256 * 1024,
            SlabClass::Mega => 1024 * 1024,
        }
    }

    pub fn from_size(size: u64) -> Option<Self> {
        if size <= 256 {
            Some(SlabClass::Tiny)
        } else if size <= 1024 {
            Some(SlabClass::Small)
        } else if size <= 4 * 1024 {
            Some(SlabClass::Medium)
        } else if size <= 16 * 1024 {
            Some(SlabClass::Large)
        } else if size <= 64 * 1024 {
            Some(SlabClass::Huge)
        } else if size <= 256 * 1024 {
            Some(SlabClass::Giant)
        } else if size <= 1024 * 1024 {
            Some(SlabClass::Mega)
        } else {
            None
        }
    }

    pub const COUNT: usize = 7;
}

/// Generic slab tracker that manages free/allocated slots via an atomic bitmap.
pub struct SlabTracker {
    pub slot_size: u64,
    pub slot_count: usize,
    #[cfg(feature = "alloc")]
    pub free_bitmap: alloc::vec::Vec<AtomicUsize>,
    #[cfg(not(feature = "alloc"))]
    pub free_bitmap: heapless::Vec<AtomicUsize, 64>, // supports up to 4096 slots
}

impl SlabTracker {
    pub fn new(slot_size: u64, slot_count: usize) -> Self {
        let words = slot_count.div_ceil(64);

        #[cfg(feature = "alloc")]
        let free_bitmap: alloc::vec::Vec<AtomicUsize> =
            (0..words).map(|_| AtomicUsize::new(!0)).collect();

        #[cfg(not(feature = "alloc"))]
        let mut free_bitmap = heapless::Vec::new();
        #[cfg(not(feature = "alloc"))]
        {
            for _ in 0..words {
                let _ = free_bitmap.push(AtomicUsize::new(!0));
            }
        }

        Self {
            slot_size,
            slot_count,
            free_bitmap,
        }
    }

    pub fn allocate(&self) -> Option<usize> {
        for (word_idx, word) in self.free_bitmap.iter().enumerate() {
            let mut retries = 0;
            loop {
                let current = word.load(Ordering::Relaxed);
                if current == 0 {
                    break;
                }

                let bit_idx = current.trailing_zeros() as usize;
                let slot_idx = word_idx * 64 + bit_idx;

                if slot_idx >= self.slot_count {
                    break;
                }

                let new_value = current & !(1 << bit_idx);

                match word.compare_exchange_weak(
                    current,
                    new_value,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Some(slot_idx),
                    Err(_) => {
                        retries += 1;
                        if retries > 100 {
                            // On non-std targets, yield_now is not easily available,
                            // but spin loops can just yield hint.
                            core::hint::spin_loop();
                            if retries > 200 {
                                break;
                            }
                        }
                        continue;
                    }
                }
            }
        }
        None
    }

    pub fn free(&self, slot_idx: usize) {
        let word_idx = slot_idx / 64;
        let bit_idx = slot_idx % 64;

        if word_idx < self.free_bitmap.len() {
            self.free_bitmap[word_idx].fetch_or(1 << bit_idx, Ordering::SeqCst);
        }
    }
}

/// Handle for a slab allocation
#[derive(Debug, Clone)]
pub struct SlabAllocation {
    pub class: SlabClass,
    pub slab_idx: usize,
    pub slot_idx: usize,
    pub offset: u64,
    pub size: u64,
}
