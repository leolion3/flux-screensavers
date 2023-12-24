use crate::config::{ColorMode, Config, FillMode};

use indoc::indoc;

use iced::alignment::Horizontal;
use iced::executor;
use iced::theme;
use iced::widget::{button, column, container, pick_list, row, text, vertical_space};
use iced::window;
use iced::{Application, Command, Element, Length, Padding, Theme};

pub fn run(config: Config) -> iced::Result {
    Config::run(iced::Settings {
        flags: config,
        window: iced::window::Settings {
            size: (500, 500),
            resizable: false,
            decorations: true,
            ..Default::default()
        },
        default_text_size: 16.0,
        ..Default::default()
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SetColorMode(ColorMode),
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
            Some(self.flux.color_mode),
            Message::SetColorMode,
        )
        .padding(8);

        let color_section = column![
            text("Colors").size(20.0),
            "Choose from a selection of presets or use your desktop wallpaper.",
            color_list
        ]
        .spacing(12);

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

        let content = column![
            color_section,
            fill_section,
            vertical_space(Length::Fill),
            button_row
        ]
        .height(Length::Fill)
        .spacing(36);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(24)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
