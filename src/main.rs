use iced::{
    widget::{column, scrollable, text, text_input},
    Application,
};

use nucleo_matcher::{
    pattern::CaseMatching, pattern::Normalization, pattern::Pattern, Config, Matcher,
};

use freedesktop_entry_parser::parse_entry;

use rayon::prelude::*;

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
    exec: String,
}

struct Runner {
    input_text_state: String,
    entries: Vec<AppEntry>,
    active_entry: Option<usize>,
    entries_limit: usize,
}

impl Runner {
    fn entries_shift<P, M>(&mut self, predicate: P, active_mutate: M)
    where
        P: Fn(usize) -> bool,
        M: Fn(usize) -> usize,
    {
        if let Some(active_ref) = &mut self.active_entry {
            if predicate(*active_ref) {
                self.entries[*active_ref].name.replace_range(0..2, "- ");
                // *active_ref -= 1;
                *active_ref = active_mutate(*active_ref);
                self.entries[*active_ref].name.replace_range(0..2, "> ");
            }
        } else if !self.entries.is_empty() {
            let active_ref = self.entries.len() - 1;
            self.entries[active_ref].name.replace_range(0..2, "> ");
            self.active_entry = Some(active_ref);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    ListUp,
    ListDown,
    Acc,
}

impl AsRef<str> for AppEntry {
    fn as_ref(&self) -> &str {
        &self.name
    }
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
                active_entry: None,
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
                self.input_text_state.clone_from(&str);
                // TODO: add validation to paths, take them from env var or from specification
                let paths = [
                    std::path::Path::new("/usr/share/applications/"),
                    std::path::Path::new("/usr/local/share/applications/"),
                ];
                let desktop = paths
                    .into_iter()
                    .map(|d| std::fs::read_dir(d).unwrap())
                    .flat_map(|d| d.into_iter())
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .map(|d| d.unwrap().path())
                    .filter(|entry| {
                        if let Some(ext) = entry.extension() {
                            return ext == "desktop"
                                && parse_entry(entry)
                                    .unwrap()
                                    .section("Desktop Entry")
                                    .has_attr("Exec");
                        }
                        false
                    })
                    .map(|entry| {
                        let desktop = parse_entry(entry.clone()).unwrap();
                        let name = desktop
                            .section("Desktop Entry")
                            .attr("Name")
                            .expect("Required Attr doesn't exist");
                        let exec = desktop
                            .section("Desktop Entry")
                            .attr("Exec")
                            .expect(name)
                            .to_string();
                        AppEntry {
                            name: name.to_string(),
                            exec: exec.to_string(),
                        }
                        // name.to_string()
                    })
                    .collect::<Vec<_>>();
                let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
                let matches = Pattern::parse(&str, CaseMatching::Ignore, Normalization::Smart)
                    .match_list(desktop, &mut matcher);
                let mut matches = matches
                    .into_iter()
                    .map(|(mut entry, _)| {
                        entry.name = "- ".to_string() + &entry.name;
                        entry
                    })
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    self.active_entry = None;
                    self.entries = vec![];
                    return iced::Command::none();
                } else if self.active_entry.is_none() {
                    self.active_entry = Some(0);
                }
                if let Some(active_ref) = &mut self.active_entry {
                    let active_ref = std::cmp::min(*active_ref, matches.len() - 1);
                    matches[active_ref].name.replace_range(0..2, "> ");
                    self.entries =
                        matches[0..std::cmp::min(self.entries_limit, matches.len())].to_vec();
                    self.active_entry = Some(active_ref);
                } else {
                    unreachable!()
                }
                iced::Command::none()
            }
            Message::Acc => {
                use std::process::Command;
                if let Some(ind) = self.active_entry {
                    Command::new(&self.entries[ind].exec)
                        .spawn()
                        .expect(&self.entries[ind].exec);
                }
                iced::window::close(iced::window::Id::MAIN)
            }
            Message::ListUp => {
                self.entries_shift(|x| x > 0, |x| x - 1);
                iced::Command::none()
            }
            Message::ListDown => {
                let max_shiftable = self.entries.len() - 1;
                self.entries_shift(|x| x < max_shiftable, |x| x + 1);
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
