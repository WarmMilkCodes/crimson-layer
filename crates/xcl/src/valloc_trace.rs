//! Crimson: `XCL_TRACE_VALLOC=1` – who makes the process's big allocations. Every new region of
//! 8 – 80 MB asked of the kernel (ntdll's NtAllocateVirtualMemory: VirtualAlloc, big heap
//! blocks, the driver's and Windows' own all end there) is logged with its callers, for the
//! first `LIMIT` after `AFTER` (past a title's startup). A retail game's loading screen got
//! 4 – 5 new 15 MB regions a second that nothing freed, and none came through the title's
//! own VirtualAlloc / XMemAlloc (`XCL_TRACE_MEM`).

use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static ORIG: AtomicUsize = AtomicUsize::new(0);
static LOGGED: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();

const LIMIT: u64 = 48;
const AFTER: Duration = Duration::from_secs(60);

thread_local! {
    /// logging allocates: no trace of the trace
    static INSIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

type NtAllocate = unsafe extern "system" fn(*mut c_void, *mut *mut c_void, usize, *mut usize, u32, u32) -> i32;

unsafe extern "system" fn hk_allocate(process: *mut c_void, base: *mut *mut c_void, zero_bits: usize, size: *mut usize, ty: u32, protect: u32) -> i32 {
    let orig: NtAllocate = std::mem::transmute(ORIG.load(Ordering::Acquire));
    let asked = if size.is_null() { 0 } else { *size };
    // a new region (no address given), not a commit inside one
    let new_region = base.is_null() || (*base).is_null();
    let status = orig(process, base, zero_bits, size, ty, protect);
    if status >= 0
        && new_region
        && (8 << 20..=80 << 20).contains(&asked)
        && LOGGED.load(Ordering::Relaxed) < LIMIT
        && START.get().is_some_and(|t| t.elapsed() >= AFTER)
        && INSIDE.try_with(|i| !i.replace(true)).unwrap_or(false)
    {
        if LOGGED.fetch_add(1, Ordering::Relaxed) < LIMIT {
            use windows_sys::Win32::System::Diagnostics::Debug::RtlCaptureStackBackTrace;
            let mut frames = [std::ptr::null_mut::<c_void>(); 14];
            let n = RtlCaptureStackBackTrace(1, frames.len() as u32, frames.as_mut_ptr(), std::ptr::null_mut());
            let callers: Vec<String> = frames[..n as usize].iter().map(|&f| crate::modules::describe_address(f as usize)).collect();
            let got = if base.is_null() { 0 } else { *base as usize };
            crate::xlog_info!("valloc: {} KB (type {ty:#x}, protect {protect:#x}) at {got:#x} from {}", asked >> 10, callers.join(" < "));
        }
        let _ = INSIDE.try_with(|i| i.set(false));
    }
    status
}

/// Hooks the kernel call (once).
pub fn install() {
    START.get_or_init(Instant::now);
    match unsafe { crate::hooks::hook("ntdll.dll", "NtAllocateVirtualMemory", hk_allocate as usize, &ORIG) } {
        Ok(()) => crate::xlog_info!("valloc: tracing new regions of 8 – 80 MB (the first {LIMIT} after {} s)", AFTER.as_secs()),
        Err(e) => crate::xlog_warn!("valloc: {e}"),
    }
}
