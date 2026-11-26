# Notice

Crimson is a modified version of [VIctory](https://github.com/Marvisak/VIctory)
(GPL-3.0-or-later), copyright its authors. Crimson is distributed under the same
license; see [LICENSE](LICENSE).

## Base

Upstream VIctory `main` at commit `63bcca4` ("update readme", 2026-11-26).
Later upstream changes are merged in; `git log` shows which.

## What Crimson changed, and when

All changes by WarmMilkCodes, 2026-11-26, unless noted.

| Files | Change |
|---|---|
| `crates/victory-gpu/src/resource.rs`, `crates/victory-gpu/src/descriptor/host.rs` | Texture format lists only where the host takes them (`VICTORY_FORMAT_LISTS`; off before Windows 11 24H2); R9G9B9E5 views of textures in other formats left null |
| `crates/victory-gpu/src/resource.rs`, `crates/victory-gpu/src/descriptor/host.rs`, `crates/victory-gpu/src/host.rs`, `crates/victory-gpu/src/list.rs`, `crates/victory-gpu/src/pipeline/host.rs` | R9G9B9E5 textures the title renders or writes to are R11G11B10_FLOAT on the host; their views, clear values and pipelines follow; copies between the two are dropped |
| `crates/victory-gpu/src/resource.rs` | `VICTORY_CARD_MARKS`: the share of the card's budget at which host textures are given up; each such pass is logged |
| `crates/victory-gpu/src/host.rs` | The window's title reads `Crimson | time since the window opened | fps, ms/frame, draws/frame` |
| `apps/victory/Cargo.toml`, `apps/victory/src/main.rs`, `apps/victory/tests/*.rs` | The program is `crimson.exe` |
| `xtask/src/policy.rs` | `.svg` files (hand-written vector graphics for the README) are allowed |
| `crates/victory-gpu/src/host.rs` | State preludes come from one pool of the host's instead of one per list (an AMD driver's 2 GiB of command memory filled, and the device was removed); the window's title also shows the time in the world; crash reports name registers that point into memory the layer wrote lately |
| `crates/victory-gpu/src/pipeline.rs`, `crates/victory-gpu/src/draw.rs` | The console's ray-tracing structure builder is off unless `VICTORY_BVH_BUILDER=1`; `VICTORY_INDIRECT_UNBOUNDED` (a measurement) |
| `crates/victory-shader/src/emit/bitcode/opt.rs` (new), `crates/victory-shader/src/emit/bitcode.rs`, `crates/victory-shader/src/emit/mod.rs`, `crates/victory-shader/src/emit/shader.rs`, `crates/victory-gpu/src/shader_cache.rs`, `crates/victory-gpu/src/pipeline/library.rs` | Clean-up passes over each translated shader before it is written (`VICTORY_SHADER_OPT`; `0` turns them off). `opt.rs` is taken whole from [VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX) (GPL-3.0-or-later; its passes after [fxceless/victory-fork](https://github.com/fxceless/victory-fork)), without its tests |
| `README.md`, `NOTICE.md`, `assets/readme/*.svg`, `.gitattributes`, `.gitignore` | Crimson's README, this notice and its graphics |

## Earlier history

Until 2026-11-26 Crimson was a fork of [XCL](https://github.com/xcl-dev/xcl)
(GPL-2.0-only). That work is kept on the `xcl-legacy` branch and is not part of
this code base.
