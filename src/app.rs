use crate::models::{drive_model::DriveModel, image_model::ImageModel};
use crate::services::{flash_service::FlashService, usb_service::UsbService};
use crate::views::HomeView;
use iced::{executor, Application, Command, Element, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    LoadDrives,
    DrivesLoaded(Vec<DriveModel>),
    DriveSelected(DriveModel),
    SelectImage,
    ImageSelected(ImageModel),
    StartFlash,
    FlashProgress(f32),
    FlashComplete,
    FlashError(String),
    Refresh,
    ClearError,
}

pub struct EtcherusApp {
    usb_service: UsbService,
    flash_service: FlashService,
    drives: Vec<DriveModel>,
    selected_drive: Option<DriveModel>,
    selected_image: Option<ImageModel>,
    is_loading: bool,
    is_flashing: bool,
    flash_progress: f32,
    error: Option<String>,
    validation_message: Option<String>,
    is_valid: bool,
}

impl Application for EtcherusApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                usb_service: UsbService::new(),
                flash_service: FlashService::new(),
                drives: Vec::new(),
                selected_drive: None,
                selected_image: None,
                is_loading: false,
                is_flashing: false,
                flash_progress: 0.0,
                error: None,
                validation_message: None,
                is_valid: false,
            },
            Command::perform(async { Message::LoadDrives }, |_| Message::LoadDrives),
        )
    }

    fn title(&self) -> String {
        String::from("Flash Tool")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadDrives => {
                self.is_loading = true;
                self.error = None;
                Command::perform(
                    async {
                        let service = UsbService::new();
                        match service.get_available_drives().await {
                            Ok(drives) => Message::DrivesLoaded(drives),
                            Err(e) => Message::FlashError(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }
            Message::DrivesLoaded(drives) => {
                self.is_loading = false;
                self.drives = drives;
                self.validate_selection();
                Command::none()
            }
            Message::DriveSelected(drive) => {
                self.selected_drive = Some(drive);
                self.validate_selection();
                Command::none()
            }
            Message::SelectImage => {
                let mock_image = ImageModel {
                    name: "firmware_v1.2.bin".to_string(),
                    path: "/path/to/firmware_v1.2.bin".to_string(),
                    size: 1024 * 1024 * 50,
                    file_type: crate::models::image_model::ImageType::Firmware,
                };
                self.selected_image = Some(mock_image);
                self.validate_selection();
                Command::none()
            }
            Message::ImageSelected(image) => {
                self.selected_image = Some(image);
                self.validate_selection();
                Command::none()
            }
            Message::StartFlash => {
                if self.is_valid {
                    self.is_flashing = true;
                    self.flash_progress = 0.0;
                    Command::perform(
                        async {
                            let flash_service = FlashService::new();
                            match flash_service.start_flash().await {
                                Ok(_) => Message::FlashComplete,
                                Err(e) => Message::FlashError(e.to_string()),
                            }
                        },
                        |msg| msg,
                    )
                } else {
                    Command::none()
                }
            }
            Message::FlashProgress(progress) => {
                self.flash_progress = progress;
                Command::none()
            }
            Message::FlashComplete => {
                self.is_flashing = false;
                self.flash_progress = 1.0;
                self.validation_message = Some("Flash completed successfully!".to_string());
                self.is_valid = true;
                Command::none()
            }
            Message::FlashError(error) => {
                self.is_flashing = false;
                self.error = Some(error);
                Command::none()
            }
            Message::Refresh => {
                self.is_loading = true;
                self.error = None;
                Command::perform(
                    async {
                        let service = UsbService::new();
                        match service.get_available_drives().await {
                            Ok(drives) => Message::DrivesLoaded(drives),
                            Err(e) => Message::FlashError(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }
            Message::ClearError => {
                self.error = None;
                Command::none()
            }
        }
    }

    // ✅ FIXED: Store HomeView in a variable
    fn view(&self) -> Element<Message> {
        let home_view = HomeView::new(
            &self.drives,
            self.selected_drive.clone(),
            self.selected_image.clone(),
            self.is_loading,
            self.is_flashing,
            self.flash_progress,
            self.error.clone(),
            self.validation_message.clone(),
            self.is_valid,
        );

        home_view.view()
    }
}

impl EtcherusApp {
    fn validate_selection(&mut self) {
        if let (Some(image), Some(drive)) = (&self.selected_image, &self.selected_drive) {
            if drive.total_space > image.size {
                self.is_valid = true;
                self.validation_message = Some("Ready to flash!".to_string());
            } else {
                self.is_valid = false;
                self.validation_message = Some("Drive doesn't have enough space".to_string());
            }
        } else {
            self.is_valid = false;
            self.validation_message = Some("Please select both image and drive".to_string());
        }
    }
}
