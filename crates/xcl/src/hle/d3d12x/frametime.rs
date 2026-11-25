//! Crimson: where the time of a slow frame goes (`XCL_ACTIVITY=1`).
//!
//! Wall time spent inside the calls where XCL or the PC's driver can keep one of the title's
//! threads, by kind, and how far behind the GPU is when a frame is shown. Times are summed
//! over all threads (several submit at once, so a part can exceed the 2 s it is reported
//! for), and a part can contain another (a submission contains the driver's share of it).
//! What is left of a frame's time after these is the title's own.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum Part {
    /// ExecuteCommandLists as the title calls it: XCL's uploads and mappings, then the driver
    Submit = 0,
    /// the driver's ExecuteCommandLists inside a submission
    DriverSubmit = 1,
    /// showing a frame: the copy to the back buffer and the swap chain's Present
    Present = 2,
    /// the swap chain's Present alone
    Flip = 3,
    /// WaitFrameEventX: the title waits for its next frame slot
    FrameWait = 4,
    /// a bind waits for a pipeline still being built
    PipelineWait = 5,
    // the rest of a submission, piece by piece
    /// textures in title memory the CPU wrote since, uploaded again
    TexRefresh = 6,
    /// the title's lists mapped to the PC lists recorded for them
    ListMap = 7,
    /// scratch copies of the lists just submitted
    Scratch = 8,
    /// allocators and fences of the lists just submitted
    Fences = 9,
    /// GPU writes into sparse windows read back
    SparseReadback = 10,
    /// writes the CPU does after the GPU
    CpuWrites = 11,
    /// the states the lists leave textures kept in step in
    TexSubmitted = 12,
    /// the presenter's and the watchers' look at the lists
    NoteExecute = 13,
}

const PARTS: usize = 14;
pub const NAMES: [&str; PARTS] = [
    "submits",
    "driver in submits",
    "present",
    "flip",
    "frame-event waits",
    "pipeline waits",
    "texture refresh",
    "list mapping",
    "scratch copies",
    "allocators and fences",
    "sparse readback",
    "CPU writes",
    "texture states",
    "submit watchers",
];

static NANOS: [AtomicU64; PARTS] = [const { AtomicU64::new(0) }; PARTS];
static CALLS: [AtomicU64; PARTS] = [const { AtomicU64::new(0) }; PARTS];
/// the longest single call of each part since the last report
static WORST: [AtomicU64; PARTS] = [const { AtomicU64::new(0) }; PARTS];
/// frames shown, the sum over them of frames the GPU had not finished, the most at once
static BACKLOG: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];

pub struct Timed(Part, Instant);

/// Times the caller's scope as `part`.
pub fn timed(part: Part) -> Timed {
    Timed(part, Instant::now())
}

impl Drop for Timed {
    fn drop(&mut self) {
        let i = self.0 as usize;
        let ns = self.1.elapsed().as_nanos() as u64;
        NANOS[i].fetch_add(ns, Ordering::Relaxed);
        CALLS[i].fetch_add(1, Ordering::Relaxed);
        WORST[i].fetch_max(ns, Ordering::Relaxed);
    }
}

/// At a frame's submission: `behind` earlier frames' copies are not finished on the GPU yet.
pub fn gpu_behind(behind: u64) {
    BACKLOG[0].fetch_add(1, Ordering::Relaxed);
    BACKLOG[1].fetch_add(behind, Ordering::Relaxed);
    BACKLOG[2].fetch_max(behind, Ordering::Relaxed);
}

/// The report since the last call (one log line), or None when nothing was timed.
pub fn report() -> Option<String> {
    let mut parts = Vec::new();
    for i in 0..PARTS {
        let ns = NANOS[i].swap(0, Ordering::Relaxed);
        let calls = CALLS[i].swap(0, Ordering::Relaxed);
        let worst = WORST[i].swap(0, Ordering::Relaxed);
        if calls != 0 {
            parts.push(format!("{} {} ms in {} (longest {} ms)", NAMES[i], ns / 1_000_000, calls, worst / 1_000_000));
        }
    }
    let frames = BACKLOG[0].swap(0, Ordering::Relaxed);
    let sum = BACKLOG[1].swap(0, Ordering::Relaxed);
    let most = BACKLOG[2].swap(0, Ordering::Relaxed);
    if frames != 0 {
        parts.push(format!("GPU behind by {:.1} frames on average when one is shown (most {})", sum as f64 / frames as f64, most));
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}
