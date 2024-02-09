// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use core::{get_default_output_device_name, start_audio_thread};
// use event_handlers::{keyboard::PianoKeyboard, InputStateHandler};
use iced::executor;
use iced::widget::{self, column, container, text};
use iced::{keyboard, subscription, Alignment, Application, Command, Length, Settings, Theme};
use view_models::oscillator::OscillatorViewModel;

mod event_handlers;
mod ui_components;
mod view_models;

fn main() {
    env_logger::init();

    let _ = MainState::run(Settings::default());
}

#[derive(Debug, Clone)]
enum MainMessage {
    KeyPressed,
    KeyReleased,
}

#[derive(Debug)]
enum MainState {
    Idle,
    KeyPressed(String),
}

impl Application for MainState {
    type Executor = iced::executor::Default;

    type Message = MainMessage;

    type Theme = Theme;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (MainState::Idle, Command::none())
    }

    fn title(&self) -> String {
        String::from("Blight Synth")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        subscription::events_with(|event, _status| match event {
            iced::Event::Keyboard(keyboard_event) => 
                match keyboard_event {
                    keyboard::Event::KeyPressed{key_code, modifiers} => {
                        println!("Key pressed: {:?}", key_code);
                        None
                    },
                    keyboard::Event::KeyReleased { key_code, modifiers } => {
                        println!("Key released: {:?}", key_code);
                        None
                    }
                    _ => None,
                },
            _ => None,
        })
    }

    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //     // subscription::events_with(|event, _status| match event {
    //     //     Event::Keyboard(keyboard_event) => match keyboard_event {
    //     //         // keyboard::Event::KeyPressed {
    //     //         //     key_code: keyboard::KeyCode::Tab,
    //     //         //     modifiers,
    //     //         // } => Some(if modifiers.shift {
    //     //         //     Message::FocusPrevious
    //     //         // } else {
    //     //         //     Message::FocusNext
    //     //         // }),
    //     //         // _ => None,
    //     //     },
    //     //     _ => None,
    //     // })
    // }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = match self {
            MainState::Idle => {
                column![text("Hello, world!").size(40),]
            }
            MainState::KeyPressed(key) => {
                column![text(key).size(40),]
            }
        }
        .max_width(500)
        .spacing(20)
        .align_items(Alignment::End);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

// struct Content {
//     default_output_device: String,
//     text: String,
//     input_handler: PianoKeyboard,
//     oscillator_viewmodel: OscillatorViewModel,
// }

// impl eframe::App for Content {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui_components::init_ui(ui, self);
//             InputStateHandler::handle_input(
//                 &mut self.input_handler,
//                 ctx,
//                 &self.oscillator_viewmodel,
//             );

//             self.text = self.input_handler.pressed_keys_as_string();

//             println!("Oscillator: {:#?}", self.oscillator_viewmodel.get_oscillator());
//         });
//     }
// }
