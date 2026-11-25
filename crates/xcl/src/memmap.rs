//! Crimson: a census of the process's committed memory by allocation (`XCL_ACTIVITY=1`, every
//! 30 s): which allocations grew since the last one, and how many of each size there are. The
//! process's commit grew ~75 MB a second on a loading screen with XCL's heap, the video memory
//! and every pool XCL reports flat – this names the addresses and sizes that grow, whoever
//! made them (the title, the driver, Windows).

use std::collections::HashMap;
use std::sync::Mutex;

/// committed private bytes per allocation base, at the last census
static LAST: Mutex<Option<HashMap<usize, u64>>> = Mutex::new(None);

const CLASSES: [(u64, &str); 6] = [(64 << 10, "<=64K"), (1 << 20, "<=1M"), (16 << 20, "<=16M"), (256 << 20, "<=256M"), (4 << 30, "<=4G"), (u64::MAX, ">4G")];

fn class(bytes: u64) -> usize {
    CLASSES.iter().position(|&(top, _)| bytes <= top).unwrap_or(CLASSES.len() - 1)
}

/// Walks the address space and logs the census (a few ms to some hundred ms: its own thread).
#[cfg(windows)]
pub fn census() {
    use windows_sys::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION};
    const MEM_COMMIT: u32 = 0x1000;
    const MEM_PRIVATE: u32 = 0x20000;
    const MEM_MAPPED: u32 = 0x40000;
    let mut now: HashMap<usize, u64> = HashMap::new();
    let (mut private, mut mapped, mut image) = (0u64, 0u64, 0u64);
    let mut at = 0x10000usize;
    let top = 0x7FFF_FFFE_0000usize;
    while at < top {
        let mut m: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { VirtualQuery(at as *const _, &mut m, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) } == 0 {
            break;
        }
        let size = m.RegionSize;
        if size == 0 {
            break;
        }
        if m.State as u32 == MEM_COMMIT {
            let ty = m.Type as u32;
            if ty == MEM_PRIVATE {
                private += size as u64;
                *now.entry(m.AllocationBase as usize).or_default() += size as u64;
            } else if ty == MEM_MAPPED {
                mapped += size as u64;
            } else {
                image += size as u64;
            }
        }
        at = (m.BaseAddress as usize).saturating_add(size);
    }
    let mut count = [0u64; CLASSES.len()];
    let mut bytes = [0u64; CLASSES.len()];
    for &b in now.values() {
        count[class(b)] += 1;
        bytes[class(b)] += b;
    }
    let by_class: Vec<String> = (0..CLASSES.len()).map(|k| format!("{} {} ({} MB)", CLASSES[k].1, count[k], bytes[k] >> 20)).collect();
    crate::xlog_info!(
        "memory map: private {} MB in {} allocations, mapped {} MB, image {} MB; by committed size: {}",
        private >> 20,
        now.len(),
        mapped >> 20,
        image >> 20,
        by_class.join(", ")
    );
    let mut last = LAST.lock().unwrap();
    if let Some(old) = last.as_ref() {
        // what grew: allocations that are new, by size class, and the ones that grew most
        let mut new_count = [0u64; CLASSES.len()];
        let mut new_bytes = [0u64; CLASSES.len()];
        let mut grown: Vec<(usize, u64, u64)> = Vec::new();
        let mut gone = 0u64;
        for (&base, &b) in &now {
            match old.get(&base) {
                None => {
                    new_count[class(b)] += 1;
                    new_bytes[class(b)] += b;
                    grown.push((base, b, b));
                }
                Some(&was) if b > was => grown.push((base, b, b - was)),
                _ => {}
            }
        }
        for (base, &was) in old {
            gone += was.saturating_sub(now.get(base).copied().unwrap_or(0));
        }
        let new: Vec<String> = (0..CLASSES.len()).filter(|&k| new_count[k] != 0).map(|k| format!("{} {} (+{} MB)", CLASSES[k].1, new_count[k], new_bytes[k] >> 20)).collect();
        grown.sort_by_key(|g| std::cmp::Reverse(g.2));
        let most: Vec<String> = grown.iter().take(8).map(|&(base, b, d)| format!("{base:#x} +{} MB (now {} MB)", d >> 20, b >> 20)).collect();
        crate::xlog_info!("memory map since the last: new allocations {}; freed or shrunk {} MB; grew most: {}", if new.is_empty() { "none".into() } else { new.join(", ") }, gone >> 20, most.join("; "));
    }
    *last = Some(now);
}

#[cfg(not(windows))]
pub fn census() {}

#[cfg(test)]
mod tests {
    #[test]
    fn sizes_fall_into_their_class() {
        assert_eq!(super::class(4096), 0);
        assert_eq!(super::class(64 << 10), 0);
        assert_eq!(super::class((64 << 10) + 1), 1);
        assert_eq!(super::class(3 << 20), 2);
        assert_eq!(super::class(8 << 30), 5);
    }
}
