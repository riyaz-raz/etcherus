use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriveModel {
    pub id: String,
    pub name: String,
    pub path: String,
    pub total_space: u64,
    pub used_space: u64,
    pub is_removable: bool,
    pub mount_point: String,
    pub drive_type: DriveType,
    pub vendor: String,
    pub model: String,
    pub serial_number: String,
    pub is_mounted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriveType {
    Usb,
    Sd,
    Internal,
    Other,
}

impl DriveModel {
    pub fn free_space(&self) -> u64 {
        self.total_space - self.used_space
    }

    pub fn formatted_total(&self) -> String {
        ByteSize::b(self.total_space).to_string_as(true)
    }

    pub fn formatted_free(&self) -> String {
        ByteSize::b(self.free_space()).to_string_as(true)
    }

    pub fn formatted_used(&self) -> String {
        ByteSize::b(self.used_space).to_string_as(true)
    }

    pub fn has_enough_space(&self, required_bytes: u64) -> bool {
        self.free_space() >= required_bytes
    }

    pub fn is_writable(&self) -> bool {
        // In a real app, check write permissions
        true
    }
}
