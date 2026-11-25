//! Crimson: the name users see, the version, the XCL commit it is pinned to, and the shader
//! translator stamp that tells whether a build needs the shader caches rebuilt.
//!
//! Only what users see is renamed: crate and module names and the `XCL_*` settings stay as
//! upstream XCL has them, so syncing with XCL does not conflict across the tree.

use std::path::{Path, PathBuf};

/// The product name in titles, dialogs and messages.
pub const NAME: &str = "Crimson";
/// Crimson's own version (releases are tagged `crimson-<VERSION>`).
pub const VERSION: &str = "0.1.0";
/// `UPSTREAM` at the repository root: the XCL commit this build is based on.
const UPSTREAM: &str = include_str!("../../../UPSTREAM");

/// The pinned XCL commit (short form), or "unknown".
pub fn upstream_commit() -> &'static str {
    UPSTREAM
        .lines()
        .find_map(|l| l.trim().strip_prefix("commit").map(|r| r.trim_start_matches([' ', '=']).trim().trim_matches('"')))
        .map(|c| &c[..c.len().min(12)])
        .unwrap_or("unknown")
}

/// The pinned XCL commit's date, or "".
pub fn upstream_date() -> &'static str {
    UPSTREAM.lines().find_map(|l| l.trim().strip_prefix("date").map(|r| r.trim_start_matches([' ', '=']).trim().trim_matches('"'))).unwrap_or("")
}

/// What decides how console shaders are translated: the translator sources (both paths) and
/// the direct translator's version. Builds with the same stamp produce the same shaders, so
/// their caches carry over; a different stamp means shaders are translated again (the
/// driver's cache still serves every shader whose output came out the same).
pub fn translator_stamp() -> String {
    format!("{}-{}-{}", env!("XCL_TRANSLATOR_SOURCE_ID"), env!("XCL_DIRECT_DXIL_SOURCE_ID"), crate::gpu::dxil::kernel::TRANSLATOR_VERSION)
}

/// One line for `--version` and the log.
pub fn version_line() -> String {
    format!("{NAME} {VERSION} (XCL {} {}; shader translator {})", upstream_commit(), upstream_date(), translator_stamp())
}

/// A folder under %LOCALAPPDATA%: Crimson's (`crimson`), unless only XCL's older one
/// (`legacy`) exists – so caches and saves made before the rename are found.
pub fn local_dir(crimson: &str, legacy: &str) -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
    let new = base.join(crimson);
    let old = base.join(legacy);
    if !new.exists() && old.exists() {
        old
    } else {
        new
    }
}

const STAMP_FILE: &str = "crimson-shaders.stamp";

/// Whether the shader cache in `cache_dir` was built by a translator like this build's.
pub enum ShaderCache {
    /// Built by this translator.
    Current,
    /// Built by another translator: shaders are translated again.
    Stale { built_by: String },
    /// No precompile recorded.
    Unknown,
}

pub fn shader_cache_state(cache_dir: &Path) -> ShaderCache {
    match std::fs::read_to_string(cache_dir.join(STAMP_FILE)) {
        Ok(s) if s.trim() == translator_stamp() => ShaderCache::Current,
        Ok(s) => ShaderCache::Stale { built_by: s.trim().to_string() },
        Err(_) => ShaderCache::Unknown,
    }
}

/// Records that `cache_dir` now holds this translator's shaders (after a precompile).
pub fn mark_shader_cache(cache_dir: &Path) {
    let _ = std::fs::create_dir_all(cache_dir);
    let _ = std::fs::write(cache_dir.join(STAMP_FILE), translator_stamp());
}

/// The note printed when a game starts with a cache built by another translator, or none.
pub fn shader_cache_note(cache_dir: &Path) -> Option<String> {
    match shader_cache_state(cache_dir) {
        ShaderCache::Current => None,
        ShaderCache::Stale { .. } => Some(format!(
            "{NAME}: this build translates shaders differently from the one that built your shader cache.\n\
             The first launch compiles shaders again (slow loading, stutter). To do it first:\n  \
             crimson shaders \"<game folder>\\shaders.rpf\" --precompile --only cs --pipelines"
        )),
        ShaderCache::Unknown => Some(format!(
            "{NAME}: no shader precompile recorded for this build. The first launch compiles shaders as the game needs them.\n\
             To do it first:  crimson shaders \"<game folder>\\shaders.rpf\" --precompile --only cs --pipelines"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pin_is_read() {
        assert_eq!(upstream_commit().len(), 12);
        assert!(!upstream_date().is_empty());
    }

    #[test]
    fn a_cache_is_current_only_with_this_stamp() {
        let d = std::env::temp_dir().join(format!("crimson-stamp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert!(matches!(shader_cache_state(&d), ShaderCache::Unknown));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(STAMP_FILE), "other").unwrap();
        assert!(matches!(shader_cache_state(&d), ShaderCache::Stale { .. }));
        mark_shader_cache(&d);
        assert!(matches!(shader_cache_state(&d), ShaderCache::Current));
        assert!(shader_cache_note(&d).is_none());
        let _ = std::fs::remove_dir_all(&d);
    }
}
