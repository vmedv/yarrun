use iced::{
    widget::{button, canvas, column, text, text_input, Column},
    Application, Command,
};

pub fn main() -> iced::Result {
    let window_settings = iced::window::Settings {
        size: iced::Size {
            width: 800.0,
            height: 100.0,
        },
        position: iced::window::Position::Centered,
        resizable: false,
        decorations: false,
        level: iced::window::Level::AlwaysOnTop,
        ..Default::default()
    };
    Runner::run(iced::Settings {
        antialiasing: true,
        window: window_settings,
        ..Default::default()
    })
}

struct Runner {
    fstate: String,
    entries: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    Up,
    Down,
}

impl Application for Runner {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Runner {
                fstate: String::from(""),
                entries: vec![],
            },
            iced::Command::none(),
        )
    }
    fn view(&self) -> iced::Element<Self::Message> {
        column![
            text_input("", &self.fstate)
                .on_input(Message::TextChanged)
                .on_submit(Message::Up),
            text(match self.entries.len() {
                0 => "a",
                i => &self.entries[i - 1],
            })
        ]
        .into()
    }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::TextChanged(str) => {
                self.fstate = str;
                iced::Command::none()
            }
            Message::Up => {
                self.entries.push(self.fstate.clone());
                self.fstate = "".to_string();
                iced::Command::none()
            }
            Message::Down => iced::Command::none(),
        }
    }
    fn title(&self) -> String {
        String::from("yarrun")
    }
}
