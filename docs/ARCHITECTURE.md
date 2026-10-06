# SecDel Architecture

> Version 1.0.0 — June 2026

---

## Table of Contents

1. [Overview](#1-overview)
2. [System Architecture](#2-system-architecture)
3. [Component Architecture](#3-component-architecture)
4. [Data Flow](#4-data-flow)
5. [Module Responsibilities](#5-module-responsibilities)
6. [Security Design Principles](#6-security-design-principles)
7. [Thread Safety Model](#7-thread-safety-model)
8. [State Management](#8-state-management)
9. [Error Handling Strategy](#9-error-handling-strategy)
10. [Plugin Architecture](#10-plugin-architecture)
11. [Configuration Management](#11-configuration-management)
12. [Build and Deployment Architecture](#12-build-and-deployment-architecture)

---

## 1. Overview

SecDel is a cross-platform desktop application built with **Tauri v2** (Rust backend) and **Svelte 5** (frontend) that provides verified, cryptographically secure deletion of files and entire drives on SSD storage.

### 1.1 Design Goals

- **Defense in depth**: Multiple layers of secure deletion techniques
- **Verifiability**: Every operation is verified and logged
- **Performance**: Asynchronous, parallel deletion pipelines
- **Auditability**: Complete cryptographic audit trails
- **Usability**: Clear Svelte-based UI with real-time progress

### 1.2 Technology Stack

| Layer       | Technology              | Purpose                        |
|-------------|--------------------------|--------------------------------|
| Frontend    | Svelte 5 + TypeScript    | Reactive UI framework          |
| Backend     | Rust + Tauri v2          | Native OS operations           |
| IPC         | Tauri Commands           | Secure frontend-backend bridge |
| Storage     | SQLite (via rusqlite)    | Local audit logs               |
| Crypto      | RustCrypto crates        | CSPRNG, hashing                |
| Installer   | WiX Toolset v4           | Windows MSI packaging          |
| CI/CD       | GitHub Actions           | Automated build/test/release   |

---

## 2. System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interface (Svelte 5)                │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Dashboard   │  │ Deletion     │  │ Reports & Audit      │   │
│  │ View        │  │ Workflow     │  │ Log Viewer           │   │
│  └──────┬──────┘  └──────┬───────┘  └──────────┬───────────┘   │
│         │                │                      │               │
│  ┌──────┴────────────────┴──────────────────────┴───────────┐  │
│  │              Tauri IPC Bridge (Commands)                 │  │
│  │  invoke('erase_file')  invoke('verify')  invoke('scan') │  │
│  └──────────────────────────┬───────────────────────────────┘  │
├─────────────────────────────┼─────────────────────────────────┤
│                     Rust Backend (Tauri)                       │
│  ┌──────────────────────────┴───────────────────────────────┐  │
│  │                    Command Dispatcher                     │  │
│  └──┬─────────┬──────────┬──────────┬──────────┬───────────┘  │
│     │         │          │          │          │               │
│  ┌──┴───┐ ┌───┴────┐ ┌──┴────┐ ┌──┴────┐ ┌──┴────────┐      │
│  │Erase │ │Verify  │ │Scan   │ │Report │ │Config     │      │
│  │Engine│ │Engine  │ │Engine │ │Engine │ │Manager    │      │
│  └──┬───┘ └───┬────┘ └──┬────┘ └──┬────┘ └──┬─────────┘      │
│     │         │         │         │          │                │
│  ┌──┴─────────┴─────────┴─────────┴──────────┴───────────┐   │
│  │                Service Layer                           │   │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌───────────┐   │   │
│  │  │CSPRNG    │ │Disk I/O  │ │FS Ops  │ │Logging    │   │   │
│  │  │(OsRng)   │ │Engine    │ │        │ │(tracing)   │   │   │
│  │  └──────────┘ └──────────┘ └────────┘ └───────────┘   │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │                 OS Abstraction Layer                    │   │
│  │  ┌─────────┐ ┌────────┐ ┌──────────┐ ┌─────────────┐  │   │
│  │  │Windows  │ │Linux   │ │macOS     │ │Platform     │  │   │
│  │  │API      │ │io_uring│ │DKIOC     │ │Detection    │  │   │
│  │  │(NtAPI)  │ │        │ │          │ │             │  │   │
│  │  └─────────┘ └────────┘ └──────────┘ └─────────────┘  │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Component Architecture

### 3.1 Core Components

#### EraseEngine (`src-tauri/src/erase/`)

The heart of SecDel. Responsible for executing secure deletion passes.

```
EraseEngine
├── FileEraseJob        — Erase a single file
├── DirectoryEraseJob   — Erase an entire directory tree
├── DriveEraseJob       — Sanitize all free space on a drive
├── PassStrategy         — Strategy pattern for overwrite passes
│   ├── SinglePass      — One pass of random data
│   ├── ThreePass       — 0x00 → 0xFF → random (DoD 5220.22-M)
│   ├── SevenPass       — Gutmann variant
│   └── CustomPass      — User-defined pass pattern
├── VerificationEngine  — Verify data was overwritten
└── IoctlHandler        — Platform-specific TRIM/deallocate
```

#### VerifyEngine (`src-tauri/src/verify/`)

Reads back overwritten sectors and cryptographically verifies that old data is unrecoverable.

```
VerifyEngine
├── BlockVerifier       — Read and compare blocks
├── StatisticalAnalyzer — Check residual magnetization
└── ReportGenerator     — Produce verification report
```

#### ScanEngine (`src-tauri/src/scan/`)

Scans files/directories/drives to identify deletable content and artifacts.

```
ScanEngine
├── FileScanner         — Enumerate files matching criteria
├── ArtifactScanner     — Find temp files, logs, thumbnails, etc.
├── DriveScanner        — Analyze partition layout and free space
└── ReportBuilder       — Build scan results for UI
```

#### ReportEngine (`src-tauri/src/report/`)

Generates and persists audit trails for all operations.

```
ReportEngine
├── AuditRecorder       — Write operation records to SQLite
├── ReportSerializer    — JSON/PDF report generation
├── ExportManager       — Export reports to file
└── IntegrityChecker    — Verify report integrity (HMAC)
```

#### ConfigManager (`src-tauri/src/config/`)

Manages application, security policy, and user preferences.

```
ConfigManager
├── AppConfig           — Application settings
├── SecurityPolicy      — Minimum pass count, algorithms
├── UserPreferences     — UI theme, language, behavior
├── ProfileManager      — Named security profiles
└── ConfigValidator     — Validate configuration integrity
```

---

## 4. Data Flow

### 4.1 Secure File Deletion Flow

```
User clicks "Erase File"
        │
        ▼
┌─────────────────────────────┐
│  Frontend (Svelte)          │
│  - Builds erase request     │
│  - Shows confirmation modal │
│  - On confirm, invoke()     │
└──────────┬──────────────────┘
           │  invoke('erase_files', { paths, passes: 3, verify: true })
           ▼
┌─────────────────────────────┐
│  Tauri Command Handler      │
│  - Validate inputs          │
│  - Check permissions        │
│  - Acquire file lock        │
│  - Spawn async task         │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│  EraseEngine                │
│  ├─ Resolve paths           │
│  ├─ Check file attributes   │
│  ├─ Log operation start     │
│  │                          │
│  │  ┌───────────────────┐   │
│  │  │  Pass 1: Write    │   │
│  │  │  CSPRNG data ─────│───│──► Disk I/O Engine
│  │  │  Sync/fsync       │   │   │
│  │  │  Verify blocks    │◄──│───│◄─ Read back
│  │  └───────────────────┘   │   │
│  │  ┌───────────────────┐   │   │
│  │  │  Pass 2: Write    │   │   │
│  │  │  0x00             │───│──► Disk I/O Engine
│  │  │  Sync/fsync       │   │   │
│  │  │  Verify blocks    │◄──│───│◄─ Read back
│  │  └───────────────────┘   │   │
│  │  ┌───────────────────┐   │   │
│  │  │  Pass 3: Write    │   │   │
│  │  │  0xFF             │───│──► Disk I/O Engine
│  │  │  Sync/fsync       │   │   │
│  │  │  Verify blocks    │◄──│───│◄─ Read back
│  │  └───────────────────┘   │   │
│  │                          │   │
│  ├─ Issue TRIM/deallocate   │   │
│  ├─ Rename → delete file    │   │
│  └─ Return result           │   │
└──────────┬──────────────────┘   │
           │                      │
           ▼                      ▼
┌──────────────────────┐  ┌──────────────┐
│  ReportEngine        │  │  Disk I/O    │
│  ├─ Write audit log  │  │  Engine      │
│  ├─ Generate report  │  │  ├─ Write()  │
│  └─ Return to UI     │  │  ├─ Read()   │
└──────────────────────┘  │  ├─ Sync()   │
                          │  └─ Ioctl()  │
                          └──────────────┘
```

### 4.2 Event Bus Data Flow

```
Frontend Component
       │
       │  Event: 'erase:progress'
       ▼
┌────────────────────┐
│  Svelte Store      │
│  (reactive state)  │
└────────┬───────────┘
         │
         │  listen('erase:progress')
         ▼
┌────────────────────┐
│  Tauri Event Bus   │
│  (global events)   │
└────────┬───────────┘
         │
         │  emit('erase:progress', { file, percent, speed })
         ▼
┌────────────────────┐
│  Rust Backend      │
│  (progress events) │
└────────────────────┘
```

---

## 5. Module Responsibilities

### 5.1 Rust Backend Modules

| Module         | Path                        | Responsibility                              |
|----------------|-----------------------------|---------------------------------------------|
| `secdel-core`  | `src-tauri/src/`            | Main crate, command dispatcher, app setup   |
| `erase`        | `src-tauri/src/erase/`      | Secure deletion pass engine                 |
| `verify`       | `src-tauri/src/verify/`     | Post-erase verification                     |
| `scan`         | `src-tauri/src/scan/`       | File, artifact, and drive scanning          |
| `report`       | `src-tauri/src/report/`     | Audit logging and report generation         |
| `config`       | `src-tauri/src/config/`     | Configuration and security policy           |
| `platform`     | `src-tauri/src/platform/`   | OS-specific disk operations                 |
| `crypto`       | `src-tauri/src/crypto/`     | CSPRNG, HMAC, hashing utilities             |
| `ffi`          | `src-tauri/src/ffi/`        | Safe FFI wrappers for Win32/Linux syscalls  |

### 5.2 Frontend Modules

| Module              | Path                     | Responsibility                         |
|---------------------|--------------------------|----------------------------------------|
| `Dashboard.svelte`  | `src/lib/pages/`         | Main dashboard with system overview    |
| `EraseWizard.svelte`| `src/lib/pages/`         | Multi-step secure deletion workflow    |
| `DriveSanitize.svelte`| `src/lib/pages/`       | Full drive sanitization interface      |
| `Scanner.svelte`    | `src/lib/pages/`         | Artifact and file scanner              |
| `Reports.svelte`    | `src/lib/pages/`         | Audit log viewer and export            |
| `Settings.svelte`   | `src/lib/pages/`         | Configuration and preferences          |
| `stores/`           | `src/lib/stores/`        | Svelte writable stores for shared state |
| `components/`       | `src/lib/components/`    | Reusable UI components                 |
| `lib/`              | `src/lib/lib/`           | Utility functions and helpers          |

---

## 6. Security Design Principles

### 6.1 Principle of Least Privilege

SecDel operates with the minimum privilege level required:

- **User-mode operations**: File-level deletion runs at user privilege
- **Administrator elevation**: Drive-level and free-space sanitization require elevation
- **Token separation**: Audit logging runs with separate token to prevent log tampering
- **Process isolation**: Heavy operations may spawn child processes at required privilege

### 6.2 Defense in Depth

```
Layer 1: Application Security
├── Input validation (all IPC commands)
├── Path traversal protection
├── Safe file handle management
└── Tauri capability whitelist

Layer 2: Cryptographic Security
├── CSPRNG (OsRng / getrandom)
├── Constant-time comparisons
├── Memory zeroization (zeroize crate)
└── Secure key management

Layer 3: OS Security
├── DACL-based file access control
├── Mandatory Integrity Control (MIC)
├── Windows Defender integration
└── Secure attention sequence

Layer 4: Physical Security
├── TRIM/deallocate commands
├── NVMe sanitize operations
├── ATA SECURE ERASE passthrough
└── Power-loss safe designs
```

### 6.3 Fail-Safe Defaults

- All operations default to the most secure settings
- Verification required by default
- Operations roll back on failure
- Partial results are never reported as complete

### 6.4 Complete Mediation

- Every access to every object is checked for authority
- Permissions evaluated on every IPC call
- File handles validated on every read/write

---

## 7. Thread Safety Model

### 7.1 Rust Ownership + Send + Sync

```rust
// EraseJob is Send + Sync
pub struct EraseJob {
    paths: Vec<PathBuf>,
    config: Arc<EraseConfig>,       // Shared, immutable config
    progress: Arc<AtomicProgress>,   // Atomic progress tracking
    events: EventEmitter,            // Tauri event emitter (Send + Sync)
    canceller: Arc<AtomicBool>,      // Cancellation flag
}
```

### 7.2 Concurrency Strategy

| Resource          | Strategy                | Mechanism                    |
|-------------------|-------------------------|------------------------------|
| Disk I/O          | Async (tokio)           | Spawn blocking pool          |
| Progress updates  | Atomic + channel        | Arc<AtomicU64> + mpsc        |
| Event emissions   | Tauri event bus         | Single-writer, fan-out       |
| SQLite access     | Mutex (rusqlite)        | Arc<Mutex<Connection>>       |
| Configuration     | RwLock                  | Arc<RwLock<Config>>          |
| Cancellation      | Atomic flag             | Arc<AtomicBool>              |
| Logging           | Global subscriber       | tracing_subscriber           |
| CSPRNG state      | Thread-local            | OsRng (no state to share)    |

### 7.3 Async Pipeline

```
Input Queue (mpsc::channel)
    │
    ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  File        │───►│  Erase Pass  │───►│  Verify      │
│  Resolver    │    │  N (1-35)    │    │  Engine      │
└──────────────┘    └──────────────┘    └──────┬───────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │  Report      │
                                        │  Writer      │
                                        └──────────────┘
```

---

## 8. State Management

### 8.1 Application State

```rust
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub db: Arc<Mutex<Connection>>,
    pub active_jobs: Arc<DashMap<Uuid, JobHandle>>,
    pub system_info: Arc<OnceCell<SystemInfo>>,
}
```

### 8.2 Frontend State (Svelte Stores)

```typescript
// Application-level stores
appState: writable<AppState>
config: writable<Config>
jobs: writable<Map<string, JobProgress>>
reports: writable<Report[]>
systemInfo: writable<SystemInfo>

// Derived stores
activeJobCount: derived<number>
overallProgress: derived<number>
securityStatus: derived<SecurityLevel>
```

### 8.3 State Transitions

```
IDLE ──► CONFIRMING ──► ERASING ──► VERIFYING ──► REPORTING ──► COMPLETE
  │          │             │             │             │
  └──────────┴─────────────┴─────────────┴─────────────┘
                          │
                      CANCELLING ──► PARTIAL_COMPLETE
                          │
                      FAILED
```

---

## 9. Error Handling Strategy

### 9.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum SecDelError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Verification failed: expected {expected:?}, got {actual:?}")]
    VerificationFailed { expected: Vec<u8>, actual: Vec<u8> },

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Other(String),
}
```

### 9.2 Error Handling Flow

```
IPC Command
    │
    ▼
┌────────────┐     Success ──────► Return Ok(T)
│  Operation │
│  Execution │     Error ─────────┐
└────────────┘                    │
                                 ▼
                       ┌─────────────────────┐
                       │  Error Classification│
                       │  ├─ Retryable?       │
                       │  ├─ User-actionable? │
                       │  └─ Security event?  │
                       └──────────┬──────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    ▼             ▼             ▼
              ┌──────────┐ ┌──────────┐ ┌──────────────┐
              │  Retry   │ │  Show UI │ │ Log + Alert  │
              │  Logic   │ │  Error   │ │ (security)   │
              └──────────┘ └──────────┘ └──────────────┘
                    │             │             │
                    └─────────────┼─────────────┘
                                  ▼
                       ┌─────────────────────┐
                       │  Return Err to UI   │
                       │  + Audit Log Entry  │
                       └─────────────────────┘
```

### 9.3 Recovery Strategies

| Error Type            | Recovery                                     |
|-----------------------|----------------------------------------------|
| Disk I/O error        | Retry 3x with backoff; fail if persistent    |
| Permission denied     | Elevation prompt; fail if denied             |
| Verification mismatch | Re-run pass; escalate if persistent          |
| Power loss            | Resume from audit log on next launch         |
| File locked           | Wait + retry; prompt user to close handle    |
| Out of disk space     | Pause; notify user; resume on space freed    |

---

## 10. Plugin Architecture

### 10.1 Plugin System

SecDel supports a plugin system for extensibility (e.g., custom pass patterns, new verification methods, external integrations).

```rust
#[async_trait]
pub trait ErasePlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    async fn on_erase_start(&self, job: &EraseJob) -> Result<()>;
    async fn on_pass_complete(&self, pass: &PassResult) -> Result<()>;
    async fn on_erase_complete(&self, result: &EraseResult) -> Result<()>;
}
```

### 10.2 Plugin Loading

```
Plugin Registry
├── Built-in plugins (pass strategies, verifiers)
├── User plugins (loaded from ~/.secdel/plugins/)
└── Remote plugins (verified via signature)
```

---

## 11. Configuration Management

### 11.1 Configuration Sources (priority order)

1. **Command-line flags** — Highest priority, overrides everything
2. **Environment variables** — `SECDEL_*` prefixed vars
3. **User config file** — `%APPDATA%\SecDel\config.toml`
4. **Security policy file** — `%PROGRAMDATA%\SecDel\policy.toml` (admin-managed)
5. **System defaults** — Hardcoded safe defaults

### 11.2 Configuration Schema

```toml
# ~.secdel\config.toml
[security]
passes = 3                          # Number of overwrite passes (1-35)
verify = true                       # Verify after each pass
verify_samples = 5                  # Percentage of blocks to verify
trim_after_erase = true             # Issue TRIM after overwrite
rename_before_delete = true         # Random name before deletion
random_name_length = 16             # Length of random temporary name

[erase]
pass_pattern = "doe"                # Pattern preset: single, doe, gutmann, custom
block_size = 1048576                # 1 MB block size for I/O
parallel_files = 4                  # Parallel file erase count
speed_limit_mbps = 0                # 0 = unlimited

[logging]
audit_log = true                    # Write audit log to database
log_level = "info"                  # tracing log level
log_file = "SecDel.log"            # Log file name (in app data dir)

[report]
format = "pdf"                      # Report format: pdf, json, html
include_system_info = true          # Include system info in reports
auto_export = false                 # Auto-export reports after operations
export_path = ""                    # Export directory

[ui]
theme = "system"                    # light, dark, system
language = "en"                     # UI language
confirm_dialog = true               # Show confirmation before erase
progress_animation = true           # Animated progress indicators
```

---

## 12. Build and Deployment Architecture

### 12.1 Build Pipeline

```
Source Code
    │
    ▼
┌──────────────────────┐
│  Rust Compiler       │
│  (cargo build)       │
│  ├─ secdel_core (lib)│
│  └─ secdel (binary)  │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  Tauri Bundler       │
│  ├─ Frontend (Svelte)│
│  ├─ Rust binary      │
│  └─ WebView2 runtime │
└──────────┬───────────┘
           │
     ┌─────┴─────┐
     ▼           ▼
┌─────────┐ ┌─────────┐
│  MSI    │ │  EXE    │
│  (WiX)  │ │  (port) │
└────┬────┘ └────┬────┘
     │           │
     ▼           ▼
  Code Sign    Code Sign
     │           │
     ▼           ▼
Checksums    Checksums
     │           │
     └─────┬─────┘
           ▼
   GitHub Release
```

### 12.2 Directory Layout

```
secdel/
├── .github/workflows/     — CI/CD pipeline definitions
├── installer/              — WiX installer sources
│   ├── secdel.wxs          — Main WiX source
│   ├── license.rtf         — License RTF for installer UI
│   ├── banner.bmp          — Installer banner graphic
│   └── dialog.bmp          — Installer dialog graphic
├── src/                    — Svelte frontend source
├── src-tauri/              — Rust/Tauri backend
│   ├── src/                — Rust source code
│   │   ├── main.rs         — App entry point
│   │   ├── lib.rs          — Library root
│   │   ├── commands/       — Tauri command handlers
│   │   ├── erase/          — Erase engine
│   │   ├── verify/         — Verification engine
│   │   ├── scan/           — Scanner engine
│   │   ├── report/         — Audit & report engine
│   │   ├── config/         — Configuration manager
│   │   ├── platform/       — Platform-specific code
│   │   ├── crypto/         — Cryptographic primitives
│   │   └── ffi/            — Safe FFI wrappers
│   ├── icons/              — Application icons
│   ├── Cargo.toml          — Rust manifest
│   └── tauri.conf.json     — Tauri configuration
├── docs/                   — Documentation
├── build.ps1               — PowerShell build script
├── build.sh                — Bash build script (Linux/macOS)
├── package.json            — Node.js manifest
├── vite.config.ts          — Vite configuration
└── svelte.config.js        — Svelte configuration
```

### 12.3 Deployment Targets

| Target              | Architecture | Installer        | Minimum OS      |
|---------------------|-------------|------------------|-----------------|
| Windows 10/11       | x64         | MSI (WiX v4)     | Windows 10 20H1 |
| Windows Server      | x64         | MSI (WiX v4)     | Server 2019     |
| Linux (future)      | x86_64      | .deb/.rpm/AppImage | Ubuntu 22.04   |
| macOS (future)      | aarch64     | .dmg             | macOS 14        |

---

*This architecture document is maintained as part of the SecDel project. Updates should be reviewed by the security team.*
