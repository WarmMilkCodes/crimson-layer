//! Crimson: the compute work a title last gave the GPU, kept to be logged when the device is
//! lost. A retail game's world starts with a burst of dispatches (12 800 in two seconds), the
//! GPU stops answering and Windows removes the device – with every pipeline built, nothing
//! skipped, and DRED not usable with this title. This says which programs the burst was made
//! of, how often each ran and with how many groups: a program run thousands of times, or with
//! group counts from memory nothing wrote, is where to look.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Clone, Copy)]
struct Entry {
    ms: u64,
    /// 0 Dispatch, 1 ExecuteIndirect (compute), 2 ExecuteIndirect (graphics)
    kind: u8,
    pso: usize,
    args: [u32; 3],
}

const KEPT: usize = 16384;

static RING: Mutex<VecDeque<Entry>> = Mutex::new(VecDeque::new());
/// pipeline → (console record's hash, bytes of DXIL)
static NAMES: Mutex<Option<HashMap<usize, (u64, usize)>>> = Mutex::new(None);

fn now_ms() -> u64 {
    static T0: OnceLock<Instant> = OnceLock::new();
    T0.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// A translated compute pipeline: the console record it was made from.
pub fn name(pso: usize, record: u64, dxil: usize) {
    if pso != 0 {
        NAMES.lock().unwrap().get_or_insert_with(HashMap::new).insert(pso, (record, dxil));
    }
}

/// Work recorded on a list (`kind` as in `Entry`; a Dispatch's groups, an ExecuteIndirect's
/// greatest count).
pub fn note(kind: u8, pso: usize, args: [u32; 3]) {
    let e = Entry { ms: now_ms(), kind, pso, args };
    let mut r = RING.lock().unwrap();
    if r.len() == KEPT {
        r.pop_front();
    }
    r.push_back(e);
}

fn named(pso: usize) -> String {
    match NAMES.lock().unwrap().as_ref().and_then(|m| m.get(&pso).copied()) {
        Some((record, dxil)) => format!("record {record:016x} ({dxil} B DXIL)"),
        None => format!("pipeline {pso:#x}"),
    }
}

/// The device is lost: what was recorded in the 10 s before the last entry.
pub fn dump() {
    let ring: Vec<Entry> = RING.lock().unwrap().iter().copied().collect();
    let Some(last) = ring.last() else { return };
    let from = last.ms.saturating_sub(10_000);
    let recent: Vec<&Entry> = ring.iter().filter(|e| e.ms >= from).collect();
    // by pipeline: how often, the most groups of one dispatch, an example
    let mut by: HashMap<(usize, u8), (u64, u64, [u32; 3])> = HashMap::new();
    for e in &recent {
        let groups = if e.kind == 0 { e.args.iter().map(|&a| a as u64).product() } else { e.args[0] as u64 };
        let b = by.entry((e.pso, e.kind)).or_insert((0, 0, e.args));
        b.0 += 1;
        if groups >= b.1 {
            b.1 = groups;
            b.2 = e.args;
        }
    }
    crate::xlog_error!(
        "gpu work: {} dispatches / indirect calls recorded in the 10 s before the last one ({} ms before now), of {} pipelines",
        recent.len(),
        now_ms().saturating_sub(last.ms),
        by.len()
    );
    let kind = |k: u8| ["Dispatch", "ExecuteIndirect (compute)", "ExecuteIndirect (graphics)"][k.min(2) as usize];
    let mut often: Vec<_> = by.iter().collect();
    often.sort_by_key(|(_, v)| std::cmp::Reverse(v.0));
    for ((pso, k), (n, most, args)) in often.iter().take(14) {
        crate::xlog_error!("gpu work:   {n} x {} of {} – most groups {most} {args:?}", kind(*k), named(*pso));
    }
    let mut big: Vec<_> = by.iter().collect();
    big.sort_by_key(|(_, v)| std::cmp::Reverse(v.1));
    for ((pso, k), (n, most, args)) in big.iter().take(8) {
        crate::xlog_error!("gpu work:   most groups {most} {args:?}: {} of {} ({n} x)", kind(*k), named(*pso));
    }
    for e in recent.iter().rev().take(24).rev() {
        crate::xlog_error!("gpu work:   -{} ms {} {:?} of {}", last.ms - e.ms, kind(e.kind), e.args, named(e.pso));
    }
}
