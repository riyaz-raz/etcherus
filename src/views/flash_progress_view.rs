use crate::app::Message;
use crate::models::{drive_model::DriveModel, image_model::ImageModel};
use iced::{
    widget::{Column, Container, ProgressBar, Text},
    Alignment, Element, Length,
};

pub struct FlashProgressView {
    image: ImageModel,
    drive: DriveModel,
    progress: f32,
    status: String,
    is_complete: bool,
    error: Option<String>,
}

impl FlashProgressView {
    pub fn new(image: ImageModel, drive: DriveModel) -> Self {
        Self {
            image,
            drive,
            progress: 0.0,
            status: "Preparing...".to_string(),
            is_complete: false,
            error: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new("Flashing in progress...").size(24))
            .push(
                Column::new()
                    .spacing(8)
                    .push(Text::new(format!("Image: {}", self.image.name)))
                    .push(Text::new(format!("Drive: {}", self.drive.name))),
            )
            .push(ProgressBar::new(0.0..=1.0, self.progress).width(Length::Fill))
            .push(Text::new(format!("{:.1}%", self.progress * 100.0)))
            .push(Text::new(&self.status));

        // Add status message based on state
        if self.is_complete {
            content = content.push(Text::new("✅ Flash completed successfully!"));
        } else if let Some(error) = &self.error {
            content = content.push(Text::new(format!("❌ Error: {}", error)));
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .center_x()
            .center_y()
            .into()
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress;
        if progress < 0.3 {
            self.status = "Writing image...".to_string();
        } else if progress < 0.6 {
            self.status = "Verifying write...".to_string();
        } else if progress < 0.9 {
            self.status = "Finalizing...".to_string();
        } else if progress >= 1.0 {
            self.status = "Complete!".to_string();
            self.is_complete = true;
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.status = "Error occurred".to_string();
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}
