use iced::{
    widget::{column, scrollable, text, text_input},
    Application,
};

pub fn main() -> iced::Result {
    let window_settings = iced::window::Settings {
        // TODO: make size of window based on display configuration / config
        size: iced::Size {
            width: 800.0,
            height: 150.0,
        },
        // TODO: move position to config
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

#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
}

struct Runner {
    input_text_state: String,
    entries: Vec<AppEntry>,
    active_entry: usize,
    entries_limit: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    ListUp,
    ListDown,
    Acc,
}

impl Application for Runner {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Runner {
                input_text_state: String::from(""),
                entries: vec![],
                active_entry: 0,
                // TODO: move limit to config
                entries_limit: 5,
            },
            iced::Command::none(),
        )
    }
    fn view(&self) -> iced::Element<Self::Message> {
        let text_input = text_input("", &self.input_text_state)
            .on_input(Message::TextChanged)
            .on_submit(Message::Acc);
        let children_entries = self.entries.clone().into_iter().map(|x| {
            iced::Element::from(
                text(x.name)
                    .height(iced::Length::Fixed(24.0))
                    .width(iced::Length::Fill),
            )
        });
        column![text_input, scrollable(column(children_entries))].into()
    }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::TextChanged(str) => {
                self.input_text_state = str;
                iced::Command::none()
            }
            Message::Acc => {
                self.entries.push(AppEntry {
                    name: "- ".to_string() + &self.input_text_state.clone(),
                });
                self.input_text_state = "".to_string();
                iced::Command::none()
            }
            Message::ListUp => {
                self.entries[self.active_entry]
                    .name
                    .replace_range(0..2, "- ");
                if self.active_entry > 0 {
                    self.active_entry -= 1;
                    self.entries[self.active_entry]
                        .name
                        .replace_range(0..2, "> ");
                }
                iced::Command::none()
            }
            Message::ListDown => {
                self.entries[self.active_entry]
                    .name
                    .replace_range(0..2, "- ");
                if self.active_entry < self.entries.len() - 1 {
                    self.active_entry += 1;
                    self.entries[self.active_entry]
                        .name
                        .replace_range(0..2, "> ");
                } else {
                    self.active_entry = 0;
                }
                iced::Command::none()
            }
        }
    }
    fn title(&self) -> String {
        String::from("yarrun")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::keyboard;
        use iced::keyboard::key;

        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::ArrowUp, _) => Some(Message::ListUp),
                (key::Named::ArrowDown, _) => Some(Message::ListDown),
                _ => None,
            }
        })
    }
}
