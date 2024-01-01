use crate::config::{ColorMode, Config, FillMode};

use async_std::task;
use indoc::indoc;
use std::path::PathBuf;
use tinyfiledialogs::open_file_dialog;

use iced::alignment::{Alignment, Horizontal};
use iced::executor;
use iced::theme;
use iced::widget::{button, column, container, pick_list, row, text, vertical_space};
use iced::window;
use iced::{Application, Command, Element, Length, Theme};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run(config: Config) -> iced::Result {
    Config::run(iced::Settings {
        flags: config,
        window: iced::window::Settings {
            size: (420, 600),
            resizable: false,
            decorations: true,
            ..Default::default()
        },
        default_text_size: 16.0,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    SetColorMode(ColorMode),
    OpenFilePicker,
    SetImageFile(Option<String>),
    SetFillMode(FillMode),
    Save,
    Cancel,
}

impl Application for Config {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Config) -> (Self, Command<Message>) {
        (config, Command::none())
    }

    fn title(&self) -> String {
        String::from("Flux Settings")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SetColorMode(new_color) => {
                self.flux.color_mode = new_color;
                Command::none()
            }

            Message::OpenFilePicker => Command::perform(
                task::spawn_blocking(|| {
                    open_file_dialog(
                        "Select an image",
                        "",
                        Some((&["*.jpg", "*.jpeg", "*.png"], "Images")),
                    )
                }),
                Message::SetImageFile,
            ),

            Message::SetImageFile(some_path) => {
                if let Some(path_string) = some_path {
                    let path = PathBuf::from(path_string);
                    self.flux.color_mode = ColorMode::ImageFile {
                        image_path: Some(path),
                    };
                }
                Command::none()
            }

            Message::SetFillMode(new_fill_mode) => {
                self.platform.windows.fill_mode = new_fill_mode;
                Command::none()
            }

            Message::Save => {
                self.save().unwrap_or_else(|err| log::error!("{}", err));
                window::close()
            }

            Message::Cancel => window::close(),
        }
    }

    fn view(&self) -> Element<Message> {
        let color_list = pick_list(
            &ColorMode::ALL[..],
            Some(self.flux.color_mode.clone()),
            Message::SetColorMode,
        )
        .padding(8);

        let mut color_section = column![
            text("Colors").size(20.0),
            "Choose from a selection of presets or use an image.",
            color_list
        ]
        .spacing(12);

        if let ColorMode::ImageFile { image_path } = &self.flux.color_mode {
            let mut image_picker = row![]
                .push(
                    button("Select image")
                        .padding(8)
                        .on_press(Message::OpenFilePicker),
                )
                .align_items(Alignment::Center)
                .spacing(12);
            if let Some(path) = &image_path {
                image_picker = image_picker.push(text(path.display()));
            }
            color_section = color_section.push(image_picker);
        }

        let save_button = button(text("Save").horizontal_alignment(Horizontal::Center))
            .padding(8)
            .width(Length::Fixed(96.0))
            .on_press(Message::Save);
        let cancel_button = button(text("Cancel").horizontal_alignment(Horizontal::Center))
            .style(theme::Button::Secondary)
            .padding(8)
            .width(Length::Fixed(96.0))
            .on_press(Message::Cancel);
        let button_row = container(row![save_button, cancel_button].spacing(12));

        let mut content = column![color_section]
            .width(Length::Fill)
            .spacing(36)
            .padding(36);

        if cfg!(windows) {
            let fill_list = pick_list(
                &FillMode::ALL[..],
                Some(self.platform.windows.fill_mode),
                Message::SetFillMode,
            )
            .padding(8);

            let fill_section = column![
                text("Fill mode").size(20.0),
                "Configure how Flux works across multiple monitors.",
                indoc! {"
                    None: Each monitor is a separate surface.
                    Span: Combines any matching adjacent monitors.
                    Fill: Combines all monitors into a single seamless surface.
                "},
                fill_list,
            ]
            .spacing(12);

            content = content.push(fill_section);
        }

        let version_text = text(format!("v{VERSION}")).size(12.0);

        content = content
            .push(button_row)
            .push(vertical_space(Length::Fill))
            .push(version_text);

        container(content).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
