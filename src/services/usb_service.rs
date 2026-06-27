// src/services/usb_service.rs
use crate::models::drive_model::{DriveModel, DriveType};
use anyhow::Result;
use regex::Regex;
use std::path::Path;

pub struct UsbService;

impl UsbService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_available_drives(&self) -> Result<Vec<DriveModel>> {
        let mut drives = Vec::new();

        #[cfg(target_os = "macos")]
        {
            drives.extend(self.get_macos_volumes().await?);
        }

        #[cfg(target_os = "linux")]
        {
            drives.extend(self.get_linux_volumes().await?);
        }

        #[cfg(target_os = "windows")]
        {
            drives.extend(self.get_windows_volumes().await?);
        }

        Ok(drives)
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_volumes(&self) -> Result<Vec<DriveModel>> {
        let mut drives = Vec::new();

        // First, try to get external drives from diskutil
        let output = tokio::process::Command::new("diskutil")
            .args(["list", "external"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse diskutil output
        let disk_regex = Regex::new(r"/dev/(disk\d+)\s+\(external,\s+physical\)")?;

        for cap in disk_regex.captures_iter(&stdout) {
            if let Some(disk) = cap.get(1) {
                let disk_name = disk.as_str();
                let dev_path = format!("/dev/{}", disk_name);

                // Get detailed info for this disk
                let info_output = tokio::process::Command::new("diskutil")
                    .args(["info", &dev_path])
                    .output()
                    .await?;

                let info_stdout = String::from_utf8_lossy(&info_output.stdout);

                // Extract mount point
                let mount_regex = Regex::new(r"Mount Point:\s+(.+)")?;
                let mut mount_point = String::new();

                if let Some(cap) = mount_regex.captures(&info_stdout) {
                    if let Some(mp) = cap.get(1) {
                        mount_point = mp.as_str().trim().to_string();
                    }
                }

                // Extract volume name
                let name_regex = Regex::new(r"Volume Name:\s+(.+)")?;
                let mut volume_name = format!("USB Drive {}", disk_name);

                if let Some(cap) = name_regex.captures(&info_stdout) {
                    if let Some(name) = cap.get(1) {
                        let name_str = name.as_str().trim();
                        if !name_str.is_empty() {
                            volume_name = name_str.to_string();
                        }
                    }
                }

                // Extract vendor
                let vendor_regex = Regex::new(r"Device / Media Name:\s+(.+)")?;
                let mut vendor = "Unknown".to_string();

                if let Some(cap) = vendor_regex.captures(&info_stdout) {
                    if let Some(v) = cap.get(1) {
                        vendor = v.as_str().trim().to_string();
                    }
                }

                // Get space info if mounted
                if !mount_point.is_empty() && Path::new(&mount_point).exists() {
                    let (total_space, free_space) = self.get_space_info(&mount_point).await;

                    if total_space > 0 {
                        drives.push(DriveModel {
                            id: dev_path.clone(),
                            name: volume_name.clone(),
                            path: dev_path,
                            total_space,
                            used_space: total_space - free_space,
                            is_removable: true,
                            mount_point: mount_point.clone(),
                            drive_type: DriveType::Usb,
                            vendor: vendor.clone(),
                            model: volume_name,
                            serial_number: "".to_string(),
                            is_mounted: true,
                        });
                    }
                }
            }
        }

        // Also scan /Volumes for any mounted USB drives
        let volumes_path = Path::new("/Volumes");
        if volumes_path.exists() {
            if let Ok(entries) = std::fs::read_dir(volumes_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // Skip system volumes
                        let skip_volumes = [
                            "Macintosh HD",
                            "System",
                            "Recovery",
                            "com.apple",
                            "Data",
                            "Preboot",
                            "VM",
                            "Update",
                        ];

                        let should_skip = skip_volumes
                            .iter()
                            .any(|&v| name == v || name.starts_with(v));
                        if should_skip {
                            continue;
                        }

                        // Check if this volume is already in our list
                        let path_str = path.to_str().unwrap_or("");
                        let exists = drives.iter().any(|d| d.mount_point == path_str);

                        if !exists {
                            let (total_space, free_space) = self.get_space_info(path_str).await;

                            if total_space > 0 {
                                drives.push(DriveModel {
                                    id: path_str.to_string(),
                                    name: name.clone(),
                                    path: path_str.to_string(),
                                    total_space,
                                    used_space: total_space - free_space,
                                    is_removable: true,
                                    mount_point: path_str.to_string(),
                                    drive_type: DriveType::Usb,
                                    vendor: "Unknown".to_string(),
                                    model: name,
                                    serial_number: "".to_string(),
                                    is_mounted: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(drives)
    }

    #[cfg(target_os = "macos")]
    async fn get_space_info(&self, path: &str) -> (u64, u64) {
        let total = self.get_total_space(path).await;
        let free = self.get_free_space(path).await;
        (total, free)
    }

    #[cfg(target_os = "macos")]
    async fn get_free_space(&self, path: &str) -> u64 {
        // Use df to get free space
        if let Ok(output) = tokio::process::Command::new("df")
            .args(["-k", path])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(blocks) = parts[3].parse::<u64>() {
                        return blocks * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    #[cfg(target_os = "macos")]
    async fn get_total_space(&self, path: &str) -> u64 {
        // Use diskutil to get total space
        if let Ok(output) = tokio::process::Command::new("diskutil")
            .args(["info", path])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Try to parse GB
            let gb_regex = Regex::new(r"Total Size:\s+([\d.]+)\s+GB").unwrap();
            if let Some(cap) = gb_regex.captures(&stdout) {
                if let Some(size_str) = cap.get(1) {
                    if let Ok(size_gb) = size_str.as_str().parse::<f64>() {
                        return (size_gb * 1024.0 * 1024.0 * 1024.0) as u64;
                    }
                }
            }

            // Try to parse TB
            let tb_regex = Regex::new(r"Total Size:\s+([\d.]+)\s+TB").unwrap();
            if let Some(cap) = tb_regex.captures(&stdout) {
                if let Some(size_str) = cap.get(1) {
                    if let Ok(size_tb) = size_str.as_str().parse::<f64>() {
                        return (size_tb * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64;
                    }
                }
            }
        }

        // Fallback: try df
        if let Ok(output) = tokio::process::Command::new("df")
            .args(["-k", path])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(blocks) = parts[1].parse::<u64>() {
                        return blocks * 1024; // Convert KB to bytes
                    }
                }
            }
        }

        0
    }

    #[cfg(target_os = "linux")]
    async fn get_linux_volumes(&self) -> Result<Vec<DriveModel>> {
        let mut drives = Vec::new();

        // Compile regex patterns once outside the loop
        let name_regex = Regex::new(r#"NAME="([^"]+)""#)?;
        let mount_regex = Regex::new(r#"MOUNTPOINT="([^"]*)""#)?;
        let label_regex = Regex::new(r#"LABEL="([^"]*)""#)?;
        let size_regex = Regex::new(r#"SIZE="([^"]+)""#)?;

        // Use lsblk to find USB drives
        if let Ok(output) = tokio::process::Command::new("lsblk")
            .args([
                "-P",
                "-o",
                "NAME,FSTYPE,LABEL,SIZE,FSAVAIL,MOUNTPOINT,RM,HOTPLUG,TRAN,TYPE",
            ])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                // Check if this is a removable or USB device
                if line.contains("RM=\"1\"") || line.contains("TRAN=\"usb\"") {
                    // Extract device name
                    if let Some(name_cap) = name_regex.captures(line) {
                        let name = name_cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip loop devices and CD-ROMs
                        if name.contains("loop") || name.contains("sr") {
                            continue;
                        }

                        // Extract mount point
                        let mount_point = mount_regex
                            .captures(line)
                            .and_then(|cap| cap.get(1))
                            .map(|m| m.as_str())
                            .unwrap_or("");

                        // Extract label
                        let label = label_regex
                            .captures(line)
                            .and_then(|cap| cap.get(1))
                            .map(|m| m.as_str())
                            .filter(|&s| !s.is_empty())
                            .unwrap_or(name);

                        // Extract size
                        let size_str = size_regex
                            .captures(line)
                            .and_then(|cap| cap.get(1))
                            .map(|m| m.as_str())
                            .unwrap_or("0");

                        // Parse size (e.g., "10.5G" -> bytes)
                        let total_space = self.parse_size(size_str);

                        // Only add if we have a valid mount point and size
                        if total_space > 0 && !mount_point.is_empty() {
                            // Get free space using df
                            let free_space = self.get_free_space_linux(mount_point).await;

                            drives.push(DriveModel {
                                id: format!("/dev/{}", name),
                                name: label.to_string(),
                                path: format!("/dev/{}", name),
                                total_space,
                                used_space: if total_space > free_space {
                                    total_space - free_space
                                } else {
                                    0
                                },
                                is_removable: true,
                                mount_point: mount_point.to_string(),
                                drive_type: DriveType::Usb,
                                vendor: "Unknown".to_string(),
                                model: label.to_string(),
                                serial_number: "".to_string(),
                                is_mounted: true,
                            });
                        }
                    }
                }
            }
        }

        Ok(drives)
    }

    #[cfg(target_os = "linux")]
    async fn get_free_space_linux(&self, path: &str) -> u64 {
        if let Ok(output) = tokio::process::Command::new("df")
            .args(["-k", path])
            .output()
            .await
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(blocks) = parts[3].parse::<u64>() {
                        return blocks * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        0
    }

    #[cfg(target_os = "linux")]
    fn parse_size(&self, size_str: &str) -> u64 {
        let size_str = size_str.trim();
        if size_str.is_empty() {
            return 0;
        }

        // Try to parse with suffix
        if let Some(stripped) = size_str.strip_suffix('G') {
            if let Ok(size) = stripped.parse::<f64>() {
                return (size * 1024.0 * 1024.0 * 1024.0) as u64;
            }
        }
        if let Some(stripped) = size_str.strip_suffix('T') {
            if let Ok(size) = stripped.parse::<f64>() {
                return (size * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64;
            }
        }
        if let Some(stripped) = size_str.strip_suffix('M') {
            if let Ok(size) = stripped.parse::<f64>() {
                return (size * 1024.0 * 1024.0) as u64;
            }
        }
        if let Some(stripped) = size_str.strip_suffix('K') {
            if let Ok(size) = stripped.parse::<f64>() {
                return (size * 1024.0) as u64;
            }
        }

        // Try to parse as plain number (bytes)
        if let Ok(size) = size_str.parse::<u64>() {
            return size;
        }

        0
    }

    #[cfg(target_os = "windows")]
    async fn get_windows_volumes(&self) -> Result<Vec<DriveModel>> {
        // Windows implementation using winapi
        // For now, return empty vector
        Ok(Vec::new())
    }
}
