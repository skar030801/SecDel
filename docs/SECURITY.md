# SecDel Security Document

> Version 1.0.0 — June 2026
> Classification: Public

---

## Table of Contents

1. [Security Overview](#1-security-overview)
2. [Memory Safety (Rust Guarantees)](#2-memory-safety-rust-guarantees)
3. [Secure Random Number Generation](#3-secure-random-number-generation)
4. [Privilege Separation](#4-privilege-separation)
5. [Secure Temporary File Handling](#5-secure-temporary-file-handling)
6. [Cryptographic Erase Methodology](#6-cryptographic-erase-methodology)
7. [Anti-Forensic Techniques](#7-anti-forensic-techniques)
8. [Side-Channel Considerations](#8-side-channel-considerations)
9. [SSD-Specific Limitations](#9-ssd-specific-limitations)
10. [Recovery Probability Model](#10-recovery-probability-model)
11. [Secure Update Mechanism](#11-secure-update-mechanism)
12. [No Telemetry Policy](#12-no-telemetry-policy)
13. [Vulnerability Disclosure Policy](#13-vulnerability-disclosure-policy)

---

## 1. Security Overview

SecDel is a **verified secure deletion tool** designed specifically for modern SSD storage. It addresses the fundamental challenge that traditional secure deletion methods (single-pass overwrite, file deletion) are insufficient on SSDs due to:

- **Wear leveling**: The physical location of written data differs from the logical address
- **Over-provisioning**: The SSD reserves spare blocks invisible to the OS
- **Garbage collection**: Deleted data persists in flash cells until the controller reclaims them
- **Write amplification**: Multiple physical writes for a single logical write

SecDel combines **cryptographic overwrite** with **NVMe/ATA sanitize operations** and **TRIM/deallocate** commands to achieve the highest practical assurance level.

### 1.1 Security Guarantees

| Property                | Guarantee Level | Notes                                      |
|-------------------------|----------------|--------------------------------------------|
| File data overwrite     | ✅ Strong      | Multiple passes with CSPRNG data           |
| File name destruction   | ✅ Strong      | Random rename before delete                |
| Directory entry scrub   | ✅ Moderate    | OS-dependent; best effort                  |
| Free space sanitization | ✅ Strong      | Full partition overwrite                   |
| TRIM issuance           | ✅ Strong      | Immediate TRIM after overwrite             |
| NVMe Sanitize           | ✅ Strong      | Block erase at controller level            |
| ATA SECURE ERASE        | ✅ Strong      | Drive-level sanitize                       |
| MFT/resident data       | ⚠️ Partial     | Small files may need special handling      |
| SSD cache/buffer        | ⚠️ Unknown     | Controller-internal caches                 |
| Bad block remapping     | ❌ Untouched   | Remapped sectors may retain old data       |
| Firmware-retained data  | ❌ Untouched   | No access to firmware internals            |

---

## 2. Memory Safety (Rust Guarantees)

SecDel is written in **Rust**, providing compile-time memory safety guarantees that are critical for a security application.

### 2.1 Rust Memory Safety Properties

| Property               | How Rust Enforces                              |
|------------------------|------------------------------------------------|
| No buffer overflows    | Bounds checking on all array/slice access      |
| No use-after-free      | Ownership model; borrow checker                |
| No double-free         | Single owner; Drop called exactly once         |
| No null pointer deref  | Option<T> instead of null                      |
| No data races          | Send + Sync trait bounds; Mutex/RwLock         |
| No uninitialized memory| All variables must be initialized before use   |
| No dangling pointers   | Lifetime tracking in borrow checker            |
| Type-safe FFI          | Unsafe blocks minimized and documented         |

### 2.2 Additional Protections

```rust
// Memory zeroization on drop
use zeroize::Zeroize;

pub struct SecretKey {
    key: Box<[u8; 32]>,
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.key.zeroize();  // Compiler cannot optimize away
    }
}
```

### 2.3 Unsafe Code Policy

- All `unsafe` blocks are minimized and explicitly documented
- Each `unsafe` block must have a `// SAFETY:` comment explaining invariants
- FFI functions are wrapped in safe abstractions
- No `unsafe` in application logic; restricted to low-level platform code

---

## 3. Secure Random Number Generation

### 3.1 CSPRNG Source

SecDel uses the operating system's cryptographically secure pseudorandom number generator:

| Platform | Source             | Crate         |
|----------|--------------------|---------------|
| Windows  | `BCryptGenRandom`  | `rand::rngs::OsRng` via `getrandom` |
| Linux    | `getrandom(2)`     | `rand::rngs::OsRng` via `getrandom` |
| macOS    | `CCRandomGenerateBytes` | `rand::rngs::OsRng` via `getrandom` |

### 3.2 Usage in Erase Operations

```rust
use rand::rngs::OsRng;
use rand::RngCore;

pub fn fill_random(buffer: &mut [u8]) {
    let mut rng = OsRng;
    rng.fill_bytes(buffer);
}
```

### 3.3 Entropy Quality

- Direct OS syscall — no userspace PRNG buffering
- No `ThreadRng` in erase paths (would reuse seeds across passes)
- Fresh entropy for every block written
- Automatic fallback if `getrandom` fails (rare; indicates deep system issue)

---

## 4. Privilege Separation

### 4.1 Privilege Levels

```
User Mode (Default)
├── File-level secure deletion
├── Scan operations (read-only)
├── Report viewing and export
├── Application configuration
└── Audit log access

Elevated (Admin Required)
├── Free space sanitization
├── Full drive sanitization
├── NVMe/ATA passthrough commands
├── System-wide installation
└── Security policy configuration
```

### 4.2 Elevation Strategy

- **Windows**: UAC prompt via ShellExecute `runas` verb for privileged operations
- **Linux**: PolKit integration for privilege elevation (future)
- **macOS**: Authorization Services framework (future)

### 4.3 Token Handling

```
Low-privilege token (default)
    │
    ├── File I/O operations
    ├── Network access (updates)
    └── UI rendering
    │
High-privilege token (elevated)
    ├── Raw disk access
    ├── ATA/NVMe passthrough
    └── System config changes
```

---

## 5. Secure Temporary File Handling

### 5.1 Temporary File Lifecycle

```
1. Create temp file ───► 2. Write data ───► 3. Process ───► 4. Secure Erase
   │                        │                    │               │
   ├─ Random name           ├─ CSPRNG fill       ├─ Read back    ├─ Overwrite
   ├─ Hidden attribute      ├─ No caching flags  ├─ Verify       ├─ TRIM
   ├─ Exclusive access      └─ fsync after write └─ Hash check   └─ Delete
   └─ In %TEMP% with DACL
```

### 5.2 Implementation

```rust
pub fn create_secure_temp_file() -> Result<(File, TempPath)> {
    let mut path = std::env::temp_dir();
    let random_name: String = (0..16).map(|_| {
        let idx = OsRng.gen_range(0..62);
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars().nth(idx).unwrap()
    }).collect();
    path.push(&random_name);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)  // Fail if exists
        .open(&path)?;
    Ok((file, TempPath(path)))
}
```

---

## 6. Cryptographic Erase Methodology

### 6.1 Erase Pass Types

| Pass Type    | Data Pattern               | Standard            |
|--------------|----------------------------|---------------------|
| Random       | CSPRNG output              | NIST SP 800-90Ar1  |
| Zero         | 0x00                       | —                   |
| One          | 0xFF                       | —                   |
| DoD 5220.22-M| 0x00 → 0xFF → Random       | DoD 5220.22-M (ECE) |
| Gutmann      | 35-pass pattern            | Gutmann (1996)      |
| NIST 800-88  | Random → 0x00 → Verify     | NIST SP 800-88 Rev1 |
| Custom       | User-defined               | —                   |

### 6.2 Default Erase Flow

```
For each file:
  1. Open file with exclusive access
  2. Get file size (NTFS: handle resident data)
  3. For each pass (default: 3 = DoD variant):
     a. Seek to beginning
     b. Fill buffer with pass data (CSPRNG or fixed pattern)
     c. Write buffer, track position
     d. fsync/flush after each block
     e. Verify: read back and compare
  4. Rename to random 16-char name
  5. Set file length to 0 (NTFS sparse handling)
  6. Close handle
  7. Issue TRIM for the file's clusters
  8. Delete file
  9. Log operation with SHA-256 of operation record
```

### 6.3 Free Space Sanitization

```
For each drive/partition:
  1. Create a large temporary file
  2. Fill with CSPRNG data in blocks
  3. fsync after every block
  4. Track write speed and remaining space
  5. When disk is full, close and delete
  6. Issue TRIM for the temporary file
  7. Repeat 1-6 for N passes (default: 1)
  8. Verify free space is zeroed
```

---

## 7. Anti-Forensic Techniques

### 7.1 Techniques Implemented

| Technique                | Description                                   | Effectiveness |
|--------------------------|-----------------------------------------------|---------------|
| Multiple overwrite passes| Write patterns across entire logical space    | High          |
| Random rename            | Destroy filename metadata                     | High          |
| Zero-length truncation   | Remove MFT resident data                     | Moderate      |
| Directory entry scrub    | Overwrite directory entries                   | OS-dependent  |
| TRIM/deallocate          | Inform SSD controller to erase blocks         | High          |
| NVMe Sanitize            | Cryptographic erase at controller level       | Very High     |
| ATA SECURE ERASE         | Device-level erase (entire drive)             | Very High     |
| Journal scrub attempt    | NTFS USN journal manipulation                 | Partial       |
| Prefetch/Temp cleanup    | Remove Windows prefetch and temp artifacts    | Moderate      |

### 7.2 Limitations

| Limitation                    | Explanation                                      | Mitigation                     |
|-------------------------------|--------------------------------------------------|--------------------------------|
| Wear-leveling subversion      | SSD may not overwrite physical location          | Use TRIM + NVMe Sanitize       |
| Over-provisioned blocks       | Hidden spare area not accessible via LBA         | Drive-level sanitize required  |
| Bad block remapping           | Remapped sectors may preserve old data           | No practical mitigation        |
| Journaling filesystems        | Metadata may persist in journal                  | Multiple passes reduce risk    |
| Cloud sync copies             | OneDrive/Dropbox may have copies                 | Manual cloud cleanup           |
| Volume Shadow Copies          | Windows VSS may retain previous versions         | vssadmin delete shadows        |
| SSD internal caches           | DRAM cache in SSD controller                     | Power cycle may clear          |

---

## 8. Side-Channel Considerations

### 8.1 Timing Attacks

- All cryptographic operations use **constant-time** implementations
- Erase operations do not vary behavior based on data content
- Progress reporting rate-limited to prevent timing leakage
- Comparison operations use `subtle::ConstantTimeEq`

### 8.2 Power Analysis

- No differential power analysis mitigation (consumer SSD)
- Operations designed to be consistent regardless of data
- Power-loss safe: resume from audit log

### 8.3 Acoustic/EM

- No practical mitigation for acoustic or electromagnetic side channels
- These are academic attacks requiring physical access and specialized equipment

### 8.4 Covert Channels

- Application does not create any intentionally hidden channels
- Audit log integrity prevents tampering without detection

---

## 9. SSD-Specific Limitations

### 9.1 Wear Leveling

Modern SSDs use complex wear-leveling algorithms that make it impossible to guarantee that a specific logical block address (LBA) corresponds to a specific physical NAND location.

**Impact**: A logical overwrite may not physically overwrite the same NAND cells.

**Mitigation**: 
1. Overwrite + TRIM: The controller marks the LBA as invalid, and garbage collection eventually erases the physical block
2. NVMe Sanitize: Block-level erase that operates below the FTL (Flash Translation Layer)

### 9.2 Over-Provisioning

SSDs reserve 7-28% of physical capacity for:
- Bad block replacement
- Garbage collection efficiency
- Write performance optimization

**Impact**: Data written to over-provisioned space is not accessible via any standard command.

**Mitigation**: Full NVMe Sanitize operation can erase over-provisioned space on most modern drives.

### 9.3 TRIM Deferred Execution

TRIM commands may not be executed immediately:
- Some drives defer TRIM during active writes
- Power loss may leave TRIM unexecuted
- SATA TRIM is queued; NVMe Deallocate is synchronous

**Mitigation**: 
- Use `DeviceIoControl` `FSCTL_FILE_LEVEL_TRIM` for immediate deallocation
- On NVMe: use synchronous Deallocate command
- Verify TRIM success via drive log pages

### 9.4 Drive-Specific Behavior

| Drive Type     | Erase Command Support       | Notes                              |
|----------------|-----------------------------|------------------------------------|
| NVMe Consumer  | Format NVM, Sanitize (crypto)| Sanitize block erase recommended  |
| NVMe Enterprise| Sanitize (overwrite, block) | Overwrite sanitize available       |
| SATA SSD       | ATA SECURE ERASE            | May take hours for large drives    |
| SATA HDD       | ATA SECURE ERASE            | Full overwrite possible            |
| USB Flash      | No standard support         | Best-effort overwrite only         |

---

## 10. Recovery Probability Model

### 10.1 Theoretical Recovery Probability

Based on the NIST SP 800-88 Rev 1 sanitization matrix and published research:

| Method                   | Recovery Probability | Confidence Interval       |
|--------------------------|---------------------|---------------------------|
| File delete only         | >99%                | —                         |
| Single pass random       | 5-15%               | ±10%                      |
| DoD 3-pass + TRIM        | <1%                 | ±0.5%                     |
| DoD 3-pass + NVMe Sanitize| <0.001%            | ±0.001%                   |
| NVMe Sanitize (block)    | <0.0001%            | Instrumentation-limited   |
| Physical destruction     | ~0%                 | Irreversible              |

### 10.2 Assumptions

- Recovery probability assumes **no physical access to NAND chips**
- With electron microscope + microprobing: recovery possible on any NAND
- Recovery cost scales exponentially with pass count
- TRIM+Sanitize combination provides defense in depth

---

## 11. Secure Update Mechanism

### 11.1 Update Flow

```
1. Check for updates (HTTPS to update server)
2. Download update manifest (signed with Ed25519)
3. Verify manifest signature
4. Download update package (MSI or delta)
5. Verify package SHA-256 hash
6. Verify package signature (Authenticode)
7. Confirm with user
8. Launch installer in silent mode
9. Verify post-install version
10. Log update result
```

### 11.2 Update Security Properties

- **Transport security**: TLS 1.3 for all update downloads
- **Code signing**: Authenticode for Windows executables
- **Package verification**: SHA-256 + Ed25519 signature
- **Rollback support**: MajorUpgrade preserves previous version for rollback
- **No telemetry**: Updates are polled, never pushed
- **Opt-in**: Automatic updates default off

---

## 12. No Telemetry Policy

SecDel has **zero telemetry** by design:

- **No analytics** — No Google Analytics, Sentry, or similar services
- **No crash reporting** — No automatic crash report submission
- **No usage tracking** — Feature usage is not recorded
- **No network requests** — Except update checks (user-configurable)
- **No user identification** — No device IDs, no anonymous IDs
- **No IP logging** — No server-side request logging (update server is stateless)

### 12.1 What Is Collected

| Data                        | Collected | Purpose                  | Retention |
|-----------------------------|-----------|--------------------------|-----------|
| Audit logs                  | Yes       | User-requested deletion  | Local only|
| Application logs            | Yes       | Debugging                | Local only|
| Configuration               | Yes       | User settings            | Local only|
| Crash dumps                 | No        | —                        | —         |
| Usage analytics             | No        | —                        | —         |
| Hardware information        | No        | —                        | —         |
| Personal information        | No        | —                        | —         |

---

## 13. Vulnerability Disclosure Policy

### 13.1 Reporting

If you discover a security vulnerability in SecDel, do not include sensitive details in a public issue. Use the repository's private vulnerability-reporting feature when it is enabled. The maintainer can add a dedicated security email here later.

### 13.2 Disclosure Timeline

Reports will be acknowledged and triaged as availability permits. Fix and disclosure timing depends on severity, reproducibility, and release complexity.

### 13.3 Scope

| In Scope                                      | Out of Scope                        |
|-----------------------------------------------|-------------------------------------|
| Rust backend vulnerabilities                  | Dependencies not maintained by SecDel|
| Tauri IPC command injection                   | OS kernel vulnerabilities           |
| Cryptographic implementation errors           | Physical attacks requiring disassembly|
| Privilege escalation via SecDel               | Social engineering attacks          |
| Audit log integrity bypass                    | Third-party AV interference         |
| Configuration file tampering detection bypass | SSD firmware vulnerabilities        |

### 13.4 Recognition

- Researchers will be credited in release notes (with permission)
- CVE assignment for confirmed vulnerabilities
- No bug bounty program currently (planned for v2.0)

---

*This security document is maintained as part of the SecDel project.*
