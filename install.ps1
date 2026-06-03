# mpm 一鍵安裝（Windows PowerShell 5.1+）
#
#   irm https://github.com/coseto6125/markdown-project-manager/releases/latest/download/install.ps1 | iex
#   irm https://github.com/coseto6125/markdown-project-manager/releases/download/v0.1.0/install.ps1 | iex
#
# 環境變數：
#   $env:MPM_VERSION        指定版本（不含 v）。預設 latest。
#   $env:MPM_INSTALL_DIR    安裝目錄。預設 $env:LOCALAPPDATA\Programs\mpm。
#   $env:MPM_NO_VERIFY = 1  跳過 SHA256 驗證（不建議）。
#   $env:MPM_FORCE_CARGO=1  跳過 release binary，強制走 `cargo install --git`。
#
# 全程使用 GitHub Release 的匿名公開資產，無需任何帳號 / token。
# 當前架構沒有 prebuilt（或尚無 Release）時，自動 fallback 到
# `cargo install --git`（需要 cargo / rustup）。

$ErrorActionPreference = 'Stop'

$repo = 'coseto6125/markdown-project-manager'
$bin  = 'mpm'
$version = if ($env:MPM_VERSION) { $env:MPM_VERSION } else { 'latest' }

if (-not $env:MPM_INSTALL_DIR) {
    $installDir = Join-Path $env:LOCALAPPDATA "Programs\mpm"
} else {
    $installDir = $env:MPM_INSTALL_DIR
}

function Invoke-CargoFallback([string]$reason) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "error: $reason" -ForegroundColor Red
        Write-Host "       and ``cargo`` not found in PATH - install Rust from https://rustup.rs," -ForegroundColor Red
        Write-Host "       then re-run this script (or wait for a prebuilt release)." -ForegroundColor Red
        exit 1
    }
    Write-Host "==> $reason"
    Write-Host "==> Falling back to ``cargo install --git`` (source build, may take a few minutes)"

    $buildRoot = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
    try {
        $cargoArgs = @('install', '--root', $buildRoot, '--git', "https://github.com/$repo", '--bin', $bin, '--locked')
        if ($script:version -ne 'latest') {
            $cargoArgs += @('--tag', "v$($script:version.TrimStart('v'))")
        }
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        New-Item -ItemType Directory -Force -Path $script:installDir | Out-Null
        Copy-Item "$buildRoot\bin\$bin.exe" "$script:installDir\$bin.exe" -Force
    } finally {
        if (Test-Path $buildRoot) { Remove-Item -Recurse -Force $buildRoot }
    }
    Write-Host ""
    Write-Host "Installed $bin via cargo -> $script:installDir\$bin.exe"
    exit 0
}

if ($env:MPM_FORCE_CARGO -eq '1') {
    Invoke-CargoFallback 'MPM_FORCE_CARGO=1 set'
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($arch) {
    'X64'   { $target = 'x86_64-pc-windows-msvc' }
    default {
        Invoke-CargoFallback "unsupported prebuilt architecture: $arch (only x86_64-pc-windows-msvc has prebuilt binaries)"
    }
}

if ($version -eq 'latest') {
    $tag = $null
    try {
        $latest = Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$repo/releases/latest"
        $tag = $latest.tag_name
    } catch {
        try {
            $resp = Invoke-WebRequest -UseBasicParsing -MaximumRedirection 0 -Uri "https://github.com/$repo/releases/latest"
            $loc = $resp.Headers.Location
        } catch {
            $response = $_.Exception.Response
            if ($response) {
                $loc = $response.Headers.Location
            }
        }
        if (-not $loc) {
            try {
                $latestPage = Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/$repo/releases/latest"
                if ($latestPage.BaseResponse.ResponseUri) {
                    $loc = $latestPage.BaseResponse.ResponseUri.AbsoluteUri
                } elseif ($latestPage.BaseResponse.RequestMessage) {
                    $loc = $latestPage.BaseResponse.RequestMessage.RequestUri.AbsoluteUri
                }
            } catch {
                # 落到下方 fallback
            }
        }
        if ($loc -match '/tag/([^/?#]+)') {
            $tag = $Matches[1]
        }
    }
    if (-not $tag) {
        Invoke-CargoFallback "no published GitHub Release yet for $repo"
    }
} else {
    $tag = "v$($version.TrimStart('v'))"
}
$ver = $tag.TrimStart('v')

try {
    $running = Get-Process -Name $bin -ErrorAction SilentlyContinue
    if ($running) {
        Write-Host "==> Found running $bin process. Attempting to stop..." -ForegroundColor Yellow
        $running | Stop-Process -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    }
} catch {
    Write-Host "warning: could not stop running $bin process automatically." -ForegroundColor Gray
}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$name    = "$bin-$tag-$target"
$archive = "$name.zip"
$url     = "https://github.com/$repo/releases/download/$tag/$archive"
$shaUrl  = "$url.sha256"

$tmp = Join-Path $env:TEMP "mpm-install-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
    Write-Host "==> Downloading $bin $ver ($target)"
    Write-Host "    $url"
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile (Join-Path $tmp $archive)
    } catch {
        Invoke-CargoFallback "release asset for $target not found (tag $tag)"
    }

    if ($env:MPM_NO_VERIFY -ne '1') {
        Invoke-WebRequest -UseBasicParsing -Uri $shaUrl -OutFile (Join-Path $tmp "$archive.sha256")
        Write-Host "==> Verifying SHA256"
        $expected = (Get-Content (Join-Path $tmp "$archive.sha256") -Raw).Trim().Split()[0].ToLower()
        $actual   = (Get-FileHash (Join-Path $tmp $archive) -Algorithm SHA256).Hash.ToLower()
        if ($expected -ne $actual) {
            Write-Error "SHA256 mismatch: expected $expected, got $actual"
            exit 1
        }
    }

    Expand-Archive -Force -Path (Join-Path $tmp $archive) -DestinationPath $tmp
    Copy-Item -Force (Join-Path $tmp "$name\$bin.exe") (Join-Path $installDir "$bin.exe")
} finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
}

Write-Host ""
Write-Host "Installed $bin $ver -> $installDir\$bin.exe"
Write-Host ""

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $userPath -or -not ($userPath -split ';' | Where-Object { $_ -ieq $installDir })) {
    $newUserPath = if ($userPath) { "$installDir;$userPath" } else { $installDir }
    [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
    $env:Path = "$installDir;$env:Path"
    Write-Host "  Added $installDir to user PATH."
    Write-Host "  It is available in this PowerShell session; restart other shells to pick it up."
    Write-Host ""
}
