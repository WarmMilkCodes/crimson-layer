//! A memory gate for heavy background shader work: translation, the direct DXIL translator and
//! the driver's pipeline builds. `dxc::wait_for_memory` watches DXC compiles only, and commit
//! only: with `XCL_DIRECT_DXIL` a compute program never reaches it, and with a big page file
//! commit stays plentiful while physical memory runs out. A precompile beside a browser ran a
//! 32 GB machine out of memory ("memory allocation of 15728640 bytes failed"), and froze the
//! desktop before that.
//!
//! [`admit`] lets a job start when enough memory is free for it:
//! - a precompile job ([`Work::Precompile`]): free physical memory and free commit both at the
//!   reserve (`XCL_MEM_RESERVE_MB`, default 6144) or more – plus `BIG_COST` for a big program
//!   (`XCL_MEM_BIG_KB`, a record of 32 KB or more: the top tenth of a retail game's compute
//!   programs, which take seconds to minutes and GBs). Big programs start at most one a second,
//!   and jobs that waited one a poll, so each sees what the ones before it took (thirteen
//!   threads woken together would all read the same free memory);
//! - a running title's background build ([`Work::Title`]): free commit only (the title's own
//!   memory is what fills the machine; physical memory says nothing there), never fewer than
//!   `TITLE_FLOOR` at once and never waiting past `TITLE_WAIT` – a title that waits for its
//!   pipelines skips passes or meets its watchdog.
//! With no gated job running, the next one always goes. `XCL_MEM_GATE=0`: off.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static RUNNING: AtomicUsize = AtomicUsize::new(0);
static WAITS: AtomicU64 = AtomicU64::new(0);
static WAITED_MS: AtomicU64 = AtomicU64::new(0);
/// ms since `epoch` of the last admission that is spaced (a big program, a job that waited)
static LAST_SPACED: AtomicU64 = AtomicU64::new(0);

const POLL: Duration = Duration::from_millis(250);
/// What a big program may still take after it starts, in bytes.
const BIG_COST: u64 = 3 << 30;
const BIG_SPACING_MS: u64 = 1000;
const WAITED_SPACING_MS: u64 = 250;
const TITLE_RESERVE: u64 = 3 << 30;
const TITLE_FLOOR: usize = 2;
const TITLE_WAIT: Duration = Duration::from_secs(2);

/// What asks to start.
#[derive(Clone, Copy)]
pub enum Work {
    /// `xcl shaders --precompile`: one shader record of `bytes`.
    Precompile { bytes: usize },
    /// A running title's background pipeline build.
    Title,
    /// A pipeline of an earlier run built ahead of its first use (`pso_async` preload): no
    /// hurry, so it waits for physical memory as a precompile does – the preload that built
    /// every pipeline an earlier run used, as fast as it could, ran the machine out of memory.
    Preload,
}

/// Held while a gated job runs.
pub struct Ticket(bool);

impl Drop for Ticket {
    fn drop(&mut self) {
        if self.0 {
            RUNNING.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

fn enabled() -> bool {
    static E: OnceLock<bool> = OnceLock::new();
    *E.get_or_init(|| std::env::var("XCL_MEM_GATE").map_or(true, |v| v.trim() != "0"))
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(default)
}

/// The memory a precompile keeps free for everything else, in bytes.
pub fn reserve() -> u64 {
    static R: OnceLock<u64> = OnceLock::new();
    *R.get_or_init(|| env_u64("XCL_MEM_RESERVE_MB", 6144) << 20)
}

fn big_bytes() -> usize {
    static B: OnceLock<usize> = OnceLock::new();
    *B.get_or_init(|| (env_u64("XCL_MEM_BIG_KB", 32) << 10) as usize)
}

/// Free physical memory and free commit, in bytes (None: unknown).
pub fn available() -> Option<(u64, u64)> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if unsafe { GlobalMemoryStatusEx(&mut m) } == 0 {
            return None;
        }
        Some((m.ullAvailPhys, m.ullAvailPageFile))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Whether `work` may start with `free` (physical, commit) and `running` gated jobs.
fn room(work: Work, free: Option<(u64, u64)>, running: usize, reserve: u64, big: usize) -> bool {
    // with nothing running the next job always goes, so work moves – but nothing waits for the
    // preload: it waits for memory even then
    if running == 0 && !matches!(work, Work::Preload) {
        return true;
    }
    let Some((phys, commit)) = free else { return true };
    match work {
        Work::Precompile { bytes } => {
            let need = reserve + if bytes >= big { BIG_COST } else { 0 };
            phys >= need && commit >= need
        }
        Work::Title => running < TITLE_FLOOR || commit >= TITLE_RESERVE,
        Work::Preload => phys >= reserve && commit >= reserve,
    }
}

fn now_ms() -> u64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    // (+ the spacing: the first spaced admission is not held back)
    EPOCH.get_or_init(Instant::now).elapsed().as_millis() as u64 + BIG_SPACING_MS
}

/// Takes the spaced-admission turn if `spacing` ms have passed since the last one.
fn take_turn(spacing: u64) -> bool {
    let now = now_ms();
    let last = LAST_SPACED.load(Ordering::Acquire);
    now.saturating_sub(last) >= spacing && LAST_SPACED.compare_exchange(last, now, Ordering::AcqRel, Ordering::Acquire).is_ok()
}

/// Waits for room (see the module), then counts the caller as running until the ticket drops.
pub fn admit(work: Work) -> Ticket {
    if !enabled() {
        return Ticket(false);
    }
    let big = matches!(work, Work::Precompile { bytes } if bytes >= big_bytes());
    let mut since: Option<Instant> = None;
    loop {
        let late = matches!(work, Work::Title) && since.is_some_and(|t| t.elapsed() >= TITLE_WAIT);
        let running = RUNNING.load(Ordering::Acquire);
        if late || room(work, available(), running, reserve(), big_bytes()) {
            // spaced: a big program, and whatever waited (a title's builds are not held for it)
            let spacing = match (work, big, since.is_some()) {
                (Work::Title, ..) => 0,
                (Work::Preload, _, true) => WAITED_SPACING_MS,
                (Work::Preload, ..) => 0,
                (_, true, _) => BIG_SPACING_MS,
                (_, _, true) => WAITED_SPACING_MS,
                _ => 0,
            };
            if running == 0 || spacing == 0 || take_turn(spacing) {
                RUNNING.fetch_add(1, Ordering::AcqRel);
                if let Some(t) = since {
                    let ms = t.elapsed().as_millis() as u64;
                    if ms >= POLL.as_millis() as u64 {
                        WAITS.fetch_add(1, Ordering::Relaxed);
                        if WAITED_MS.fetch_add(ms, Ordering::Relaxed) == 0 {
                            crate::xlog_info!("memory gate: background shader work waited {ms} ms for memory (XCL_MEM_GATE=0: off)");
                        }
                    }
                }
                return Ticket(true);
            }
            // (its turn comes within the spacing)
            since.get_or_insert_with(Instant::now);
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        since.get_or_insert_with(Instant::now);
        std::thread::sleep(POLL);
    }
}

/// How often jobs waited for memory, and for how long in all (ms).
pub fn stats() -> (u64, u64) {
    (WAITS.load(Ordering::Relaxed), WAITED_MS.load(Ordering::Relaxed))
}

/// This process below normal priority – its DXC workers and the driver's compile threads with
/// it: a precompile is background work, the desktop and other programs come first.
pub fn background_process() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS};
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1 << 30;
    const BIG: usize = 32 << 10;

    #[test]
    fn with_nothing_running_a_job_always_goes() {
        assert!(room(Work::Precompile { bytes: 1 << 20 }, Some((0, 0)), 0, 6 * GB, BIG));
        assert!(room(Work::Title, Some((0, 0)), 0, 6 * GB, BIG));
    }

    #[test]
    fn a_precompile_job_needs_physical_memory_and_commit() {
        let small = Work::Precompile { bytes: 1000 };
        assert!(room(small, Some((6 * GB, 6 * GB)), 3, 6 * GB, BIG));
        assert!(!room(small, Some((5 * GB, 60 * GB)), 3, 6 * GB, BIG));
        assert!(!room(small, Some((20 * GB, 5 * GB)), 3, 6 * GB, BIG));
    }

    #[test]
    fn a_big_program_needs_its_own_room_too() {
        let big = Work::Precompile { bytes: BIG };
        assert!(!room(big, Some((8 * GB, 60 * GB)), 1, 6 * GB, BIG));
        assert!(room(big, Some((9 * GB, 60 * GB)), 1, 6 * GB, BIG));
    }

    #[test]
    fn the_preload_waits_for_physical_memory() {
        // the title fills physical memory: building ahead stops, the title's own builds go on
        assert!(!room(Work::Preload, Some((4 * GB, 60 * GB)), 1, 6 * GB, BIG));
        assert!(!room(Work::Preload, Some((4 * GB, 60 * GB)), 0, 6 * GB, BIG), "the preload went ahead with nothing running");
        assert!(room(Work::Title, Some((4 * GB, 60 * GB)), 3, 6 * GB, BIG));
        assert!(room(Work::Preload, Some((7 * GB, 60 * GB)), 1, 6 * GB, BIG));
    }

    #[test]
    fn a_titles_builds_watch_commit_only_and_keep_a_floor() {
        // the title itself fills physical memory: that does not hold its builds
        assert!(room(Work::Title, Some((0, 10 * GB)), 8, 6 * GB, BIG));
        assert!(!room(Work::Title, Some((16 * GB, 2 * GB)), 2, 6 * GB, BIG));
        assert!(room(Work::Title, Some((16 * GB, 2 * GB)), 1, 6 * GB, BIG));
    }
}
