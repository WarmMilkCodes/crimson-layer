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
| `README.md`, `NOTICE.md`, `assets/readme/*.svg`, `.gitattributes`, `.gitignore` | Crimson's README, this notice and its graphics |

## Earlier history

Until 2026-11-26 Crimson was a fork of [XCL](https://github.com/xcl-dev/xcl)
(GPL-2.0-only). That work is kept on the `xcl-legacy` branch and is not part of
this code base.
