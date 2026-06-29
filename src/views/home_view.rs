use crate::app::Message;
use crate::models::{drive_model::DriveModel, image_model::ImageModel};
use iced::{
    widget::{self, button, container, row, text, Column, Space},
    Alignment, Color, Element, Length,
};

pub struct HomeView {
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

impl HomeView {
    pub fn new(
        drives: &[DriveModel],
        selected_drive: Option<DriveModel>,
        selected_image: Option<ImageModel>,
        is_loading: bool,
        is_flashing: bool,
        flash_progress: f32,
        error: Option<String>,
        validation_message: Option<String>,
        is_valid: bool,
    ) -> Self {
        Self {
            drives: drives.to_vec(),
            selected_drive,
            selected_image,
            is_loading,
            is_flashing,
            flash_progress,
            error,
            validation_message,
            is_valid,
        }
    }

    pub fn view(self) -> Element<'static, Message> {
        let content = if self.is_loading {
            Self::loading_view()
        } else {
            self.main_view()
        };

        // Convert the Column to Element here
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn loading_view() -> Column<'static, Message> {
        Column::new()
            .push(text("Loading drives...").size(20))
            .push(
                widget::ProgressBar::new(0.0..=1.0, 0.5)
                    .style(iced::theme::ProgressBar::Primary)
                    .width(Length::Fill),
            )
            .spacing(20)
            .align_items(Alignment::Center)
    }

    fn main_view(self) -> Column<'static, Message> {
        let mut col = Column::new().spacing(20);

        col = col.push(Self::image_selection_section(self.selected_image));
        col = col.push(Self::drive_selection_section(
            self.drives,
            self.selected_drive,
        ));

        if let Some(msg) = self.validation_message {
            col = col.push(Self::validation_status(self.is_valid, msg));
        }

        if let Some(error) = self.error {
            col = col.push(Self::error_display(error));
        }

        col = col.push(Self::action_buttons(
            self.is_valid,
            self.is_flashing,
            self.flash_progress,
        ));

        col
    }

    fn image_selection_section(selected_image: Option<ImageModel>) -> Column<'static, Message> {
        let mut col = Column::new().spacing(12);

        let mut header = row![text("📷 Image").size(18), Space::with_width(Length::Fill),];

        if selected_image.is_some() {
            header = header.push(button("Change").on_press(Message::SelectImage));
        } else {
            header = header.push(widget::Row::new());
        }

        col = col.push(header);

        if let Some(image) = selected_image {
            let formatted_size = image.formatted_size();
            let image_info = row![container(
                row![
                    text("📄"),
                    text(image.name),
                    text(format!("({})", formatted_size)),
                ]
                .spacing(8)
            )
            .padding(10)
            .style(container_styles::bordered_box)];
            col = col.push(image_info);
        } else {
            let select_button = button("Select Image")
                .on_press(Message::SelectImage)
                .padding(10);
            col = col.push(select_button);
        }

        col
    }

    fn drive_selection_section(
        drives: Vec<DriveModel>,
        selected_drive: Option<DriveModel>,
    ) -> Column<'static, Message> {
        let mut col = Column::new().spacing(12);

        let header = row![
            text("💾 Drive").size(18),
            Space::with_width(Length::Fill),
            text(format!("{} found", drives.len())).size(14),
        ];

        col = col.push(header);

        if drives.is_empty() {
            let empty_state = container(
                Column::new()
                    .push(text("🔌 No drives found").size(16))
                    .push(text("Insert a USB drive and refresh").size(14))
                    .push(button("Refresh").on_press(Message::Refresh))
                    .spacing(8)
                    .align_items(Alignment::Center),
            )
            .padding(20)
            .style(container_styles::bordered_box)
            .width(Length::Fill);

            col = col.push(empty_state);
        } else {
            for drive in drives {
                let is_selected = selected_drive
                    .as_ref()
                    .map(|d| d.id == drive.id)
                    .unwrap_or(false);

                col = col.push(Self::drive_card(drive, is_selected));
            }
        }

        col
    }

    fn drive_card(drive: DriveModel, is_selected: bool) -> Element<'static, Message> {
        let mut row = row![
            Column::new()
                .push(text(drive.name.clone()).size(16))
                .push(
                    text(format!(
                        "{} total, {} free",
                        drive.formatted_total(),
                        drive.formatted_free()
                    ))
                    .size(12),
                )
                .push(text(drive.mount_point.clone()).size(12))
                .spacing(4),
            Space::with_width(Length::Fill),
        ];

        if is_selected {
            row = row.push(text("✅").size(24));
        } else {
            row = row.push(widget::Row::new());
        }

        let content = container(row.align_items(Alignment::Center).padding(10))
            .padding(10)
            .style(container_styles::rounded_box)
            .width(Length::Fill);

        button(content)
            .on_press(Message::DriveSelected(drive))
            .style(iced::theme::Button::Text)
            .into()
    }

    fn validation_status(is_valid: bool, msg: String) -> Element<'static, Message> {
        let is_success = is_valid;
        let (icon, color) = if is_success {
            ("✅", Color::from_rgb(0.0, 0.6, 0.0))
        } else {
            ("⚠️", Color::from_rgb(0.8, 0.5, 0.0))
        };

        let status = row![
            text(icon).size(20),
            text(msg).size(14).style(iced::theme::Text::Color(color)),
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        container(status)
            .padding(12)
            .style(container_styles::bordered_box)
            .width(Length::Fill)
            .into()
    }

    fn error_display(error: String) -> Element<'static, Message> {
        let content = row![
            text("❌").size(20),
            text(error).size(14),
            Space::with_width(Length::Fill),
            button("×")
                .on_press(Message::ClearError)
                .style(iced::theme::Button::Text),
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        container(content)
            .padding(12)
            .style(container_styles::bordered_box)
            .width(Length::Fill)
            .into()
    }

    fn action_buttons(
        is_valid: bool,
        is_flashing: bool,
        flash_progress: f32,
    ) -> Column<'static, Message> {
        let mut col = Column::new().spacing(12);

        let is_ready = is_valid && !is_flashing;

        let button_content: Element<'static, Message> = if is_flashing {
            row![
                text("⏳ Flashing in progress..."),
                widget::ProgressBar::new(0.0..=1.0, flash_progress)
                    .style(iced::theme::ProgressBar::Primary)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .into()
        } else {
            text("▶️ Start Flashing").size(16).into()
        };

        let flash_button = button(button_content)
            .on_press_maybe(if is_ready {
                Some(Message::StartFlash)
            } else {
                None
            })
            .padding(12)
            .width(Length::Fill);

        col = col.push(flash_button);

        let refresh_button = button("🔄 Refresh")
            .on_press(Message::Refresh)
            .padding(8)
            .width(Length::Fill);

        col = col.push(refresh_button);

        col
    }
}

mod container_styles {
    use iced::widget::container;
    use iced::{border::Radius, Border, Color, Shadow};

    pub fn bordered_box(theme: &iced::Theme) -> container::Appearance {
        let palette = theme.palette();
        container::Appearance {
            text_color: None,
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: Radius::from(4.0),
                width: 1.0,
                color: Color::from_rgb(0.8, 0.8, 0.8),
            },
            shadow: Shadow::default(),
        }
    }

    pub fn rounded_box(theme: &iced::Theme) -> container::Appearance {
        let palette = theme.palette();
        container::Appearance {
            text_color: None,
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: Radius::from(8.0),
                width: 1.0,
                color: Color::from_rgb(0.8, 0.8, 0.8),
            },
            shadow: Shadow::default(),
        }
    }
}
