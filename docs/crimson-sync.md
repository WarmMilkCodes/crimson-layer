# Syncing Crimson with XCL

Crimson is pinned to one XCL commit, recorded in [`UPSTREAM`](../UPSTREAM). It moves only by a
deliberate sync, so players don't rebuild shaders for every upstream commit.

```powershell
.\scripts\crimson-sync.ps1
```

The script fetches XCL's `main`, merges it, refuses to commit leftover conflict markers, updates
`UPSTREAM`, builds, and commits with the new version line. On conflicts it stops; resolve
them and run `.\scripts\crimson-sync.ps1 -Continue`. `-Commit <hash>` syncs to a chosen XCL
commit instead of the latest.

`README.md` and `assets/readme/` always keep Crimson's version (`.gitattributes`, `merge=ours`).

## Does a sync need a shader rebuild?

`crimson version` prints the shader translator stamp. If it changed, the sync changed how
shaders are translated: players' caches are re-translated on the next precompile or launch
(the driver's cache still covers every shader whose output came out the same). Say so in the
release notes. `crimson run` also warns players when their cache is from another build.

## Releases

Tag `crimson-<version>` (the version is `brand::VERSION`), and name the XCL commit and
whether a shader rebuild is due.
