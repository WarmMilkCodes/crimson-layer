<p align="center">
  <img src="assets/readme/banner.svg" width="100%" alt="Crimson – Xbox Series X|S games you own, on a mid-range PC, tuned for Series S–level performance. Built on XCL; AMD RDNA2; 8 GB VRAM; no CPU emulation.">
</p>

<p align="center">
  <a href="https://discord.gg/bbdTtcVKnR"><img alt="Discord: join the server" src="https://img.shields.io/badge/Discord-join%20the%20server-e0183a?style=for-the-badge&logo=discord&logoColor=white"></a>
  <a href="https://github.com/xcl-dev/xcl"><img alt="Built on XCL" src="https://img.shields.io/badge/built%20on-XCL-e0183a?style=for-the-badge"></a>
  <a href="LICENSE"><img alt="License: GPL-2.0-only" src="https://img.shields.io/badge/license-GPL--2.0--only-e0183a?style=for-the-badge"></a>
  <img alt="Status: experimental" src="https://img.shields.io/badge/status-experimental-ff8a3d?style=for-the-badge">
  <img alt="Windows 10 / 11 x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20x64-8a1c2c?style=for-the-badge">
  <img alt="AMD RDNA2" src="https://img.shields.io/badge/GPU-AMD%20RDNA2-8a1c2c?style=for-the-badge">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable--msvc-8a1c2c?style=for-the-badge">
</p>

<p align="center">
  <b><a href="#status">Status</a></b> ·
  <b><a href="#highlights">Highlights</a></b> ·
  <b><a href="#how-it-works">How it works</a></b> ·
  <b><a href="#quick-start">Quick start</a></b> ·
  <b><a href="#recommended-settings">Settings</a></b> ·
  <b><a href="#what-crimson-changes">What Crimson changes</a></b> ·
  <b><a href="#known-issues">Known issues</a></b> ·
  <b><a href="#upstream">Upstream</a></b> ·
  <b><a href="https://discord.gg/bbdTtcVKnR">Discord</a></b>
</p>

# Crimson

Crimson is a performance-focused fork of [XCL](https://github.com/xcl-dev/xcl), the Xbox Series X|S → Windows compatibility layer. XCL does the hard part: the game's x64 code runs natively, console system libraries are reimplemented, and the console's RDNA2 shaders are recompiled to DXIL. Crimson builds on that with one goal: **Series X|S games you own, running at roughly Series S–level performance on comparable PC hardware** – an 8 GB RDNA2 card, a 6-core CPU and 32 GB of RAM.

> [!IMPORTANT]
> Crimson works only on unpacked game files you already own. Like XCL, it does not download, decrypt or unpack game packages and contains nothing for circumventing DRM. The repository holds no console SDK code, keys or game data.

> [!WARNING]
> Crimson is early and experimental. There are no releases yet, nothing is playable yet, and first launches are long. Progress is posted as it lands – follow along on [Discord](https://discord.gg/bbdTtcVKnR).

<p align="center"><img src="assets/readme/divider.svg" width="100%" alt=""></p>

## Status

<p align="center">
  <img src="assets/readme/status.svg" width="100%" alt="A newer retail game on an RX 6600 XT: boots, plays its intro and menus at a steady 30 fps and loads into the opening of its story with sound; the 3D world does not render yet and the GPU is lost about 7 minutes in.">
</p>

| Stage | State | Notes |
|---|---|---|
| Boots and plays its intro | ✅ | steady 30 fps |
| Menus and account selection | ✅ | keyboard works as a gamepad |
| Loads into the story | ✅ | audio and music play |
| Renders the 3D world | 🚧 | black screen; the GPU is lost a few minutes in (being worked on) |
| Playable | ❌ | – |

<sub>Test system: AMD Radeon RX 6600 XT 8 GB · Ryzen 5 5500 · 32 GB DDR4 · Windows 11.</sub>

## Highlights

<table>
  <tr>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-gauge.svg" width="56" alt=""><br>
      <p align="left"><b>Series S as the target</b> – tuned and tested on the PC hardware closest to a Series S, not a top-end GPU.</p>
    </td>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-memory.svg" width="56" alt=""><br>
      <p align="left"><b>Memory gate</b> – shader compiles wait for free RAM instead of piling on, so a precompile or a game doesn't crash because your browser is open.</p>
    </td>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-timer.svg" width="56" alt=""><br>
      <p align="left"><b>Bounded pipeline waits</b> – the game never waits long enough on a compiling shader to trip its own hang watchdog.</p>
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-ray.svg" width="56" alt=""><br>
      <p align="left"><b>Ray-tracing shader routing</b> – ray-tracing shaders that hung AMD's driver are built through a path the driver survives.</p>
    </td>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-shield.svg" width="56" alt=""><br>
      <p align="left"><b>Driver-failure diagnostics</b> – pipeline tracing, skip and filter switches that pin a GPU hang on one shader in seconds instead of hours.</p>
    </td>
    <td width="33%" valign="top" align="center">
      <img src="assets/readme/icon-upstream.svg" width="56" alt=""><br>
      <p align="left"><b>Tracks upstream XCL</b> – synced with XCL's main branch; general fixes are offered back upstream.</p>
    </td>
  </tr>
</table>

## How it works

<p align="center">
  <img src="assets/readme/how.svg" width="100%" alt="How Crimson works: the game's x64 code runs natively on the CPU; XCL provides the console libraries and recompiles RDNA2 shaders to DXIL; Crimson adds a tuning layer – memory gate, bounded pipeline waits, ray-tracing routing, 8 GB defaults and diagnostics – in front of Direct3D 12 and the GPU.">
</p>

Crimson is XCL plus a set of changes focused on mid-range hardware. Everything XCL documents still applies; see [XCL's README](https://github.com/xcl-dev/xcl#readme) for the design and the full feature list.

<p align="center"><img src="assets/readme/divider.svg" width="100%" alt=""></p>

## Quick start

**Requirements:** Windows 10 / 11 x64, Rust (`stable-x86_64-pc-windows-msvc`), the Windows SDK (DXC 1.8 or newer), a D3D12 GPU with Shader Model 6.6 and mesh shaders. Same as [XCL](https://github.com/xcl-dev/xcl#building-and-usage).

1. **Build**

   ```powershell
   git clone https://github.com/WarmMilkCodes/crimson-layer
   cd crimson-layer
   cargo build --release --workspace
   ```

2. **Precompile the shaders** (first time only; can take hours for a large game, and everything built is cached)

   ```powershell
   .\target\release\xcl.exe shaders "D:\Game\shaders.rpf" --precompile --only cs --pipelines
   ```

3. **Run the game** – the folder that holds `MicrosoftGame.config`:

   ```powershell
   .\target\release\xcl.exe run "D:\Game"
   ```

The game folder is never modified. A gamepad is recommended; keyboard and mouse also work as a gamepad.

## Recommended settings

For an 8 GB RDNA2 card. Set once; they apply to new PowerShell windows:

```powershell
setx XCL_PSO_WAIT_MS 4000      # bound how long the game waits for a pipeline still compiling
setx XCL_PSO_PRELOAD 0         # the used-pipeline preload leaks memory on this setup
setx XCL_DIRECT_DXIL 1         # faster compute shader translation (ray tracing goes the safe way)
setx XCL_HEAP_WINDOWS 1        # smaller, faster shaders
setx XCL_NETWORK 0             # offline, including crash-report uploads
setx XCL_WINDOW_POS center     # open the game window on the main monitor
```

Optional: put the shader caches on a roomy drive with `setx XCL_DATA_ROOT D:\xcl-data`.

<details>
<summary><b>Crimson's own switches</b></summary>

| Variable | Default | What it does |
|---|---|---|
| `XCL_MEM_RESERVE_MB` | `6144` | RAM the memory gate keeps free for everything else |
| `XCL_MEM_BIG_KB` | `32` | shader size counted as "big" (needs extra free RAM to start) |
| `XCL_MEM_GATE` | on | `0` turns the memory gate off |
| `XCL_PSO_WAIT_COOLDOWN` | on | `0` turns off the short-wait cooldown after a late pipeline |
| `XCL_DIRECT_DXIL_BVH` | off | `1` sends ray-tracing shaders through the direct translator anyway |
| `XCL_PIPELINE_TRACE` | off | precompile: name each pipeline before the driver gets it |
| `XCL_PIPELINE_SKIP` | – | precompile: skip pipelines whose name contains any of these (comma-separated) |
| `XCL_PIPELINE_ONLY` | – | precompile: build only pipelines whose name contains any of these |
| `XCL_DRED` | off | `breadcrumbs` or `pagefault`: D3D12 crash diagnostics |

</details>

<p align="center"><img src="assets/readme/divider.svg" width="100%" alt=""></p>

## What Crimson changes

<details>
<summary><b>Stability</b> – hangs, watchdogs, out-of-memory</summary>

| Change | Why |
|---|---|
| **Bounded pipeline waits with a cooldown** | Back-to-back multi-second waits for compiling pipelines tripped the game's own hang watchdog. After any late pipeline, waits are cut short for a while. |
| **Memory gate** | Several huge shaders compiling at once, plus a browser, ran a 32 GB machine out of memory. Compiles now start only when there's room, and the precompile runs at below-normal priority so the desktop stays responsive. |
| **Ray-tracing shaders through the HLSL path** | The direct DXIL translator's form of a retail game's ray-tracing shaders hung AMD's driver on load (reproducible in seconds). The HLSL path's form of the same shaders builds fine. |

</details>

<details>
<summary><b>Diagnostics</b> – finding what breaks the driver</summary>

| Change | Why |
|---|---|
| **Refusal details** | A refused pipeline now reports the error, the device state and free memory, instead of a bare count. |
| **Pipeline trace / skip / only** | Single-threaded tracing names the pipeline that takes the GPU down; skip and only lists confirm it in a one-minute test. |
| **DRED modes** | Breadcrumbs and page-fault reporting can be switched on separately. |

</details>

<details>
<summary><b>Planned</b></summary>

- Make passes that are skipped while a pipeline compiles harmless, so they can't hang the GPU.
- Compose the console's two display planes, so Series S mode shows its menus.
- Default to Series S settings on 8 GB cards.
- Track down in-game memory growth.
- Ship the recommended settings as defaults.

</details>

## Known issues

- **The 3D world doesn't render yet**, and the GPU is lost a few minutes after the story starts.
- **Series S mode** (`device_type = "lockhart"` in the game's `xcl.toml`) shows a black screen after the intro video – the menus are drawn but not shown yet.
- **First launch is long.** Precompile the shaders first.
- **PCIe power saving** made AMD's driver lose the device during long compiles on the test system. If you see `DEVICE_HUNG` / `0x887A0006` while precompiling:

  ```powershell
  powercfg /setacvalueindex SCHEME_CURRENT SUB_PCIEXPRESS ASPM 0
  powercfg /setactive SCHEME_CURRENT
  ```

## Community

Questions, test results and progress updates: **[join the Crimson Discord](https://discord.gg/bbdTtcVKnR)**. Bugs and code changes go in this repository's issues and pull requests.

## Upstream

Crimson exists because of [XCL](https://github.com/xcl-dev/xcl) and its contributors. It stays in sync with XCL's main branch, and fixes that help everyone are offered back upstream. Problems that also happen on plain XCL belong in [XCL's issues](https://github.com/xcl-dev/xcl/issues); Crimson-specific ones belong here.

## License

GPL-2.0-only, the same as XCL. See [LICENSE](LICENSE) and [THIRD_PARTY.md](THIRD_PARTY.md).
