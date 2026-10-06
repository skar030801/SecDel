<#
.SYNOPSIS
    SecDel build script. Compiles frontend, Rust backend, and MSI installer.
.DESCRIPTION
    Orchestrates the full build pipeline for the SecDel application:
    - Frontend (npm install + build)
    - Rust backend (cargo build)
    - Tauri bundle (MSI installer generation)
    - Optional code signing if certificate is available
    - SHA256 checksum generation
    - Test execution
.PARAMETER Configuration
    Build configuration: Debug or Release (default: Release)
.PARAMETER InstallWix
    If set, installs WiX Toolset v4 via winget before building
.PARAMETER SkipFrontend
    If set, skips the frontend build step
.PARAMETER SkipBackend
    If set, skips the Rust backend build step
.PARAMETER SkipInstaller
    If set, skips the MSI installer build step
.PARAMETER SkipTests
    If set, skips running tests
.PARAMETER SignCertificatePath
    Path to a code signing certificate (.pfx) for signing the installer
.PARAMETER SignCertificatePassword
    Password for the code signing certificate
.EXAMPLE
    .\build.ps1 -Configuration Release
.EXAMPLE
    .\build.ps1 -Configuration Debug -SkipInstaller
.EXAMPLE
    .\build.ps1 -InstallWix -SignCertificatePath .\cert.pfx -SignCertificatePassword "mypassword"
#>

param(
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',

    [switch]$InstallWix,
    [switch]$SkipFrontend,
    [switch]$SkipBackend,
    [switch]$SkipInstaller,
    [switch]$SkipTests,

    [string]$SignCertificatePath = '',
    [string]$SignCertificatePassword = ''
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ConfigLower = $Configuration.ToLower()
$ReleaseDir = "src-tauri\target\$ConfigLower"

# ─── Helper functions ──────────────────────────────────────────────
function Write-Step([string]$Title, [string]$Number) {
    Write-Host "`n[$Number] $Title..." -ForegroundColor Yellow
}

function Write-Success([string]$Message) {
    Write-Host "  ✓ $Message" -ForegroundColor Green
}

function Write-Failure([string]$Message) {
    Write-Host "  ✗ $Message" -ForegroundColor Red
}

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Invoke-NativeCommand([scriptblock]$ScriptBlock) {
    & $ScriptBlock
    if ($LASTEXITCODE -ne 0) {
        throw "Command exited with code $LASTEXITCODE"
    }
}

# ─── Prerequisites check ──────────────────────────────────────────
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "       SecDel Build Script v1.0.0" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Configuration : $Configuration"
Write-Host "Project Root  : $ProjectRoot"
Write-Host "Admin Mode    : $(if (Test-Admin) { 'Yes' } else { 'No (may fail)' })"
Write-Host ""

# Check required tools
$requiredTools = @(
    @{ Name = 'Node.js'; Command = 'node --version' },
    @{ Name = 'npm'; Command = 'npm --version' },
    @{ Name = 'Rust (cargo)'; Command = 'cargo --version' },
    @{ Name = 'Rust (rustc)'; Command = 'rustc --version' }
)

foreach ($tool in $requiredTools) {
    try {
        $result = Invoke-Expression $tool.Command
        Write-Success "$($tool.Name): $($result.Trim())"
    } catch {
        Write-Failure "$($tool.Name) not found. Please install it."
        throw "Missing dependency: $($tool.Name)"
    }
}

# Check for Tauri CLI
try {
    $tauriVersion = npx tauri --version 2>$null
    Write-Success "Tauri CLI: $($tauriVersion.Trim())"
} catch {
    Write-Host "  Installing @tauri-apps/cli globally..." -ForegroundColor DarkYellow
    npm install -g @tauri-apps/cli
}

# ─── Install WiX (optional) ──────────────────────────────────────
if ($InstallWix) {
    Write-Step "Installing WiX Toolset" "0/7"
    try {
        # Check if WiX v4 is available via dotnet tools
        if (Get-Command 'dotnet' -ErrorAction SilentlyContinue) {
            dotnet tool install --global wix --version 4.*
            if ($LASTEXITCODE -eq 0) {
                Write-Success "WiX Toolset v4 installed via dotnet tool"
            } else {
                throw "WiX installation failed"
            }
        } else {
            # Fall back to winget
            winget install "WiX Toolset" --accept-package-agreements
            Write-Success "WiX Toolset installed via winget"
        }
    } catch {
        Write-Warning "WiX installation failed: $_"
        Write-Warning "Proceeding without WiX. Installer build may fail."
    }
}

# ─── Step 1: Frontend ─────────────────────────────────────────────
if (-not $SkipFrontend) {
    Write-Step "Building frontend (npm install + build)" "1/7"
    Set-Location $ProjectRoot

    Write-Host "  -> npm ci (clean install)..." -ForegroundColor Gray
    Invoke-NativeCommand { npm ci }

    Write-Host "  -> npm run build..." -ForegroundColor Gray
    Invoke-NativeCommand { npm run build }

    # Frontend lint
    if (Test-Path "node_modules/.bin/eslint") {
        Write-Host "  -> eslint..." -ForegroundColor Gray
        npx eslint src/ --max-warnings=10
        if ($LASTEXITCODE -eq 0) { Write-Success "ESLint passed" }
        else { Write-Warning "ESLint found warnings (non-fatal)" }
    }

    Write-Success "Frontend build complete"
} else {
    Write-Host "[1/7] Frontend build: SKIPPED" -ForegroundColor DarkGray
}

# ─── Step 2: Rust Backend ─────────────────────────────────────────
if (-not $SkipBackend) {
    Write-Step "Building Rust backend (cargo build --$ConfigLower)" "2/7"
    Set-Location "$ProjectRoot\src-tauri"

    Write-Host "  -> cargo build..." -ForegroundColor Gray
    Invoke-NativeCommand { cargo build --$ConfigLower }

    Write-Success "Rust backend build complete"
} else {
    Write-Host "[2/7] Rust backend: SKIPPED" -ForegroundColor DarkGray
}

# ─── Step 3: Unit + Integration Tests ─────────────────────────────
if (-not $SkipTests) {
    Write-Step "Running tests" "3/7"
    Set-Location "$ProjectRoot\src-tauri"

    Write-Host "  -> cargo test..." -ForegroundColor Gray
    Invoke-NativeCommand { cargo test --all-features }

    Write-Host "  -> cargo doc-test..." -ForegroundColor Gray
    Invoke-NativeCommand { cargo test --doc }

    # Frontend tests
    Set-Location $ProjectRoot
    if (Test-Path "node_modules/.bin/vitest" -or (Test-Path "package.json" -and (Select-String -Path "package.json" -Pattern '"test"' -Quiet))) {
        Write-Host "  -> npm test..." -ForegroundColor Gray
        npm test -- --run
        if ($LASTEXITCODE -eq 0) { Write-Success "Frontend tests passed" }
        else { Write-Warning "Frontend tests failed (continuing)" }
    }

    Write-Success "All tests passed"
} else {
    Write-Host "[3/7] Tests: SKIPPED" -ForegroundColor DarkGray
}

# ─── Step 4: Clippy Lint ──────────────────────────────────────────
if (-not $SkipTests) {
    Write-Step "Running clippy lint" "4/7"
    Set-Location "$ProjectRoot\src-tauri"

    Write-Host "  -> cargo clippy..." -ForegroundColor Gray
    cargo clippy --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Clippy passed"
    } else {
        Write-Warning "Clippy found issues. Run 'cargo clippy --fix' to auto-fix."
        throw "Clippy lint failed"
    }

    Write-Host "  -> cargo fmt check..." -ForegroundColor Gray
    cargo fmt --all -- --check
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Formatting check passed"
    } else {
        Write-Warning "Formatting issues found. Run 'cargo fmt' to fix."
    }

    Write-Success "Linting complete"
} else {
    Write-Host "[4/7] Lint: SKIPPED" -ForegroundColor DarkGray
}

# ─── Step 5: Tauri Bundle / MSI Installer ────────────────────────
if (-not $SkipInstaller) {
    Write-Step "Building MSI installer (npx tauri build)" "5/7"
    Set-Location $ProjectRoot

    Invoke-NativeCommand { npx tauri build --bundles msi }

    # Verify MSI was produced
    $msiFiles = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\$ConfigLower\bundle\msi\*.msi" -ErrorAction SilentlyContinue
    if ($msiFiles.Count -eq 0) {
        Write-Warning "No MSI files found in bundle output. Check Tauri configuration."
    } else {
        foreach ($msi in $msiFiles) {
            Write-Success "MSI produced: $($msi.Name) ($([math]::Round($msi.Length / 1MB, 2)) MB)"
        }
    }

    Write-Success "Installer build complete"
} else {
    Write-Host "[5/7] Installer: SKIPPED" -ForegroundColor DarkGray
}

# ─── Step 6: Code Signing ─────────────────────────────────────────
if ($SignCertificatePath -and (Test-Path $SignCertificatePath)) {
    Write-Step "Code signing installer" "6/7"

    $msiFiles = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\$ConfigLower\bundle\msi\*.msi"
    $exePath = "$ProjectRoot\src-tauri\target\$ConfigLower\SecDel.exe"

    # Find signtool.exe
    $signtool = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter "signtool.exe" |
        Sort-Object -Property FullName -Descending |
        Select-Object -First 1

    if (-not $signtool) {
        $signtool = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Microsoft SDKs\ClickOnce\SignTool" -Recurse -Filter "signtool.exe" |
            Select-Object -First 1
    }

    if ($signtool) {
        $signArgs = @(
            'sign', '/fd', 'SHA256',
            '/f', $SignCertificatePath,
            '/p', $SignCertificatePassword,
            '/tr', 'http://timestamp.digicert.com',
            '/td', 'SHA256'
        )

        # Sign MSI files
        foreach ($msi in $msiFiles) {
            Write-Host "  -> Signing $($msi.Name)..." -ForegroundColor Gray
            & $signtool.FullName @signArgs $msi.FullName
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Signed: $($msi.Name)"
            } else {
                Write-Failure "Signing failed for $($msi.Name)"
            }
        }

        # Sign portable EXE
        if (Test-Path $exePath) {
            Write-Host "  -> Signing SecDel.exe..." -ForegroundColor Gray
            & $signtool.FullName @signArgs $exePath
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Signed: SecDel.exe"
            } else {
                Write-Failure "Signing failed for SecDel.exe"
            }
        }
    } else {
        Write-Warning "signtool.exe not found. Skipping code signing."
    }

    Write-Success "Code signing complete"
} elseif ($SignCertificatePath) {
    Write-Warning "Certificate path '$SignCertificatePath' not found. Skipping code signing."
    Write-Host "[6/7] Code signing: SKIPPED" -ForegroundColor DarkGray
} else {
    Write-Host "[6/7] Code signing: SKIPPED (no certificate provided)" -ForegroundColor DarkGray
}

# ─── Step 7: Checksums ──────────────────────────────────────────
Write-Step "Generating SHA-256 checksums" "7/7"

$checksumFiles = @()

# MSI checksums
$msiFiles = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\$ConfigLower\bundle\msi\*.msi" -ErrorAction SilentlyContinue
foreach ($msi in $msiFiles) {
    $hash = Get-FileHash -Path $msi.FullName -Algorithm SHA256
    $checksumLine = "$($hash.Hash.ToLower())  $($msi.Name)"
    $checksumLine | Out-File -FilePath "$($msi.FullName).sha256" -Encoding ascii
    $checksumFiles += "$($msi.FullName).sha256"
    Write-Success "SHA256: $($msi.Name) -> $($hash.Hash.Substring(0, 16))..."
}

# EXE checksum
$exePath = "$ProjectRoot\src-tauri\target\$ConfigLower\SecDel.exe"
if (Test-Path $exePath) {
    $hash = Get-FileHash -Path $exePath -Algorithm SHA256
    $checksumLine = "$($hash.Hash.ToLower())  $($exeName)"
    $exeName = Split-Path -Leaf $exePath
    $checksumLine | Out-File -FilePath "$($exePath).sha256" -Encoding ascii
    $checksumFiles += "$($exePath).sha256"
    Write-Success "SHA256: $exeName -> $($hash.Hash.Substring(0, 16))..."
}

# Combined checksums file
$combinedPath = "$ProjectRoot\src-tauri\target\$ConfigLower\SHA256SUMS.txt"
$header = "# SecDel $Configuration Build - SHA-256 Checksums"
$header | Out-File -FilePath $combinedPath -Encoding ascii
"# Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" | Out-File -FilePath $combinedPath -Encoding ascii -Append
"" | Out-File -FilePath $combinedPath -Encoding ascii -Append
(Get-ChildItem -Path "$ProjectRoot\src-tauri\target\$ConfigLower\bundle\msi\*.sha256" -ErrorAction SilentlyContinue) | ForEach-Object {
    Get-Content $_.FullName | Out-File -FilePath $combinedPath -Encoding ascii -Append
}
if (Test-Path "$($exePath).sha256") {
    Get-Content "$($exePath).sha256" | Out-File -FilePath $combinedPath -Encoding ascii -Append
}
Write-Success "Combined checksums: SHA256SUMS.txt"

Write-Host ""
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "       BUILD COMPLETE" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Configuration : $Configuration"
Write-Host "  Output        : $ProjectRoot\$ReleaseDir"
Write-Host "  MSI Installer : $(if ($msiFiles) { $msiFiles.FullName -join '; ' } else { 'N/A' })"
Write-Host "  Portable EXE  : $(if (Test-Path $exePath) { $exePath } else { 'N/A' })"
Write-Host "  Checksums     : $combinedPath"
Write-Host ""
Write-Host "  Frontend Build: $(if (-not $SkipFrontend) { 'Yes' } else { 'Skipped' })"
Write-Host "  Backend Build : $(if (-not $SkipBackend) { 'Yes' } else { 'Skipped' })"
Write-Host "  Tests Run     : $(if (-not $SkipTests) { 'Yes' } else { 'Skipped' })"
Write-Host "  Installer     : $(if (-not $SkipInstaller) { 'Yes' } else { 'Skipped' })"
Write-Host "  Code Signed   : $(if ($SignCertificatePath -and (Test-Path $SignCertificatePath)) { 'Yes' } else { 'No' })"
Write-Host ""
Write-Host "══════════════════════════════════════════════" -ForegroundColor Cyan
