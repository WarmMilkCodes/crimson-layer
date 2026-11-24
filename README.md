# Crimson

**A performance-focused fork of [XCL](https://github.com/xcl-dev/xcl), the Xbox Series X|S → Windows compatibility layer, tuned for AMD RDNA2 GPUs with 8 GB of VRAM.**

Crimson tracks upstream XCL and adds changes aimed at one goal: running Series X|S games you own at roughly Series S–level performance on comparable PC hardware (an RX 6600 XT–class GPU, a 6-core CPU, 32 GB RAM).

> **Status: early and experimental.** No releases yet. Expect crashes, missing effects and long first launches. Progress updates are posted as work lands.

---

## Why a fork?

XCL does the hard part: it runs a console game's x64 code natively, reimplements the console system libraries, and recompiles the console's RDNA2 shaders to DXIL. Testing a demanding retail title on an 8 GB RDNA2 card turned up problems that matter most on mid-range hardware:

| Problem | Where it comes from | Crimson's approach |
|---|---|---|
| Hang watchdog ends the game while loading | The title waits on pipelines that take seconds to compile | Bounded pipeline waits, with a cooldown so waits can't pile up back to back |
| GPU lost ("device removed") during world loading | A pass skipped because its pipeline wasn't ready leaves garbage for the next pass | Make skipped passes harmless (in progress) |
| Very long first launch | Thousands of large translated compute shaders, each slow in the driver's compiler | Build the pipelines a title actually uses first; precompile tooling that survives driver failures |
| Memory climbing past 40 GB | Extra copies of title memory held during streaming | Measuring, then trimming (planned) |
| Shared memory read over PCIe | Title memory the GPU reads lives in system RAM | A layout that fits 8 GB cards (planned) |

General fixes are offered back to upstream XCL. Crimson-specific tuning stays here.

## Tested on

| | |
|---|---|
| GPU | AMD Radeon RX 6600 XT, 8 GB (RDNA2) |
| CPU | AMD Ryzen 5 5500 |
| RAM | 32 GB |
| OS | Windows 11 |

Other GPUs should work wherever upstream XCL works, but they aren't the focus.

## Requirements

The same as upstream XCL: Windows 10/11 x64, Rust (`stable-x86_64-pc-windows-msvc`), the Windows SDK (DXC 1.8 or newer), a D3D12 GPU with Shader Model 6.6 and mesh shaders, and the system XGameRuntime. See [upstream's build notes](https://github.com/xcl-dev/xcl#building-and-usage) for details, including the Agility SDK on Windows 10.

## Building and running

```powershell
git clone https://github.com/WarmMilkCodes/crimson-layer
cd crimson-layer
cargo build --release --workspace
.\target\release\xcl.exe run "D:\Path\To\Your\Unpacked\Game"
```

Crimson works only with **unpacked game files you already own**, exactly like XCL. It does not download, decrypt or unpack game packages, and contains nothing for circumventing DRM.

### Recommended settings for 8 GB RDNA2 cards

Set these once with `setx` (they take effect in new PowerShell windows):

```powershell
setx XCL_PSO_WAIT_MS 4000      # bound how long a title waits for a pipeline still building
setx XCL_PSO_PRELOAD 0         # the used-pipeline preload leaks memory on this setup (being reported upstream)
setx XCL_NETWORK 0             # run titles offline, including their crash-report uploads
setx XCL_WINDOW_POS center     # open the game window on the main monitor
```

### First launch: precompile the shaders

A large title has tens of thousands of shaders. Compiling them ahead of time avoids most stalls in game:

```powershell
$env:XCL_PIPELINE_THREADS = "2"   # driver compiles at once; raise it only if you have RAM to spare
.\target\release\xcl.exe shaders "D:\Path\To\Game\shaders.rpf" --precompile --pipelines
```

This can take hours the first time, and the biggest shaders use a lot of RAM. Everything built is cached and skipped on later runs.

### Known system issue: PCIe power saving

On the test system, Windows' PCIe link power management made AMD's driver lose the device during long shader compiles. If you see `DEVICE_HUNG` / `0x887A0006` while precompiling, turn it off:

```powershell
powercfg /setacvalueindex SCHEME_CURRENT SUB_PCIEXPRESS ASPM 0
powercfg /setactive SCHEME_CURRENT
```

(To undo it, run the same with `ASPM 2`.)

## Staying in sync with upstream

Crimson merges upstream XCL regularly. Note that upstream changes to the shader translator invalidate the shader cache, so expect a fresh precompile after some updates.

## Credits

Crimson is built on **[XCL](https://github.com/xcl-dev/xcl)** and the work of its contributors. All the core work (the loader, the console system libraries, the shader recompiler, DirectStorage, BCPack, textures, ray tracing and more) is theirs. Please report issues that also happen on upstream XCL there.

## License

**GPL-2.0-only**, the same as XCL. See [`LICENSE`](LICENSE) and [`THIRD_PARTY.md`](THIRD_PARTY.md). Crimson's changes are released under the same license.

## Trademarks

Xbox, Xbox Series X|S and related names are trademarks of Microsoft. AMD, Radeon and RDNA are trademarks of Advanced Micro Devices. Crimson is an independent project, not affiliated with or endorsed by Microsoft, AMD, any game publisher, or the XCL project.
