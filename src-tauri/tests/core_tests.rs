#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_overwrite_file_random() {
        let dir = std::env::temp_dir().join("secdel_test_overwrite");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test.bin");

        fs::write(&file_path, &[0xABu8; 4096]).expect("Failed to create test file");

        let config = secdel_lib::core::overwrite::OverwriteConfig {
            passes: 1,
            verify: false,
            pattern_type: secdel_lib::core::overwrite::PatternType::Random,
        };

        let result = secdel_lib::core::overwrite::overwrite_file(&file_path, &config);
        assert!(
            result.is_ok(),
            "Overwrite should succeed: {:?}",
            result.err()
        );
        let result = result.unwrap();
        assert_eq!(result.passes_completed, 1, "Should complete 1 pass");
        assert!(
            !file_path.exists(),
            "File should be deleted after overwrite"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_overwrite_file_zero() {
        let dir = std::env::temp_dir().join("secdel_test_zero");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test_zero.bin");

        fs::write(&file_path, &[0xFFu8; 1024]).expect("Failed to create test file");

        let config = secdel_lib::core::overwrite::OverwriteConfig {
            passes: 1,
            verify: false,
            pattern_type: secdel_lib::core::overwrite::PatternType::Zero,
        };

        let result = secdel_lib::core::overwrite::overwrite_file(&file_path, &config);
        assert!(result.is_ok(), "Overwrite should succeed");
        assert!(!file_path.exists(), "File should be deleted");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_overwrite_file_not_found() {
        let config = secdel_lib::core::overwrite::OverwriteConfig {
            passes: 1,
            verify: false,
            pattern_type: secdel_lib::core::overwrite::PatternType::Random,
        };

        let result = secdel_lib::core::overwrite::overwrite_file(
            Path::new("C:\\nonexistent_file_xyz123.bin"),
            &config,
        );
        assert!(result.is_err(), "Should fail for non-existent file");
    }

    #[test]
    fn test_overwrite_empty_file() {
        let dir = std::env::temp_dir().join("secdel_test_empty");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("empty.bin");

        fs::write(&file_path, &[]).expect("Failed to create empty file");

        let config = secdel_lib::core::overwrite::OverwriteConfig {
            passes: 3,
            verify: true,
            pattern_type: secdel_lib::core::overwrite::PatternType::Random,
        };

        let result = secdel_lib::core::overwrite::overwrite_file(&file_path, &config);
        assert!(result.is_ok(), "Overwrite of empty file should succeed");
        let result = result.unwrap();
        assert_eq!(
            result.bytes_written, 0,
            "Bytes written should be 0 for empty file"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_overwrite_multiple_passes() {
        let dir = std::env::temp_dir().join("secdel_test_multi");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("multi.bin");

        fs::write(&file_path, &[0x42u8; 8192]).expect("Failed to create test file");

        let config = secdel_lib::core::overwrite::OverwriteConfig {
            passes: 3,
            verify: false,
            pattern_type: secdel_lib::core::overwrite::PatternType::Random,
        };

        let result = secdel_lib::core::overwrite::overwrite_file(&file_path, &config);
        assert!(result.is_ok(), "Multi-pass overwrite should succeed");
        assert!(
            !file_path.exists(),
            "File should be deleted after multi-pass overwrite"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_trim_execute() {
        let dir = std::env::temp_dir().join("secdel_test_trim");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("trim_test.bin");

        fs::write(&file_path, &[0x00u8; 65536]).expect("Failed to create test file");

        let result = secdel_lib::core::trim::execute_trim(&file_path);
        assert!(result.is_ok(), "TRIM should execute without error");
        let op = result.unwrap();
        assert!(
            matches!(
                op.status,
                secdel_lib::core::trim::TrimStatus::Completed
                    | secdel_lib::core::trim::TrimStatus::Unsupported
            ),
            "TRIM should complete or report unsupported"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_trim_nonexistent() {
        let result =
            secdel_lib::core::trim::execute_trim(Path::new("C:\\nonexistent_trim_test.bin"));
        assert!(result.is_ok(), "TRIM on non-existent should not error");
        let op = result.unwrap();
        assert!(matches!(
            op.status,
            secdel_lib::core::trim::TrimStatus::Unsupported
        ));
    }

    #[test]
    fn test_verify_deletion() {
        let dir = std::env::temp_dir().join("secdel_test_verify");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("verify_test.bin");

        fs::write(&file_path, &[0x00u8; 1024]).expect("Failed to create test file");

        let result = secdel_lib::core::verify::verify_file_deletion(&file_path);
        assert!(result.is_ok());

        let vr = result.unwrap();
        assert!(!vr.deleted, "File should exist before deletion");
        assert_eq!(vr.bytes_remaining, 1024, "Should report original size");

        fs::remove_file(&file_path).ok();

        let result = secdel_lib::core::verify::verify_file_deletion(&file_path);
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(vr.deleted, "File should be reported as deleted");
        assert_eq!(vr.bytes_remaining, 0, "No bytes remaining after deletion");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_file_size_human() {
        assert_eq!(secdel_lib::utils::file_size_human(0), "0.00 B");
        assert_eq!(secdel_lib::utils::file_size_human(1024), "1.00 KB");
        assert_eq!(secdel_lib::utils::file_size_human(1048576), "1.00 MB");
        assert_eq!(secdel_lib::utils::file_size_human(1073741824), "1.00 GB");
    }

    #[test]
    fn test_timestamp_iso8601() {
        let ts = secdel_lib::utils::timestamp_iso8601();
        assert!(
            ts.len() >= 20,
            "ISO8601 timestamp should be at least 20 chars"
        );
        assert!(ts.ends_with('Z'), "ISO8601 should end with Z");
    }

    #[test]
    fn test_is_admin() {
        let admin = secdel_lib::utils::is_admin();
        assert_eq!(admin, false, "Tests typically run without admin privileges");
    }

    #[test]
    fn test_hash_file() {
        let dir = std::env::temp_dir().join("secdel_test_hash");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("hash_test.bin");

        fs::write(&file_path, b"hello world").expect("Failed to create test file");

        let hash = secdel_lib::utils::hash_file(&file_path).expect("Hashing should succeed");
        assert_eq!(hash.len(), 64, "SHA-256 should produce 64 hex chars");
        assert_eq!(
            hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "SHA-256 of 'hello world' should match expected"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_device_info_serde() {
        let info = secdel_lib::core::drive_sanitize::DeviceInfo {
            path: "\\\\.\\PhysicalDrive0".into(),
            model: "Test Drive".into(),
            size: 1024u64 * 1024 * 1024 * 256,
            interface_type: "NVMe".into(),
            is_ssd: true,
            is_nvme: true,
            supports_trim: true,
            supports_secure_erase: true,
            supports_sanitize: true,
            supports_crypto_erase: false,
        };

        let json = serde_json::to_string(&info).expect("Serialization should succeed");
        assert!(
            json.contains("PhysicalDrive0"),
            "JSON should contain device path"
        );
        assert!(json.contains("NVMe"), "JSON should contain interface type");

        let deserialized: secdel_lib::core::drive_sanitize::DeviceInfo =
            serde_json::from_str(&json).expect("Deserialization should succeed");
        assert_eq!(deserialized.path, info.path);
        assert_eq!(deserialized.size, info.size);
    }

    #[test]
    fn test_pattern_type_serde() {
        let patterns = vec![
            secdel_lib::core::overwrite::PatternType::Random,
            secdel_lib::core::overwrite::PatternType::Zero,
            secdel_lib::core::overwrite::PatternType::One,
            secdel_lib::core::overwrite::PatternType::RandomZero,
        ];

        for pattern in &patterns {
            let json = serde_json::to_string(pattern).expect("Serialization should succeed");
            let deserialized: secdel_lib::core::overwrite::PatternType =
                serde_json::from_str(&json).expect("Deserialization should succeed");
            assert!(std::mem::discriminant(pattern) == std::mem::discriminant(&deserialized));
        }
    }
}
