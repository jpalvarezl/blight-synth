use iced::{Application, Settings, Length, executor, Command, widget::{Container, Column}};

// https://github.com/irvingfisica/iced_examples/blob/master/Life.md
// buen tutorial, pero en español!
pub fn main() -> iced::Result {
    BlightSynthApp::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct BlightSynthApp {
}

#[derive(Debug)]
struct ApplicationMessage {}

impl Application for BlightSynthApp {
    type Message = ApplicationMessage;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self {
            ..Self::default()
        },
        Command::none())
    }

    fn title(&self) -> String {
        String::from("Blight Synth")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = Column::new();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
