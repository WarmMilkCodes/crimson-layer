//! Crimson: how much memory XCL's own Rust code holds on its heap. Counted at every allocation
//! (one relaxed atomic add), so `XCL_ACTIVITY=1`'s memory line can tell a leak in XCL from
//! memory the game, the driver or Windows hold: the process's commit grew ~5 GB a minute on a
//! loading screen with every pool XCL reports flat.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, AtomicU64, Ordering};

static LIVE: AtomicIsize = AtomicIsize::new(0);
static ALLOCS: AtomicU64 = AtomicU64::new(0);

pub struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if !p.is_null() {
            LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(l);
        if !p.is_null() {
            LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l);
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let q = System.realloc(p, l, new);
        if !q.is_null() {
            LIVE.fetch_add(new as isize - l.size() as isize, Ordering::Relaxed);
        }
        q
    }
}

#[global_allocator]
static A: Counting = Counting;

/// (bytes XCL holds on its heap now, allocations so far)
pub fn heap() -> (u64, u64) {
    (LIVE.load(Ordering::Relaxed).max(0) as u64, ALLOCS.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_held_buffer_is_counted_and_a_freed_one_is_not() {
        let before = super::heap().0;
        let v = vec![0u8; 64 << 20];
        assert!(super::heap().0 >= before + (64 << 20) - (1 << 20));
        drop(v);
        assert!(super::heap().0 < before + (32 << 20));
    }
}
