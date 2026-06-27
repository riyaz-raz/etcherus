use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageModel {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub file_type: ImageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageType {
    Firmware,
    Os,
    Data,
    Other,
}

impl ImageModel {
    pub fn formatted_size(&self) -> String {
        ByteSize::b(self.size).to_string_as(true)
    }

    pub fn type_display_name(&self) -> String {
        match self.file_type {
            ImageType::Firmware => "Firmware".to_string(),
            ImageType::Os => "OS Image".to_string(),
            ImageType::Data => "Data Image".to_string(),
            ImageType::Other => "Other".to_string(),
        }
    }
}
