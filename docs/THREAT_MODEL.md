# SecDel Threat Model

> Version 1.0.0 — June 2026
> Classification: Internal — Do Not Distribute

---

## Table of Contents

1. [Asset Definition](#1-asset-definition)
2. [Threat Agents](#2-threat-agents)
3. [Attack Vectors](#3-attack-vectors)
4. [STRIDE Analysis](#4-stride-analysis)
5. [DREAD Scoring](#5-dread-scoring)
6. [Mitigation Strategies](#6-mitigation-strategies)
7. [Residual Risk](#7-residual-risk)
8. [Assumptions and Limitations](#8-assumptions-and-limitations)
9. [Regulatory Compliance Considerations](#9-regulatory-compliance-considerations)

---

## 1. Asset Definition

### 1.1 Primary Assets

| Asset ID | Asset                     | Description                                  | Classification    |
|----------|---------------------------|----------------------------------------------|-------------------|
| A-01     | User data to be deleted   | Files, directories, or drives targeted       | Confidential      |
| A-02     | Deletion proof/audit trail| Cryptographic evidence of secure deletion    | Sensitive         |
| A-03     | Application configuration| Security policies and user preferences       | Internal          |
| A-04     | User authentication state| Admin privilege token or handle              | Restricted        |
| A-05     | Application binary        | Compiled SecDel executable                   | Public            |
| A-06     | Audit log database        | SQLite DB containing operation records       | Sensitive         |

### 1.2 Secondary Assets

| Asset ID | Asset                     | Description                                  | Classification    |
|----------|---------------------------|----------------------------------------------|-------------------|
| A-07     | Temporary files           | Intermediate data during erase operations    | Confidential      |
| A-08     | Log files                 | Application diagnostic logs                  | Internal          |
| A-09     | System metadata           | File timestamps, attributes modified by ops  | Internal          |
| A-10     | Update server             | Server hosting signed update packages        | Critical          |

### 1.3 Asset Criticality Matrix

```
High Criticality
    ┌──────────────────────────────────────────────────┐
    │  A-01 (User Data)     A-04 (Auth Token)          │
    │  A-06 (Audit Log DB)                             │
    ├──────────────────────────────────────────────────┤
    │  A-02 (Deletion Proof) A-10 (Update Server)      │
    ├──────────────────────────────────────────────────┤
    │  A-03 (Config)         A-07 (Temp Files)          │
    │  A-08 (Log Files)     A-09 (System Metadata)     │
    ├──────────────────────────────────────────────────┤
    │  A-05 (App Binary)                                │
Low Criticality
```

---

## 2. Threat Agents

### 2.1 Agent Classification

| Agent ID | Agent                       | Skill Level    | Resources       | Motivation        |
|----------|-----------------------------|----------------|-----------------|-------------------|
| TA-01    | Casual user (accidental)    | Low            | Minimal         | None (accidental) |
| TA-02    | User attempting recovery    | Low-Moderate   | Minimal         | Recover own data  |
| TA-03    | Non-targeted adversary      | Low            | Limited         | Opportunistic     |
| TA-04    | Forensic analyst           | High           | Specialized HW  | Professional      |
| TA-05    | Law enforcement            | Very High      | Significant     | Legal authority   |
| TA-06    | Nation-state actor         | Very High      | Extensive       | Intelligence      |
| TA-07    | Malware on target system   | Moderate-High  | Part of botnet  | Data theft        |
| TA-08    | Insider (malicious admin)  | High           | Internal access | Various           |
| TA-09    | Drive recovery service     | High           | Clean room + HW | Financial         |

### 2.2 Agent Capability Matrix

```
Capability
    ▲
    │  TA-06 (Nation-state)
    │  TA-05 (Law enforcement)
    │  TA-04 (Forensic analyst)  TA-09 (Data recovery)
    │  TA-08 (Insider/admin)
    │       TA-07 (Malware)
    │  TA-02 (User)  TA-03 (Opportunistic)
    │       TA-01 (Accidental)
    └──────────────────────────────────────────────►
        Motivation
```

---

## 3. Attack Vectors

### 3.1 Vector Catalog

| Vector ID | Attack Vector                 | Targets        | Difficulty     |
|-----------|-------------------------------|----------------|----------------|
| V-01      | Physical NAND readout         | A-01           | Very High      |
| V-02      | SSD firmware extraction       | A-01           | Very High      |
| V-03      | Wear-leveling table analysis  | A-01           | High           |
| V-04      | TRIM command interception     | A-01           | Moderate       |
| V-05      | Audit log tampering           | A-02, A-06     | Moderate       |
| V-06      | Configuration file injection  | A-03           | Low            |
| V-07      | IPC command injection         | A-04           | Moderate       |
| V-08      | Man-in-the-middle (update)    | A-10, A-05     | Moderate       |
| V-09      | Malware persistence           | A-07, A-08     | Low            |
| V-10      | Rollback attack               | A-05           | Low            |
| V-11      | Temp file recovery            | A-07           | Low-Moderate   |
| V-12      | Volume Shadow Copy extraction | A-01           | Low            |
| V-13      | Power-loss during erase       | A-01, A-02     | Low-Moderate   |
| V-14      | DLL/COM hijacking             | A-05           | Low            |
| V-15      | Symbolic link attack          | A-01, A-07     | Moderate       |

### 3.2 Attack Tree: Recover Deleted Data

```
Goal: Recover data after SecDel erase
    │
    ├── Physical NAND access [V-01]
    │   ├── Chip-off + NAND reader
    │   └── FTL reverse engineering
    │       ├── Known FTL (publicly documented)
    │       │   └── Success: High (for old SSDs)
    │       └── Proprietary FTL (modern SSDs)
    │           └── Success: Very Low
    │
    ├── SSD firmware exploitation [V-02]
    │   ├── Firmware dump via JTAG/SWD
    │   └── Reverse engineer erase logic
    │       └── Success: Very Low (requires specialized equipment)
    │
    ├── Wear-leveling table analysis [V-03]
    │   ├── Extract LBA-to-PBA mapping
    │   └── Read physical blocks before GC reclaims
    │       └── Success: Low (time-constrained)
    │
    ├── Intercept before erase completes [V-04, V-13]
    │   ├── Power loss during erase [V-13]
    │   │   └── Success: Moderate (depends on timing)
    │   └── TRIM not yet processed [V-04]
    │       └── Success: Moderate (depends on controller)
    │
    ├── Recover from copies [V-12]
    │   ├── Volume Shadow Copy [V-12]
    │   │   └── Success: High (if VSS enabled and not cleaned)
    │   ├── Cloud sync (OneDrive/Dropbox)
    │   │   └── Success: High (if synced)
    │   └── Backup (Windows Backup/File History)
    │       └── Success: High (if backup exists)
    │
    └── Recover from metadata
        ├── Journal (NTFS $UsnJrnl) [partial]
        └── Directory entry (if not overwritten)
            └── Success: Low
```

---

## 4. STRIDE Analysis

### 4.1 STRIDE Per Asset

| Asset   | Spoofing | Tampering | Repudiation | Info Disclosure | DoS | EoP |
|---------|----------|-----------|-------------|-----------------|-----|-----|
| A-01    | L        | M         | L           | H               | L   | M   |
| A-02    | L        | H         | H           | M               | L   | L   |
| A-03    | L        | H         | L           | M               | M   | H   |
| A-04    | H        | H         | M           | H               | L   | H   |
| A-05    | H        | M         | L           | L               | L   | L   |
| A-06    | L        | H         | H           | H               | L   | L   |
| A-07    | L        | M         | L           | H               | L   | M   |
| A-10    | H        | H         | L           | M               | M   | L   |

**Risk Key**: H=High, M=Moderate, L=Low

### 4.2 Detailed STRIDE Threats

#### S — Spoofing

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| S-01      | Attacker spoofs SecDel update server             | H    | A-10     |
| S-02      | Attacker spoofs administrator authentication     | M    | A-04     |
| S-03      | Attacker forges audit log entry provenance       | M    | A-02     |

#### T — Tampering

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| T-01      | Attacker modifies audit log to hide deletion     | H    | A-02,A-06|
| T-02      | Attacker alters config to reduce security passes | H    | A-03     |
| T-03      | Attacker swaps SecDel binary with trojan         | H    | A-05     |
| T-04      | Attacker modifies temporary files during erase   | M    | A-07     |

#### R — Repudiation

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| R-01      | User denies initiating a deletion operation      | H    | A-02     |
| R-02      | Attacker performs deletion without trace         | H    | A-06     |
| R-03      | System claims erase success but data remains     | M    | A-02     |

#### I — Information Disclosure

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| I-01      | Audit log reveals deleted file names/metadata    | H    | A-06     |
| I-02      | Temporary files recovered before cleanup         | H    | A-07     |
| I-03      | Config reveals security policy details           | M    | A-03     |
| I-04      | Log files contain sensitive paths                | M    | A-08     |

#### D — Denial of Service

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| D-01      | Attacker exhausts disk with unnecessary erases   | L    | A-01     |
| D-02      | Corrupt config prevents application launch       | M    | A-03     |
| D-03      | Update server DoS prevents security patches      | M    | A-10     |

#### E — Elevation of Privilege

| Threat ID | Description                                      | Risk | Affected |
|-----------|--------------------------------------------------|------|----------|
| E-01      | IPC command injection leads to admin operations  | H    | A-04     |
| E-02      | Config tampering leads to relaxed security       | M    | A-03     |
| E-03      | Malware leverages SecDel to destroy data         | M    | A-01     |

---

## 5. DREAD Scoring

### 5.1 DREAD Methodology

Each threat is scored 1-10 on five dimensions:

| Dimension  | Description                          |
|------------|--------------------------------------|
| **D**amage | Potential damage if attack succeeds  |
| **R**eproducibility| How easy to reproduce attack  |
| **E**xploitability| How easy to execute the attack |
| **A**ffected users | How many users impacted       |
| **D**iscoverability| How easy to discover weakness  |

**Score**: Sum / 5 → Rating

### 5.2 DREAD Ratings

| Threat ID | Description                   | D  | R  | E  | A  | D  | Total | Risk    |
|-----------|-------------------------------|----|----|----|----|----|-------|---------|
| T-02      | Config tampering              | 9  | 7  | 6  | 9  | 8  | 7.8   | **High**|
| T-01      | Audit log tampering           | 8  | 6  | 5  | 9  | 7  | 7.0   | **High**|
| T-03      | Binary substitution           | 10 | 4  | 4  | 9  | 5  | 6.4   | **High**|
| E-01      | IPC command injection         | 9  | 5  | 5  | 8  | 8  | 7.0   | **High**|
| I-01      | Audit log disclosure          | 8  | 8  | 7  | 7  | 8  | 7.6   | **High**|
| I-02      | Temp file recovery            | 7  | 8  | 7  | 8  | 9  | 7.8   | **High**|
| R-01      | Repudiation of delete         | 6  | 8  | 6  | 7  | 7  | 6.8   | **High**|
| V-12      | VSS copy recovery             | 7  | 9  | 9  | 8  | 9  | 8.4   | **Critical**|
| S-01      | Update server spoof           | 9  | 3  | 3  | 9  | 4  | 5.6   | Moderate |
| D-02      | Corrupt config DoS            | 4  | 6  | 5  | 6  | 6  | 5.4   | Moderate |
| V-13      | Power loss during erase       | 5  | 7  | 6  | 7  | 8  | 6.6   | Moderate |
| V-11      | Temp file recovery (low-skill)| 6  | 9  | 8  | 6  | 9  | 7.6   | **High** |
| V-15      | Symlink attack                | 8  | 4  | 4  | 5  | 6  | 5.4   | Moderate |

**Risk Thresholds**:
- **Critical**: 8.0 – 10.0
- **High**: 6.0 – 7.9
- **Moderate**: 4.0 – 5.9
- **Low**: 1.0 – 3.9

---

## 6. Mitigation Strategies

### 6.1 Mitigation by Threat

| Threat ID | Mitigation Strategy                                         | Status         |
|-----------|-------------------------------------------------------------|----------------|
| T-02      | Config file integrity check (HMAC-signed)                   | Implemented    |
| T-01      | Audit log HMAC chain; append-only with monotonic counter    | Implemented    |
| T-03      | Code signing; installer hash verification                   | Implemented    |
| E-01      | Input validation; Tauri capability whitelist                | Implemented    |
| I-01      | Audit log encryption at rest; file metadata hashing         | Planned v1.1   |
| I-02      | Secure temp file creation; immediate overwrite on close     | Implemented    |
| R-01      | HMAC-authenticated audit log with user identity tagging     | Implemented    |
| V-12      | Warn user about VSS; provide cleanup utility                | Implemented    |
| S-01      | Code signing; TLS 1.3; certificate pinning                  | Implemented    |
| D-02      | Config schema validation; safe defaults on corruption       | Implemented    |
| V-13      | Audit log resume; atomic rename-on-complete                 | Implemented    |
| V-11      | Exclusive access; shadow copy prevention                    | Implemented    |
| V-15      | Canonical path resolution; deny symlink targets             | Implemented    |

### 6.2 Defense Layers

```
Layer 1: Input Validation
├── All IPC command parameters validated server-side
├── Path canonicalization (resolve .., symlinks, junctions)
├── Type checking for all parameters
└── Bounds checking for arrays/buffers

Layer 2: Access Control
├── Privilege check before any operation
├── File handle ACL verification
├── Tauri capability whitelist enforcement
└── Elevation token scoping

Layer 3: Cryptographic Integrity
├── Audit log HMAC chain (SHA-256 HMAC per entry)
├── Config file HMAC verification on load
├── Update package signature verification
└── Verification block hash comparison

Layer 4: Secure State Management
├── Atomic file operations
├── Power-loss safe state transitions
├── Cancellation token support
└── Graceful degradation on error

Layer 5: Audit Trail
├── Immutable append-only audit log
├── All operations timestamped and signed
├── Integrity check on every read
└── Export with embedded verification
```

---

## 7. Residual Risk

### 7.1 Accepted Risks

| Risk ID | Description                                               | Rationale for Acceptance                  |
|---------|-----------------------------------------------------------|-------------------------------------------|
| R-01    | Physical NAND readout with electron microscope            | Cost > $100K; nation-state only           |
| R-02    | SSD firmware-level data retention                         | No practical mitigation without OEM access|
| R-03    | Over-provisioned block data persistence                   | Requires proprietary FTL knowledge        |
| R-04    | Bad block remapped sector recovery                        | No API to access remapped sectors         |
| R-05    | SSD DRAM cache data retention after power loss            | Cache volatile; clears within seconds     |
| R-06    | Cloud sync service copies not deleted                     | Out of SecDel scope; user responsibility  |
| R-07    | Backup copies not deleted                                 | Out of SecDel scope; user responsibility  |
| R-08    | Malware installed after secure deletion                   | OS-level protection; SecDel cannot prevent|

### 7.2 Residual Risk Matrix

```
Likelihood
    High
    │  R-06 (Cloud copies)              
    │  R-07 (Backups)                   
    │                                   
    │       R-05 (DRAM cache)           
    │                                   
    │  R-03 (OP blocks)  R-04 (Bad blocks)
    │       R-01 (NAND readout)   R-02 (Firmware)
    └────────────────────────────────────────────►
    Low                          High   Impact
```

---

## 8. Assumptions and Limitations

### 8.1 Assumptions

1. **Operating system integrity**: The host OS is not compromised. SecDel cannot guarantee secure deletion on a system with kernel-level malware.
2. **Hardware trust**: The SSD controller is assumed to correctly execute standard commands (TRIM, Sanitize, SECURE ERASE).
3. **User competence**: The user understands that deleted files may exist in backups, cloud sync, or shadow copies.
4. **Physical security**: SecDel does not protect against physical attacks (chip-off, electron microscopy) on the raw NAND.
5. **SSD compliance**: The SSD correctly implements ATA/NVMe command set specifications.
6. **File system integrity**: NTFS/ext4/Btrfs file system metadata operations proceed as expected.

### 8.2 Limitations

1. **No guarantee of physical overwrite**: Due to wear leveling, logical overwrite may not correspond to physical overwrite.
2. **Over-provisioned space**: SSD spare area inaccessible through standard commands.
3. **Bad block remapping**: Remapped sectors may retain original data indefinitely.
4. **Journaling file systems**: Metadata may persist in journals (NTFS $LogFile, ext4 journal).
5. **Windows Volume Shadow Copy**: Previous file versions may persist unless explicitly deleted.
6. **File system compression/encryption**: Encrypted/compressed filesystem behavior during overwrite is implementation-dependent.
7. **Real-time protection interference**: Windows Defender or other AV may hold file handles during operation.
8. **Power loss**: Mid-erase power loss may leave partial data; recovery mechanism provided but not guaranteed.

---

## 9. Regulatory Compliance Considerations

### 9.1 GDPR (General Data Protection Regulation)

| Requirement                       | SecDel Compliance                              |
|-----------------------------------|------------------------------------------------|
| Right to erasure (Art. 17)        | ✅ Secure deletion provides technical means    |
| Data minimization (Art. 5)        | ✅ No telemetry; local-only audit              |
| Security of processing (Art. 32)  | ✅ Cryptographic erase; verified               |
| Data breach notification (Art. 33)| ✅ Audit log provides breach evidence          |
| Records of processing (Art. 30)   | ✅ Complete audit trail                        |
| Privacy by design (Art. 25)       | ✅ Security-first architecture                 |

**GDPR Note**: Secure deletion fulfills the "right to erasure" (right to be forgotten) requirement when data is properly deleted with verification. SecDel provides the technical mechanism; organizational policies must ensure consistent application.

### 9.2 HIPAA (Health Insurance Portability and Accountability Act)

| Requirement                       | Standard                                      | SecDel Compliance |
|-----------------------------------|-----------------------------------------------|-------------------|
| Disposal standard (45 CFR §164.310)| Clearance, purging, destruction              | ✅ Verified erasure|
| Media reuse (45 CFR §164.310)     | Clearance prior to reuse                      | ✅ Full drive wipe |
| Backup disposal                   | Same standards as primary                     | ✅ Supports        |
| Audit controls (45 CFR §164.312)  | Record access and deletion                    | ✅ Audit log       |
| Integrity controls (45 CFR §164.312)| Protect PHI from improper alteration       | ✅ HMAC chain      |

**HIPAA Note**: SecDel's verification step provides evidence that PHI has been properly disposed. The audit trail supports compliance documentation. Organizations should follow NIST SP 800-88 guidelines for media sanitization.

### 9.3 PCI-DSS (Payment Card Industry Data Security Standard)

| Requirement                       | SecDel Compliance                              |
|-----------------------------------|------------------------------------------------|
| Cardholder data disposal (Req 3.1)| Render unrecoverable                           | ✅ Verified erase  |
| Periodic security testing (Req 11)| Audit trail provides evidence                  | ✅ Reports         |
| Access control (Req 7)            | Admin elevation for sanitization               | ✅ Privilege model |
| Logging (Req 10)                  | Complete audit trail                           | ✅ HMAC-chained    |

**PCI-DSS Note**: SecDel with DoD 3-pass + TRIM meets PCI-DSS data disposal requirements. NVMe Sanitize (crypto) provides the highest assurance level for decommissioning PCI-DSS media.

### 9.4 Additional Standards

| Standard                    | Relevance                                      |
|-----------------------------|------------------------------------------------|
| NIST SP 800-88 Rev 1       | Media sanitization guidelines                  |
| NIST SP 800-53 Rev 5       | Security controls (AU, SC, SI families)        |
| ISO/IEC 27001:2022          | Information security management               |
| ISO/IEC 27040:2015          | Storage security                              |
| DoD 5220.22-M (ECE)         | National Industrial Security Program          |
| BSI TR-03132               | Secure deletion (German standard)             |
| OWASP ASVS v4.0            | Application security verification             |

### 9.5 Compliance Recommendations

1. **For GDPR compliance**: Use minimum 3-pass erase + verification + audit logging. Export audit reports for regulatory review.
2. **For HIPAA compliance**: Use verified 3-pass erase for all ePHI media. Retain audit logs per record retention policy.
3. **For PCI-DSS compliance**: Use NVMe Sanitize (crypto) for cardholder data environments. Run quarterly sanitization verification.
4. **For mixed compliance**: Use highest common denominator (3-pass verified + TRIM + audit). Policy configuration can enforce minimum standards.

---

## References

1. NIST SP 800-88 Rev 1 — Guidelines for Media Sanitization
2. NIST SP 800-53 Rev 5 — Security and Privacy Controls
3. Gutmann, P. (1996). Secure Deletion of Data from Magnetic and Solid-State Memory
4. Wei, M. et al. (2011). Reliably Erasing Data from Flash-Based Solid State Drives
5. DoD 5220.22-M — National Industrial Security Program Operating Manual
6. ATA/ATAPI Command Set — ACS-4
7. NVM Express Base Specification, Rev 2.0
8. OWASP Application Security Verification Standard v4.0

---

*This threat model document is a living document maintained as part of the SecDel project. Review and update frequency: quarterly or on major feature changes.*
