//! Windows device detection
//!
//! Uses PowerShell Get-Disk to enumerate block devices.

use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::utils::format_size;
use crate::log_error;

use super::types::BlockDevice;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Get list of block devices on Windows
pub fn get_block_devices() -> Result<Vec<BlockDevice>, String> {

    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Disk | Where-Object { $_.BusType -ne 'NVMe' -or $_.IsSystem -eq $false } | Select-Object Number, FriendlyName, Size, BusType | ConvertTo-Json",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| {
            log_error!("devices", "Failed to run PowerShell: {}", e);
            format!("Failed to run PowerShell: {}", e)
        })?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Disk | Where-Object { $_.BusType -ne 'NVMe' -or $_.IsSystem -eq $false } | Select-Object Number, FriendlyName, Size, BusType | ConvertTo-Json",
        ])
        .output()
        .map_err(|e| {
            log_error!("devices", "Failed to run PowerShell: {}", e);
            format!("Failed to run PowerShell: {}", e)
        })?;

    if !output.status.success() {
        log_error!("devices", "PowerShell command failed with status: {:?}", output.status);
        return Err("PowerShell command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| {
            log_error!("devices", "Failed to parse disk info JSON: {}", e);
            format!("Failed to parse disk info: {}", e)
        })?;

    let mut devices = Vec::new();

    let disks = if json.is_array() {
        json.as_array().unwrap().clone()
    } else {
        vec![json]
    };

    let system_disk = get_system_disk();

    for disk in disks {
        let number = disk["Number"].as_i64().unwrap_or(-1);
        if number < 0 {
            continue;
        }

        let size = disk["Size"].as_u64().unwrap_or(0);
        if size == 0 {
            continue;
        }

        let model = disk["FriendlyName"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        let bus_type_str = disk["BusType"].as_str().unwrap_or("");
        let is_removable = bus_type_str == "USB" || bus_type_str == "SD";

        // Mark as system disk instead of skipping (consistent with macOS behavior)
        let is_system = system_disk.map(|sys_num| number == sys_num).unwrap_or(false);

        // Get drive letters for user-friendly display
        let drive_letters = get_drive_letters(number);
        let name = match drive_letters {
            Some(letters) => format!("Disk {} ({})", number, letters),
            None => format!("Disk {}", number),
        };

        devices.push(BlockDevice {
            path: format!("\\\\.\\PhysicalDrive{}", number),
            name,
            size,
            size_formatted: format_size(size),
            model,
            is_removable,
            is_system,
            bus_type: if bus_type_str.is_empty() { None } else { Some(bus_type_str.to_string()) },
        });
    }

    Ok(devices)
}

/// Get the system disk number
fn get_system_disk() -> Option<i64> {
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Partition -DriveLetter C | Get-Disk).Number",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Partition -DriveLetter C | Get-Disk).Number",
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse().ok()
}

/// Get drive letters for a disk number (e.g., "C:", "D:")
fn get_drive_letters(disk_number: i64) -> Option<String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Get-Partition -DiskNumber {} | Where-Object {{ $_.DriveLetter }} | Select-Object -ExpandProperty DriveLetter",
                disk_number
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Get-Partition -DiskNumber {} | Where-Object {{ $_.DriveLetter }} | Select-Object -ExpandProperty DriveLetter",
                disk_number
            ),
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let letters: Vec<String> = stdout
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("{}:", s.trim()))
        .collect();

    if letters.is_empty() {
        None
    } else {
        Some(letters.join(", "))
    }
}
