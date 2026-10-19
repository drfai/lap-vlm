# ============================================================
# FORK: 在已激活 MSVC 环境的 shell 里构建 Lap fork。
#
# 为什么需要这个脚本:
#   cargo 能找到，但 MSVC 的 link.exe / cl.exe 不在 PATH。
#   Rust 的 cc crate 会自动找 vswhere, 所以 cargo check 能过,
#   但 pnpm tauri build 会调 rc.exe / mt.exe 等 Windows SDK 工具,
#   必须显式激活 vcvars64。
#
# 用法:
#   pwsh -File build-fork.ps1            # cargo check + test + tauri build
#   pwsh -File build-fork.ps1 -CheckOnly # 只跑 cargo check + test (快)
#   pwsh -File build-fork.ps1 -TestOnly  # 只跑 cargo test t_llm
# ============================================================
param(
    [switch]$CheckOnly,
    [switch]$TestOnly
)

$ErrorActionPreference = 'Continue'

$VSWHERE = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$VCVARS = $null

if (Test-Path $VSWHERE) {
    $vsPath = & $VSWHERE -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vsPath) {
        $candidate = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
        if (Test-Path $candidate) { $VCVARS = $candidate }
    }
}
if (-not $VCVARS) {
    foreach ($p in @("C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat", "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat")) {
        if (Test-Path $p) { $VCVARS = $p; break }
    }
}

if (-not $VCVARS) {
    Write-Error "找不到 vcvars64.bat。请安装 VS 2022 的 '使用 C++ 的桌面开发' 工作负载。"
    exit 1
}

Write-Host "=== vcvars64: $VCVARS ===" -ForegroundColor Cyan

# 关键: 用 cmd /c 激活环境后, 在同一个 cmd 进程里执行目标命令。
# 不能分成两个 pwsh 调用 —— 环境变量不会跨进程传递。
function Invoke-InMsvcEnv {
    param([string]$Command, [string]$WorkDir)

    $full = "`"$VCVARS`" >nul 2>&1 && cd /d `"$WorkDir`" && $Command"
    Write-Host "`n>>> $Command" -ForegroundColor Yellow
    cmd /c $full
    return $LASTEXITCODE
}

$SRC_TAURI = Join-Path $PSScriptRoot "src-tauri"
$SRC_VITE = Join-Path $PSScriptRoot "src-vite"

# 确认 cargo 可用
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoBin = "$env:USERPROFILE\.cargo\bin\cargo.exe"
    if (Test-Path $cargoBin) {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
        Write-Host "已将 $env:USERPROFILE\.cargo\bin 加入本次会话 PATH" -ForegroundColor Cyan
    } else {
        Write-Error "cargo 未安装或不在 PATH。请先运行 rustup-init。"
        exit 1
    }
}

Write-Host "`n=== 工具链版本 ===" -ForegroundColor Cyan
cargo --version
rustc --version

$failed = $false

if ($TestOnly) {
    $rc = Invoke-InMsvcEnv "cargo test t_llm -- --nocapture" $SRC_TAURI
    if ($rc -ne 0) { $failed = $true }
}
elseif ($CheckOnly) {
    $rc = Invoke-InMsvcEnv "cargo check" $SRC_TAURI
    if ($rc -ne 0) { $failed = $true }
    if (-not $failed) {
        $rc = Invoke-InMsvcEnv "cargo test t_llm" $SRC_TAURI
        if ($rc -ne 0) { $failed = $true }
    }
}
else {
    # 1. Rust 侧检查
    $rc = Invoke-InMsvcEnv "cargo check" $SRC_TAURI
    if ($rc -ne 0) {
        Write-Host "`n!!! cargo check 失败, 中止构建" -ForegroundColor Red
        $failed = $true
    }

    # 2. Rust 单元测试
    if (-not $failed) {
        $rc = Invoke-InMsvcEnv "cargo test t_llm" $SRC_TAURI
        if ($rc -ne 0) { $failed = $true }
    }

    # 3. 前端依赖
    if (-not $failed) {
        if (-not (Test-Path (Join-Path $SRC_VITE "node_modules"))) {
            Write-Host "`n>>> pnpm install (src-vite)" -ForegroundColor Yellow
            Push-Location $SRC_VITE
            pnpm install 2>&1 | Select-Object -Last 15
            Pop-Location
        }
    }

    # 4. 完整打包
    if (-not $failed) {
        $rc = Invoke-InMsvcEnv "pnpm tauri build" $SRC_TAURI
        if ($rc -ne 0) { $failed = $true }
    }
}

Write-Host ""
if ($failed) {
    Write-Host "=== 构建失败 ===" "Red"
    exit 1
} else {
    Write-Host "=== 构建成功 ===" "Green"
    exit 0
}
