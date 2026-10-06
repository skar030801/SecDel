# SecDel

SecDel is a Windows desktop application for secure file deletion and storage-sanitization workflows. It combines a React interface with a Rust/Tauri backend and provides deletion progress, verification, device information, artifact scanning, and exportable reports.

> [!CAUTION]
> Secure deletion is destructive. Test the application with non-critical data first, verify the selected paths and devices, and keep backups of anything you need.

## Platform status

SecDel currently targets 64-bit Windows 10 and Windows 11. Some operations require administrator privileges and hardware or driver support.

The effectiveness of deletion depends on the storage medium. In-place overwriting is generally more predictable on magnetic hard drives than on SSDs, where wear levelling, over-provisioning, snapshots, backups, and controller behavior can retain additional copies. For highly sensitive SSD data, prefer a supported device sanitize or cryptographic-erase operation and follow the device manufacturer's guidance.

## Features

- Secure deletion workflows with multiple overwrite levels
- SSD TRIM requests and verification where supported
- Full-drive erase workflows for supported devices
- Scanning for related artifacts such as temporary files and recent-file records
- Device and storage information
- PDF, CSV, and JSON reports
- Light and dark themes
- Local operation without application telemetry

Implementation details and limitations are documented in [Security](docs/SECURITY.md), [Threat model](docs/THREAT_MODEL.md), and [Architecture](docs/ARCHITECTURE.md).

## Build from source

### Prerequisites

- Windows 10 or Windows 11 (64-bit)
- [Node.js](https://nodejs.org/) 22 or later
- [Rust](https://www.rust-lang.org/tools/install) 1.77.2 or later
- Microsoft C++ Build Tools and WebView2, as required by Tauri on Windows

### Development

```powershell
npm ci
npm run tauri dev
```

### Checks

```powershell
npm run lint
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

### Production build

```powershell
npm ci
npm run tauri build
```

Generated installers are written beneath `src-tauri/target/release/bundle/`.

## Documentation

- [User manual](docs/USER_MANUAL.md)
- [Security notes](docs/SECURITY.md)
- [Threat model](docs/THREAT_MODEL.md)
- [Architecture](docs/ARCHITECTURE.md)

## Security

Do not rely on SecDel as the only control for exceptionally sensitive data. Encryption from the start of a device's life, protected key disposal, vendor-supported sanitize commands, and physical destruction may be appropriate depending on the threat model.

Please avoid publishing sensitive vulnerability details in a public issue. See [SECURITY.md](docs/SECURITY.md) for the current security guidance.

## License

Copyright (c) 2026 Shaik Abdul Razak.

This project is licensed under the [MIT License](LICENSE).
