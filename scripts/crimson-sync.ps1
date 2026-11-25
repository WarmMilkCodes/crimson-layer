# Crimson: sync with upstream XCL – a deliberate step, not every commit.
#   .\scripts\crimson-sync.ps1            merge XCL's main and pin it in UPSTREAM
#   .\scripts\crimson-sync.ps1 -Commit X  merge a chosen XCL commit instead
# Stops on conflicts; after resolving them, run it again with -Continue.
param([string]$Commit = "", [switch]$Continue)
$ErrorActionPreference = "Stop"
Set-Location (git rev-parse --show-toplevel)

# README and its graphics are Crimson's own (.gitattributes merge=ours needs this driver)
git config merge.ours.driver true

$upstream = (git remote -v | Select-String "xcl-dev/xcl|ivall/xcl" | Select-Object -First 1)
if (-not $upstream) { git remote add upstream https://github.com/xcl-dev/xcl.git; $remote = "upstream" }
else { $remote = ($upstream.Line -split "\s+")[0] }

if (-not $Continue) {
    if (git status --porcelain --untracked-files=no) { throw "Commit or stash your changes first." }
    git fetch $remote main
    $target = if ($Commit) { $Commit } else { (git rev-parse "$remote/main").Trim() }
    $old = (Select-String -Path UPSTREAM -Pattern '^commit').Line -replace '.*"(.*)".*', '$1'
    $count = (git rev-list --count "$old..$target").Trim()
    Write-Host "XCL $($old.Substring(0,12)) -> $($target.Substring(0,12)): $count commits"
    git merge --no-ff --no-commit $target
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nConflicts – resolve these, then run: .\scripts\crimson-sync.ps1 -Continue" -ForegroundColor Yellow
        git diff --name-only --diff-filter=U
        Set-Content .git\crimson-sync-target $target
        exit 1
    }
} else {
    $target = (Get-Content .git\crimson-sync-target).Trim()
    $left = git diff --name-only --diff-filter=U
    if ($left) { Write-Host "Still unmerged:"; $left; exit 1 }
}

# no conflict markers may be committed (the README went out with them once)
$markers = git grep -l -E "^(<<<<<<<|>>>>>>>) " -- . 2>$null
if ($markers) { Write-Host "Conflict markers left in:" -ForegroundColor Red; $markers; exit 1 }

$date = (git log -1 --format=%ad --date=short $target).Trim()
(Get-Content UPSTREAM) -replace '^commit = ".*"', "commit = `"$target`"" -replace '^date = ".*"', "date = `"$date`"" | Set-Content UPSTREAM -Encoding ascii
git add UPSTREAM

cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed – fix it before committing the sync." -ForegroundColor Red; exit 1 }
$stamp = (& .\target\release\crimson.exe version)
git commit -m "Sync with XCL $($target.Substring(0,12)) ($date)" -m $stamp
Remove-Item .git\crimson-sync-target -ErrorAction SilentlyContinue
Write-Host "`nSynced. $stamp"
Write-Host "If the shader translator changed, tell players a shader rebuild is due."
