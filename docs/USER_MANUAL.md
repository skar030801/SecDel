# SecDel User Manual

> Version 1.0.0 — June 2026

---

## Table of Contents

1. [Introduction to SecDel](#1-introduction-to-secdel)
2. [Installation Guide](#2-installation-guide)
3. [Quick Start Guide](#3-quick-start-guide)
4. [Dashboard Overview](#4-dashboard-overview)
5. [Secure Deletion Walkthrough](#5-secure-deletion-walkthrough)
6. [Full Drive Sanitization](#6-full-drive-sanitization)
7. [Using the Artifact Scanner](#7-using-the-artifact-scanner)
8. [Understanding Verification Results](#8-understanding-verification-results)
9. [Reports and Logging](#9-reports-and-logging)
10. [Configuration Reference](#10-configuration-reference)
11. [Security Level Reference Table](#11-security-level-reference-table)
12. [TRIM Technology Explanation](#12-trim-technology-explanation)
13. [SSD Limitations and Realistic Expectations](#13-ssd-limitations-and-realistic-expectations)
14. [FAQ](#14-faq)
15. [Troubleshooting](#15-troubleshooting)
16. [Glossary](#16-glossary)

---

## 1. Introduction to SecDel

### 1.1 What Is SecDel?

SecDel (Secure Deletion) is a desktop application that performs **verified, cryptographically secure deletion** of files, directories, and entire drive free space on modern SSD storage. Unlike standard file deletion (which only removes the file system reference, leaving data recoverable), SecDel overwrites the underlying data, verifies it has been written, and optionally issues TRIM commands to the SSD controller.

### 1.2 Why SecDel?

**The problem with standard deletion:**
- Deleting a file only removes its entry from the file system table
- The actual data remains on the storage media until overwritten
- File recovery tools can easily restore "deleted" files
- SSDs complicate matters with wear leveling and over-provisioning

**The SecDel approach:**
- Multiple overwrite passes with cryptographically secure random data
- Post-erase verification to confirm data was written
- TRIM/deallocate commands for SSD-aware sanitization
- Complete audit trail for compliance
- Administrator-only privileges for drive-level operations

### 1.3 Who Is SecDel For?

- **IT administrators**: Sanitizing drives before decommissioning
- **Security professionals**: Ensuring data cannot be recovered
- **Privacy-conscious users**: Permanently deleting sensitive files
- **Compliance officers**: Meeting GDPR/HIPAA/PCI-DSS disposal requirements
- **Organizations**: Implementing data retention and destruction policies

---

## 2. Installation Guide

### 2.1 System Requirements

| Requirement          | Minimum                           | Recommended                      |
|----------------------|-----------------------------------|----------------------------------|
| Operating System     | Windows 10 20H1 (x64)             | Windows 11 23H2 or later         |
| Processor            | Dual-core 2.0 GHz                 | Quad-core 3.0 GHz+               |
| RAM                  | 4 GB                              | 8 GB+                            |
| Disk Space           | 200 MB (application)              | 500 MB+ (for temp files)         |
| Storage to sanitize  | Any ATA/NVMe SSD or HDD           | NVMe SSD recommended             |
| Administrator access | Required for drive-level ops      | Required for drive-level ops     |
| Internet             | Required for updates (optional)   | Broadband connection             |

### 2.2 Installation Steps

#### Option A: MSI Installer (Recommended)

1. **Download** the latest MSI from the repository's **Releases** page.
2. **Verify** the SHA-256 checksum:
   ```
   certutil -hashfile SecDel-1.0.0.msi SHA256
   ```
   Compare with the published checksum on the releases page.

3. **Run as Administrator**:
   - Right-click the MSI → **Install**
   - Or double-click and accept the UAC prompt

4. **Follow the installer wizard**:
   - Accept the license agreement
   - Choose installation directory (default: `C:\Program Files\SecDel\`)
   - Select components:
     - ✅ SecDel Application
     - ✅ Start Menu Shortcut
     - ✅ Desktop Shortcut
     - ✅ File Association (.secdel-report)

5. **Launch** SecDel from the Start Menu or desktop shortcut.

#### Option B: Portable Executable

1. **Download** `SecDel.exe` from the releases page
2. **Place** it in any directory (e.g., USB drive)
3. **Run as Administrator** (right-click → Run as Administrator)

> **Note**: The portable version cannot perform drive-level operations without admin rights. File-level deletion works in user mode.

### 2.3 Silent Installation (IT Deployments)

```
msiexec /i SecDel-1.0.0.msi /qn INSTALLFOLDER="C:\Program Files\SecDel" /norestart
```

### 2.4 Updating

SecDel supports in-app updates:
1. Go to **Settings → Updates**
2. Click **Check for Updates**
3. If available, click **Download and Install**
4. The installer will upgrade your existing installation

---

## 3. Quick Start Guide

### 3.1 Your First Secure Deletion

```
Step 1: Launch SecDel (Run as Administrator for best results)
Step 2: Click "Erase Files" on the Dashboard
Step 3: Select the file(s) to delete
Step 4: Choose security level: "Standard" (default)
Step 5: Click "Start Erase"
Step 6: Confirm the operation
Step 7: Wait for completion and verification
Step 8: Review the verification report
```

**Total time**: A 100 MB file takes approximately 10-30 seconds with 3 passes.

### 3.2 Keyboard Shortcuts

| Shortcut            | Action                          |
|---------------------|---------------------------------|
| `Ctrl+E`            | Open erase dialog               |
| `Ctrl+S`            | Open scan dialog                |
| `Ctrl+R`            | View reports                    |
| `Ctrl+,`            | Open settings                   |
| `Ctrl+Q`            | Quit application                |
| `Escape`            | Cancel current operation        |
| `Ctrl+Shift+I`      | Toggle dark/light mode          |

---

## 4. Dashboard Overview

When you launch SecDel, you are greeted with the **Dashboard**:

```
┌─────────────────────────────────────────────────────────────────┐
│  SecDel v1.0.0  ───  [Dashboard] [Erase] [Scan] [Reports] [⚙] │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Quick Erase  │  │  Scan Files  │  │  Full Drive  │          │
│  │  [Click here] │  │  [Click here]│  │  Sanitize    │          │
│  └──────────────┘  └──────────────┘  │  [Click here] │          │
│                                       └──────────────┘          │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Recent Activity                                           │   │
│  │  ┌────────┬──────────┬────────┬────────┬──────────────┐  │   │
│  │  │ Time   │ Operation│ Status │ Size   │ Passes       │  │   │
│  │  ├────────┼──────────┼────────┼────────┼──────────────┤  │   │
│  │  │ 14:23  │ Erase    │ ✅     │ 45 MB  │ 3 verify     │  │   │
│  │  │ 14:20  │ Scan     │ ✅     │ 1.2 GB │ —            │  │   │
│  │  │ 13:55  │ Erase    │ ⚠️     │ 2.1 GB │ 1 verify     │  │   │
│  │  └────────┴──────────┴────────┴────────┴──────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  System Status: ✅ Administrator | 💾 C:\ 45% free | 🔒 TRIM OK│
└─────────────────────────────────────────────────────────────────┘
```

### Dashboard Elements

| Element              | Description                                       |
|----------------------|---------------------------------------------------|
| Navigation bar       | Top menu: Dashboard, Erase, Scan, Reports, Settings|
| Quick Action Cards   | One-click access to common operations             |
| Recent Activity      | Last 10 operations with status indicators          |
| System Status Bar    | Admin status, disk usage, TRIM support             |
| Security Indicator   | Current effective security level (shield icon)     |

---

## 5. Secure Deletion Walkthrough

### 5.1 Step-by-Step: Erase Files

```
┌─────────────────────────────────────────────────────────────┐
│  Step 1: Select Files                                      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  📁 Documents\secrets\                                │   │
│  │  ├─ budget_2027.xlsx  [45.2 MB]  ☑                   │   │
│  │  ├─ strategy.pdf      [12.8 MB]  ☑                   │   │
│  │  └─ notes.txt         [0.1 MB]   ☑                   │   │
│  │  └─ confidential\                                    │   │
│  │      └─ report.docx   [8.3 MB]   ☑                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                     [Browse...] [Add Files...]              │
│                                                             │
│  Total size: 66.4 MB  |  Files selected: 4                 │
│                                                             │
│                                         [Next →]            │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Security Level Selection

```
┌─────────────────────────────────────────────────────────────┐
│  Step 2: Security Level                                     │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Security Profile:  [Standard       ▼]              │   │
│  │                                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │   │
│  │  │ Quick        │  │ Standard     │  │ Maximum  │  │   │
│  │  │ 1 pass       │  │ 3 passes     │  │ 7 passes │  │   │
│  │  │ Basic verify │  │ Full verify  │  │ Full ver │  │   │
│  │  │ ✓ TRIM       │  │ ✓ TRIM       │  │ ✓ TRIM   │  │   │
│  │  │              │  │ ✓ Random rn  │  │ ✓ Random │  │   │
│  │  │              │  │ ✓ Dir scrub  │  │ ✓ Dir scr│  │   │
│  │  └──────────────┘  └──────────────┘  └──────────┘  │   │
│  │                                                     │   │
│  │  Custom Settings:                                    │   │
│  │  Passes: [3]  Verify: [✅]  TRIM: [✅]              │   │
│  │  Random rename: [✅]  Directory scrub: [✅]         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Estimated time: ~45 seconds                                │
│                                         [← Back] [Start →] │
└─────────────────────────────────────────────────────────────┘
```

### 5.3 Erase Progress

```
┌─────────────────────────────────────────────────────────────┐
│  Erasing...  [■■■■■■■□□□□□□□□□□□□□□□]  35%                 │
│                                                             │
│  File: budget_2027.xlsx  (Pass 2 of 3)                      │
│  Speed: 245 MB/s  |  Elapsed: 12s  |  Remaining: ~22s      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  📋 Progress:                                       │   │
│  │  ☑ budget_2027.xlsx      Pass 2/3  ✓ Verified     │   │
│  │  ☐ strategy.pdf          Queued                     │   │
│  │  ☐ notes.txt            Queued                     │   │
│  │  ☐ report.docx          Queued                     │   │
│  │                                                     │   │
│  │  ☑ 45 MB written  |  ✓ 15 MB verified              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  [Cancel Operation]                                         │
└─────────────────────────────────────────────────────────────┘
```

### 5.4 Verification Results

```
┌─────────────────────────────────────────────────────────────┐
│  ✅ Erase Complete — All files securely deleted             │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Summary:                                            │   │
│  │  Files erased: 4 of 4                                │   │
│  │  Total data:   66.4 MB                              │   │
│  │  Passes:       3                                    │   │
│  │  Time:         38 seconds                           │   │
│  │  TRIM issued:  Yes  ✓                               │   │
│  │  Verification: 100% blocks verified                  │   │
│  │  Original name: Randomly renamed  ✓                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  File Details:                                              │
│  ┌────────────┬────────┬──────────┬──────────┬──────────┐ │   │
│  │ File       │ Status │ Passes   │ Speed    │Verified  │ │   │
│  ├────────────┼────────┼──────────┼──────────┼──────────┤ │   │
│  │ budget.xlsx│ ✅     │ 3/3      │ 256 MB/s │ 100%     │ │   │
│  │ strategy   │ ✅     │ 3/3      │ 312 MB/s │ 100%     │ │   │
│  │ notes.txt  │ ✅     │ 3/3      │ 1.2 GB/s │ 100%     │ │   │
│  │ report.docx│ ✅     │ 3/3      │ 298 MB/s │ 100%     │ │   │
│  └────────────┴────────┴──────────┴──────────┴──────────┘ │   │
│                                                             │
│   [View Full Report]  [Export Report]  [Back to Dashboard]  │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Full Drive Sanitization

### 6.1 When to Use

Full drive sanitization is appropriate when:
- **Decommissioning** a drive before disposal or sale
- **Repurposing** a drive for a different department/user
- **Compliance** requires evidence of complete media sanitization
- **Returning** leased hardware

### 6.2 Drive Sanitization Walkthrough

```
┌─────────────────────────────────────────────────────────────┐
│  Full Drive Sanitize                                        │
│                                                             │
│  ⚠️ WARNING: This will make all data on the selected       │
│     drive permanently unrecoverable.                        │
│                                                             │
│  Select Drive: [C:\ System Drive        ▼]  [Refresh]      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Drive Information:                                  │   │
│  │  Drive:      C:\ (NVMe)                             │   │
│  │  Model:      Samsung SSD 990 Pro 1TB                │   │
│  │  Size:       953 GB                                 │   │
│  │  Used:       342 GB (36%)                            │   │
│  │  Free:       611 GB (64%)                            │   │
│  │  TRIM:       Supported and enabled                   │   │
│  │  Sanitize:   Crypto Erase available                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Sanitize Method: [NVMe Crypto Erase (Recommended) ▼]      │
│                                                             │
│  Options:                                                   │
│  ☑ Verify after sanitize                                   │
│  ☐ Quick mode (passes: 1, no verify)                       │
│                                                             │
│  Estimated time: ~2 hours (full pass)                       │
│                                                             │
│  [Start Sanitization] is disabled — requires admin rights   │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 Sanitize Methods Comparison

| Method                  | Time (1 TB) | Effectiveness | SSD Compat.   | HDD Compat. |
|-------------------------|-------------|---------------|---------------|-------------|
| Overwrite free space    | 2-4 hours   | High          | All           | All         |
| ATA SECURE ERASE        | 1-3 hours   | Very High     | SATA          | SATA        |
| NVMe Format NVM         | 1-5 minutes | Very High     | NVMe          | N/A         |
| NVMe Sanitize (crypto)  | 1-3 seconds | Highest       | NVMe (some)   | N/A         |
| NVMe Sanitize (block)   | 1-5 minutes | Very High     | NVMe (some)   | N/A         |

---

## 7. Using the Artifact Scanner

### 7.1 What the Scanner Finds

The artifact scanner identifies potentially sensitive data remnants:

| Artifact Type          | Examples                                        | Risk  |
|------------------------|-------------------------------------------------|-------|
| Temporary files        | `*.tmp`, `~$*.docx`                             | High  |
| Browser cache          | Chrome/Edge/Firefox caches                      | High  |
| Browser history        | SQLite databases, `History`, `Cookies`          | High  |
| Prefetch files         | `C:\Windows\Prefetch\*.pf`                     | Medium|
| Thumbnail cache        | `thumbcache_*.db`                               | Medium|
| Recycle Bin            | `$Recycle.Bin\*`                                | High  |
| Recent files           | `%APPDATA%\Microsoft\Windows\Recent\*`          | Medium|
| Clipboard history      | `%LOCALAPPDATA%\Microsoft\Windows\Clipboard\*`  | Medium|
| Windows Event Logs     | `*.evtx` (depending on logged app data)         | Low   |
| Office MRU             | Recent documents lists in Office apps           | Medium|
| Jump lists             | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*` | Medium |

### 7.2 Scan Results

```
┌─────────────────────────────────────────────────────────────┐
│  Scan Results — System Scan                                 │
│                                                             │
│  📊 Summary:                                                │
│  Artifacts found: 847                                       │
│  Total size:      2.3 GB                                   │
│  Risk:            High                                      │
│                                                             │
│  ┌────────────┬───────┬────────┬──────────────┬─────────┐  │
│  │ Category   │ Count │ Size   │ Risk Level   │ Action  │  │
│  ├────────────┼───────┼────────┼──────────────┼─────────┤  │
│  │ Temp files │ 423   │ 1.1 GB │ 🟡 Medium    │ ☐ Erase │  │
│  │ Browser    │ 89    │ 890 MB │ 🔴 High      │ ☐ Erase │  │
│  │ Prefetch   │ 156   │ 12 MB  │ 🟢 Low       │ ☐ Erase │  │
│  │ Thumbnails │ 102   │ 245 MB │ 🟡 Medium    │ ☐ Erase │  │
│  │ Recycle Bin│ 47    │ 89 MB  │ 🔴 High      │ ☐ Erase │  │
│  │ Recent     │ 30    │ 2 MB   │ 🟢 Low       │ ☐ Erase │  │
│  └────────────┴───────┴────────┴──────────────┴─────────┘  │
│                                                             │
│  Select All: [☐]   [Erase Selected] [Export Results]        │
└─────────────────────────────────────────────────────────────┘
```

---

## 8. Understanding Verification Results

### 8.1 Verification Levels

| Level       | Description                               | When to Use                |
|-------------|-------------------------------------------|----------------------------|
| None        | No verification performed                 | Quick; lowest assurance    |
| Sample      | Verify random 5% of blocks                | Balance of speed/trust     |
| Full        | Verify 100% of blocks                     | Maximum assurance          |
| Statistical| Analyze residual error rates              | Research / forensics       |

### 8.2 Reading Verification Reports

```
Verification Report
═══════════════════
Operation ID: 550e8400-e29b-41d4-a716-446655440000
Timestamp:    2026-06-20T14:23:45Z
File:         budget_2027.xlsx
Security:     Passes=3, Verify=Full, TRIM=Yes

Pass 1 (Random):     ✓ 45.2 MB written, 45.2 MB verified — PASS
Pass 2 (0x00):       ✓ 45.2 MB written, 45.2 MB verified — PASS
Pass 3 (0xFF):       ✓ 45.2 MB written, 45.2 MB verified — PASS
TRIM:                ✓ 98,304 clusters deallocated
Rename:              ✓ budget_2027.xlsx → K8fJ2mN4pQ3rA9bX.dat
Delete:              ✓ File entry removed

Overall: ✅ PASS — Data is unrecoverable via software means.

Integrity:     HMAC-SHA256 of this record: a1b2c3d4...
Verified by:   Administrator on DESKTOP-ABC123
```

### 8.3 Verification Failure

If verification detects that a block retains old data:

```
⚠️ Verification Warning — Pass 2
Blocks with unexpected data: 3 of 92,304 (0.003%)

Failed blocks:
  LBA 0xA3F7B200: Expected 0x00, read 0x1A (retry: 0x00 ✓)
  LBA 0xA3F7B400: Expected 0x00, read 0x2F (retry: 0x00 ✓)
  LBA 0xA3F7B600: Expected 0x00, read time-out (retry: 0x00 ✓)

Action: Automatic retry succeeded for 3 blocks.
Status: ✅ PASS (after retry)
```

---

## 9. Reports and Logging

### 9.1 Report Types

| Report Type   | Format       | Content                                          |
|---------------|--------------|--------------------------------------------------|
| Operation     | JSON / PDF   | Single erase/scan/sanitize operation details     |
| Session       | JSON / PDF   | All operations in a session                      |
| Audit         | JSON / HTML  | Full audit trail with HMAC chain                 |
| Compliance    | PDF          | Pre-formatted for GDPR/HIPAA/PCI-DSS compliance  |

### 9.2 Accessing Reports

1. Navigate to **Reports** tab
2. Use date range filter and search
3. Click a report to view details
4. Use **Export** button for PDF or JSON

### 9.3 Audit Log Location

```
%APPDATA%\SecDel\secdel.db   — SQLite audit database
%APPDATA%\SecDel\secdel.log  — Application diagnostic log
```

---

## 10. Configuration Reference

### 10.1 Configuration File Location

```
User config:   %APPDATA%\SecDel\config.toml
Policy (admin): %PROGRAMDATA%\SecDel\policy.toml
```

### 10.2 Full Configuration Reference

```toml
# ============================================================================
# SecDel Configuration
# File: %APPDATA%\SecDel\config.toml
# ============================================================================

[security]
# Number of overwrite passes (1-35)
#  1 = Quick (single random pass)
#  3 = Standard (DoD 5220.22-M variant)
#  7 = Maximum (Gutmann variant)
passes = 3

# Verify after each pass
# true  = Read back and compare every block
# false = Skip verification (faster, less assurance)
verify = true

# Percentage of blocks to verify (if verify = true)
# 100 = Full verification
# 5   = Spot-check random 5% of blocks
verify_samples = 100

# Issue TRIM/deallocate after overwrite
# true  = Inform SSD controller to release physical blocks
# false = Skip TRIM (not recommended)
trim_after_erase = true

# Rename file to random name before deletion
# true  = Destroy filename metadata
# false = Keep original name (not recommended)
rename_before_delete = true

# Random temporary name length (characters)
# Longer = more entropy in temporary name
random_name_length = 16

# Scrub directory entry after file deletion
# true  = Overwrite directory entry metadata
# false = Skip directory scrub
scrub_directory_entry = true

[erase]
# Pass pattern preset
# "single"  = One random pass
# "doe"     = DoD 5220.22-M (3-pass)
# "gutmann" = Gutmann (35-pass)
# "nist"    = NIST SP 800-88 (random → 0x00)
# "custom"  = User-defined patterns
pass_pattern = "doe"

# I/O block size in bytes
# 65536   (64 KB)  = Lowest memory, slower
# 1048576 (1 MB)   = Default, good balance
# 4194304 (4 MB)   = Fastest, more memory
block_size = 1048576

# Maximum parallel file erase operations
# Higher = faster batch operations, more I/O contention
parallel_files = 4

# Speed limit in MB/s (0 = unlimited)
# Useful to prevent system unresponsiveness during large operations
speed_limit_mbps = 0

# Fail on first verification error (vs. warn and continue)
fail_on_verify_error = false

[logging]
# Record operations in audit database
audit_log = true

# Diagnostic logging level
# "error"   = Errors only
# "warn"    = Warnings and errors
# "info"    = Normal operations (default)
# "debug"   = Verbose debugging
# "trace"   = Extremely verbose (development only)
log_level = "info"

# Maximum log file size before rotation (in MB)
log_max_size_mb = 50

# Number of rotated log files to keep
log_max_files = 5

[report]
# Default export format
# "pdf"  = Portable Document Format
# "json" = Machine-readable JSON
# "html" = Web-viewable HTML
format = "pdf"

# Include system information in reports (OS version, hardware)
include_system_info = true

# Automatically export report after each operation
auto_export = false

# Export directory (empty = prompt each time)
export_path = ""

[ui]
# Color theme
# "light"  = Light mode
# "dark"   = Dark mode
# "system" = Follow OS setting
theme = "system"

# UI language
# "en" = English
# "de" = German (planned)
# "fr" = French (planned)
# "ja" = Japanese (planned)
language = "en"

# Show confirmation dialog before starting erase
confirm_dialog = true

# Animated progress indicators
progress_animation = true

# Minimize to system tray instead of closing
minimize_to_tray = true

[updates]
# Check for updates on startup
check_on_startup = true

# Update channel
# "stable" = Stable releases only
# "beta"   = Beta releases (may be unstable)
# "nightly" = Daily builds (may be unstable)
channel = "stable"

# Proxy server for updates (empty = system default)
proxy_url = ""

[advanced]
# Enable experimental NVMe passthrough (requires compatible hardware)
enable_nvme_passthrough = true

# Override SSD detection (force TRIM even on non-SSD)
force_trim = false

# Enable development/debug features
developer_mode = false
```

### 10.3 Security Policy (Administrator-Managed)

Administrators can deploy a policy file at `%PROGRAMDATA%\SecDel\policy.toml`:

```toml
# ============================================================================
# SecDel Security Policy (Deployed by Administrator)
# ============================================================================
# This file overrides user configuration for compliance enforcement.
# Users cannot modify settings marked as locked.

[security]
minimum_passes = 3       # Locked: Users cannot reduce below 3
verify_required = true   # Locked: Verification always required
trim_required = true     # Locked: TRIM always required
audit_log_required = true # Locked: Audit logging cannot be disabled
```

---

## 11. Security Level Reference Table

| Level    | Name     | Passes | Verify | TRIM | Rename | Dir Scrub | Use Case              |
|----------|----------|--------|--------|------|--------|-----------|-----------------------|
| 0        | Quick    | 1      | Sample | Yes  | No     | No        | Non-sensitive temp    |
| 1        | Standard | 3      | Full   | Yes  | Yes    | Yes       | General purpose       |
| 2        | Maximum  | 7      | Full   | Yes  | Yes    | Yes       | Highly sensitive      |
| 3        | Paranoid | 35     | Full   | Yes  | Yes    | Yes       | Maximum assurance     |
| Custom   | —        | 1-35   | Config | Config| Config | Config    | User-defined          |

### Security Level vs. Performance

```
Time (relative to Quick)
    ▲
 35x │                          Paranoid
     │
  7x │          Maximum
     │
  3x │  Standard
     │
  1x │  Quick
     └──────────────────────────────────►
        Low            High   Assurance
```

---

## 12. TRIM Technology Explanation

### 12.1 What Is TRIM?

TRIM is an ATA/NVMe command that allows the operating system to inform the SSD controller which logical blocks contain deleted data. The controller can then:

1. Mark the physical NAND blocks as invalid
2. Add them to the garbage collection pool
3. Erase them during idle time or before next write

### 12.2 Why TRIM Matters for Secure Deletion

Without TRIM:
```
You write "AAAA" at LBA 100 ──► SSD writes to physical block 500
You overwrite "BBBB" at LBA 100 ──► SSD writes to physical block 600 (wear leveling!)
"AAAA" still exists at physical block 500 (recoverable!)
```

With TRIM:
```
You overwrite "BBBB" at LBA 100 ──► SSD writes to physical block 600
You issue TRIM for LBA 100 ──► Controller marks physical block 500 as invalid
                             -► Garbage collection eventually erases block 500
```

### 12.3 TRIM Coverage

| Interface | Command              | Synchronous | Immediate Erase |
|-----------|----------------------|-------------|-----------------|
| SATA      | DATA SET MANAGEMENT  | No          | No (deferred)   |
| NVMe      | Deallocate           | Yes         | Yes (most drives)|
| NVMe      | Write Zeroes         | Yes         | Yes             |

### 12.4 Verifying TRIM Success

SecDel can query the SSD's TRIM status via:
- **NVMe**: Get Log Page — Deallocate status
- **SATA**: SMART read for TRIM-completed count
- **Windows**: `FSCTL_FILE_LEVEL_TRIM` return value

---

## 13. SSD Limitations and Realistic Expectations

### 13.1 What SecDel Can and Cannot Do

| ✅ Can Do                                          | ❌ Cannot Do                               |
|----------------------------------------------------|--------------------------------------------|
| Overwrite logical file content with random data    | Guarantee physical NAND overwrite          |
| Issue TRIM/deallocate to SSD controller            | Access over-provisioned spare blocks       |
| Verify that logical blocks contain expected data   | Access SSD-internal DRAM cache             |
| Perform NVMe Sanitize (crypto/block erase)         | Recover data from physically damaged NAND  |
| Rename and delete file entries                     | Delete data in SSD firmware tables         |
| Scan and clean common artifact locations           | Delete cloud sync copies                   |
| Generate HMAC-authenticated audit trail            | Delete backup copies                       |
| Support DoD/NIST compliant deletion procedures     | Prevent forensic physical NAND readout     |

### 13.2 Realistic Recovery Scenarios

| Scenario                                          | Recovery Likelihood | Notes                                |
|---------------------------------------------------|---------------------|--------------------------------------|
| SecDel erase + TRIM, software recovery attempt    | Near-zero           | No software tool can recover         |
| SecDel erase + TRIM, hardware forensic lab        | Low                 | Requires NAND chip readout           |
| SecDel erase (no TRIM), software recovery         | Low-Moderate        | Some data may remain in unallocated  |
| SecDel erase + NVMe Sanitize, any method          | Extremely low       | Cryptographic key destruction        |
| File deleted normally, software recovery          | Very high           | Standard recovery tools work         |
| SecDel erase, VSS copies exist                    | High                | Previous versions recoverable        |
| SecDel erase, cloud sync was active               | High                | Cloud retains copies                 |

### 13.3 Best Practices for Maximum Security

1. **Sanitize entire drives**, not individual files, when decommissioning
2. **Use NVMe Sanitize (crypto)** if your drive supports it
3. **Run as Administrator** for all operations
4. **Disable Volume Shadow Copy** before sanitization:
   ```
   vssadmin delete shadows /all
   vssadmin resize shadowstorage /for=C: /on=C: /maxsize=1GB
   ```
5. **Disconnect from cloud sync** services before deletion
6. **Check for backups** that may contain the data
7. **Use Maximum level** (7 passes) for highly sensitive data
8. **Verify the audit log** after operations

---

## 14. FAQ

### General

**Q: Is SecDel free?**
A: Yes, SecDel is free and open-source software.

**Q: Does SecDel work on HDDs?**
A: Yes, SecDel works on both SSDs and HDDs. For HDDs, overwriting is more reliable since wear leveling is not a concern.

**Q: Does SecDel work on macOS or Linux?**
A: Currently Windows only. Linux and macOS support are planned for future releases.

**Q: Can SecDel recover deleted files?**
A: No. SecDel is designed for secure deletion, not recovery. Use dedicated file recovery tools for undeleting files.

### Technical

**Q: How many passes do I need?**
A: For most users, 3 passes (DoD 5220.22-M variant) with TRIM is sufficient. Additional passes provide diminishing returns on modern SSDs.

**Q: Does SecDel support NVMe drives?**
A: Yes, with full support for NVMe Deallocate and Sanitize commands (depending on drive capabilities).

**Q: What happens if the power goes out during an erase?**
A: SecDel uses atomic state transitions and writes to the audit log before/after each phase. On restart, it can resume or report the incomplete operation.

**Q: Does SecDel work with BitLocker encrypted drives?**
A: Yes, at the file level. For drive-level sanitization, decrypt first or use NVMe Sanitize (which operates below the encryption layer).

**Q: How fast is file deletion?**
A: Typically 200-500 MB/s per pass on modern NVMe SSDs. Speed depends on:
- Drive speed and interface
- Block size setting
- CPU for CSPRNG generation
- Whether verification is enabled

### Security

**Q: Can SecDel data be recovered after deletion?**
A: With TRIM + verification, recovery via software means is impossible. Hardware-based recovery (NAND chip readout) is theoretically possible but requires specialized equipment and expertise.

**Q: Does SecDel leave traces in the Windows Registry?**
A: The installer creates registry entries for Add/Remove Programs. Operation audit logs are stored in `%APPDATA%\SecDel\`. No operation data is stored in the registry.

**Q: Can malware use SecDel to destroy data?**
A: Like any tool that can write to disk, SecDel could be misused. However, SecDel always requires user confirmation, logs all operations, and requires administrator privileges for destructive operations.

**Q: Does SecDel use secure memory handling?**
A: Yes. SecDel uses the `zeroize` crate to clear sensitive memory buffers on drop, preventing data leakage from memory dumps.

---

## 15. Troubleshooting

### 15.1 Common Issues

| Issue                          | Cause                                         | Solution                                          |
|--------------------------------|-----------------------------------------------|---------------------------------------------------|
| "Access denied" error          | Insufficient permissions                      | Run as Administrator                              |
| "File in use" error            | Another process holds the file handle         | Close the file in other apps; use Handle tool     |
| Slow erase speed               | Verify enabled, large block size, slow drive  | Reduce passes, disable verify, increase block size|
| TRIM not supported             | Drive or driver doesn't support TRIM          | Check drive compatibility; update drivers         |
| "Operation cancelled"          | User cancelled or system shutdown             | Resume from audit log; re-run operation           |
| Application won't start        | Missing WebView2 runtime                      | Install WebView2 from Microsoft                   |
| MSI installation fails         | Existing install conflicts                    | Uninstall previous version first                  |
| Report export fails            | Output path not writable                      | Choose a different export directory               |
| NVMe Sanitize fails            | Drive doesn't support the command             | Use fallback: overwrite + TRIM                    |
| Verification shows mismatches  | SSD background operations (wear leveling)     | Re-run verification; usually transient            |

### 15.2 Diagnostic Steps

1. **Check logs**: `%APPDATA%\SecDel\secdel.log`
2. **Review the application logs** for the operation that failed.
3. **Verify TRIM support**:
   ```
   fsutil behavior query DisableDeleteNotify
   ```
   (Result should be `0` for TRIM enabled)

### 15.3 Getting Help

Use the repository's **Issues** page for reproducible bugs and feature requests. Do not post sensitive vulnerability details publicly; follow the process in [SECURITY.md](SECURITY.md) instead.

---

## 16. Glossary

| Term                | Definition                                                   |
|---------------------|--------------------------------------------------------------|
| ATA                 | Advanced Technology Attachment — interface for SATA drives   |
| Audit Trail         | Chronological record of operations with cryptographic integrity|
| Block Size          | Size of data chunks used for I/O operations                  |
| CSPRNG              | Cryptographically Secure Pseudorandom Number Generator       |
| DACL                | Discretionary Access Control List — Windows permission system|
| Deallocate          | NVMe command to release logical blocks (equivalent to TRIM)  |
| DoD 5220.22-M       | US Department of Defense standard for media sanitization     |
| FTL                 | Flash Translation Layer — maps logical to physical NAND addresses|
| GC                  | Garbage Collection — SSD controller's block reclamation process|
| Gutmann Method      | 35-pass overwrite pattern (1996) — largely obsolete for SSDs |
| HMAC                | Hash-based Message Authentication Code                       |
| LBA                 | Logical Block Address — what the OS uses to address storage  |
| MFT                 | Master File Table — NTFS file system metadata structure     |
| NAND                | Type of non-volatile flash memory used in SSDs              |
| NVMe                | Non-Volatile Memory Express — high-speed SSD interface       |
| Over-provisioning   | Reserved SSD capacity not exposed to the OS                  |
| PBA                 | Physical Block Address — actual NAND location                |
| Sanitize            | NVMe command for cryptographic or block-level erase          |
| SATA                | Serial ATA — common interface for SSDs and HDDs              |
| TRIM                | ATA command informing SSD controller of deleted blocks       |
| UAC                 | User Account Control — Windows privilege elevation system    |
| VSS                 | Volume Shadow Copy — Windows backup/restore technology       |
| Wear Leveling       | SSD algorithm distributing writes across all NAND blocks     |
| WebView2            | Microsoft Edge-based runtime for rendering UI                |

---

*This user manual is maintained as part of the SecDel project. For corrections or suggestions, open an issue on GitHub.*
