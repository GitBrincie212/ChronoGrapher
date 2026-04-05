use std::time::Instant;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use dashmap::DashMap;
use chronographer::utils::slotmap::{OwnedSlotMap, SlotMap};

struct CountingAlloc;

static ALLOC: AtomicUsize = AtomicUsize::new(0);
static DEALLOC: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            ALLOC.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) } ;
        DEALLOC.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

macro_rules! inspect {
    ($label: literal: {$($toks: tt)+}) => {
        let prev_alloc = ALLOC.load(Ordering::Acquire);
        let prev_dealloc = DEALLOC.load(Ordering::Acquire);
        let prev_time = Instant::now();

        $($toks)+

        let elapsed = prev_time.elapsed();

        let curr_alloc = ALLOC.load(Ordering::Acquire);
        let curr_dealloc = DEALLOC.load(Ordering::Acquire);

        let delta = (curr_alloc as i128 - curr_dealloc as i128)
                  - (prev_alloc as i128 - prev_dealloc as i128);

        println!("=== {} ===", $label);
        println!("MEMORY USED: {:?}MB", delta as f64 / 1e+6);
        println!("RUNTIME: {:?}", elapsed);
        println!("=======\n");
    };
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn main() {
    const SIZE: usize = 500_000;

    inspect!("DashMap Initialization": {
        let dash_map: DashMap<usize, usize> = DashMap::new();
    });

    inspect!("SlotMap Initialization": {
        let slot_map: OwnedSlotMap<usize> = SlotMap::default();
    });

    inspect!("DashMap Allocations": {
        for i in 0..SIZE {
            dash_map.insert(SIZE - i, i); // To prevent potential caching influence
        }
    });

    inspect!("SlotMap Allocations": {
        for i in 0..SIZE {
            slot_map.blocking_allocate(i);
        }
    });

    inspect!("DashMap Clear": {
        dash_map.clear();
    });

    inspect!("SlotMap Clear": {
        slot_map.blocking_clear();
    });

    let mut keys = Vec::with_capacity(SIZE);
    for i in 0..SIZE {
        dash_map.insert(SIZE - i, i);
        let key = slot_map.blocking_allocate(i);
        keys.push(key);
    }

    inspect!("DashMap Deallocations": {
        for i in 0..SIZE {
            dash_map.remove(&(SIZE - i));
        }
    });

    inspect!("SlotMap Deallocations": {
        for key in keys {
            slot_map.blocking_remove(&key);
        }
    });
}