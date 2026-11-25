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
    /// 0 Dispatch, 1 ExecuteIndirect (compute), 2 ExecuteIndirect (graphics), 3 DrawInstanced,
    /// 4 DrawIndexedInstanced
    kind: u8,
    pso: usize,
    args: [u32; 3],
    /// the PC command list it was recorded on
    list: usize,
}

const KEPT: usize = 65536;

thread_local! {
    /// the list the calling thread records on (`on_list`): `note` has no list of its own
    static LIST: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The PC list the calling thread's next `note`s are recorded on.
pub fn on_list(list: usize) {
    let _ = LIST.try_with(|l| l.set(list));
}

static RING: Mutex<VecDeque<Entry>> = Mutex::new(VecDeque::new());
/// pipeline → (console record's hash, bytes of DXIL)
static NAMES: Mutex<Option<HashMap<usize, (u64, usize)>>> = Mutex::new(None);
/// graphics pipeline → (hash of its console VS, of its PS)
static GRAPHICS: Mutex<Option<HashMap<usize, (u64, u64)>>> = Mutex::new(None);

/// A translated graphics pipeline: its console shaders.
pub fn name_graphics(pso: usize, vs: u64, ps: u64) {
    if pso != 0 {
        GRAPHICS.lock().unwrap().get_or_insert_with(HashMap::new).insert(pso, (vs, ps));
    }
}

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
    let e = Entry { ms: now_ms(), kind, pso, args, list: LIST.try_with(|l| l.get()).unwrap_or(0) };
    let mut r = RING.lock().unwrap();
    if r.len() == KEPT {
        r.pop_front();
    }
    r.push_back(e);
}

/// `XCL_SKIP_RECORDS=hash,hash,…`: the dispatches (direct and indirect) of the compute
/// pipelines made from these console records are left out – to find which program's work the
/// GPU does not come back from. `indirect` in the list: every compute ExecuteIndirect.
pub fn left_out(kind: u8, pso: usize) -> bool {
    static LIST: OnceLock<(Vec<u64>, bool)> = OnceLock::new();
    let (records, indirect) = LIST.get_or_init(|| {
        let v = std::env::var("XCL_SKIP_RECORDS").unwrap_or_default();
        let parts: Vec<&str> = v.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        (parts.iter().filter_map(|s| u64::from_str_radix(s, 16).ok()).collect(), parts.iter().any(|s| s.eq_ignore_ascii_case("indirect")))
    });
    if records.is_empty() && !*indirect {
        return false;
    }
    let out = (*indirect && kind == 1) || NAMES.lock().unwrap().as_ref().and_then(|m| m.get(&pso)).is_some_and(|(r, _)| records.contains(r));
    if out {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if n.is_power_of_two() {
            crate::xlog_warn!("gpu work: XCL_SKIP_RECORDS – {n} dispatches left out (the last: {})", named(pso));
        }
    }
    out
}

fn named(pso: usize) -> String {
    match NAMES.lock().unwrap().as_ref().and_then(|m| m.get(&pso).copied()) {
        Some((record, dxil)) => format!("record {record:016x} ({dxil} B DXIL)"),
        None => match GRAPHICS.lock().unwrap().as_ref().and_then(|m| m.get(&pso).copied()) {
            Some((vs, ps)) => format!("graphics pipeline {pso:#x} (VS {vs:016x}, PS {ps:016x})"),
            None => format!("pipeline {pso:#x}"),
        },
    }
}

const KINDS: [&str; 5] = ["Dispatch", "ExecuteIndirect (compute)", "ExecuteIndirect (graphics)", "DrawInstanced", "DrawIndexedInstanced"];

/// DRED's operation code (D3D12_AUTO_BREADCRUMB_OP) of an entry's kind.
fn dred_code(kind: u8) -> i32 {
    [6, 5, 5, 3, 4][kind.min(4) as usize]
}

/// The device is lost and DRED says the GPU did `done` of the operations `history` (DRED's
/// codes) of `list`: the draws and dispatches recorded on that list around where it stopped.
/// The list's last operations of those kinds are matched to DRED's by counting – the line
/// shows both, so a mismatch is seen.
pub fn dump_list(list: usize, done: usize, history: &[i32]) {
    let drawing = |c: i32| (3..=6).contains(&c);
    let total = history.iter().filter(|&&c| drawing(c)).count();
    let before = history[..done.min(history.len())].iter().filter(|&&c| drawing(c)).count();
    let mine: Vec<Entry> = RING.lock().unwrap().iter().filter(|e| e.list == list).copied().collect();
    if mine.len() < total || total == 0 {
        crate::xlog_error!("gpu work: list {list:#x}: {} draws / dispatches recorded, DRED has {total} – not matched", mine.len());
        return;
    }
    let session = &mine[mine.len() - total..];
    let codes: Vec<i32> = history.iter().copied().filter(|&c| drawing(c)).collect();
    let agree = session.iter().zip(codes.iter()).filter(|(e, c)| dred_code(e.kind) == **c).count();
    crate::xlog_error!("gpu work: list {list:#x} stopped at its draw / dispatch #{before} of {total} ({agree} of {total} kinds agree with DRED's)");
    for k in before.saturating_sub(6)..(before + 3).min(total) {
        let e = &session[k];
        crate::xlog_error!(
            "gpu work:   {}#{k} {} {:?} of {} (DRED: {})",
            if k == before { ">" } else if k + 1 == before { "*" } else { " " },
            KINDS[e.kind.min(4) as usize],
            e.args,
            named(e.pso),
            codes[k]
        );
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
        let groups = match e.kind {
            0 => e.args.iter().map(|&a| a as u64).product(),
            3 | 4 => e.args[0] as u64 * e.args[1] as u64,
            _ => e.args[0] as u64,
        };
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
    let kind = |k: u8| KINDS[k.min(4) as usize];
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
