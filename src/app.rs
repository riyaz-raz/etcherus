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
    NoOp,
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
            Message::SelectImage => Command::perform(
                async {
                    let file_handle = rfd::AsyncFileDialog::new()
                        .add_filter("ISO Images", &["iso"])
                        .add_filter("Disk Images", &["img"])
                        .add_filter("Binary Files", &["bin"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                        .await;

                    if let Some(handle) = file_handle {
                        let path = handle.path();
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

                        let file_type = match path.extension().and_then(|ext| ext.to_str()) {
                            Some("iso") => crate::models::image_model::ImageType::Os,
                            Some("img") => crate::models::image_model::ImageType::Os,
                            _ => crate::models::image_model::ImageType::Firmware,
                        };

                        Some(ImageModel {
                            name,
                            path: path.to_string_lossy().to_string(),
                            size,
                            file_type,
                        })
                    } else {
                        None
                    }
                },
                |maybe_image| {
                    if let Some(image) = maybe_image {
                        Message::ImageSelected(image)
                    } else {
                        Message::NoOp
                    }
                },
            ),
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
            Message::NoOp => Command::none(),
        }
    }

    // ✅ FIXED: Store HomeView in a variable
    fn view(&self) -> Element<'_, Message> {
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
