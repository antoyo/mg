/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

extern crate glib;
extern crate gtk;
#[macro_use]
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use std::sync::Arc;

use gtk::{
    Inhibit,
    OrientableExt,
    WidgetExt,
    WindowExt,
};
use gtk::Orientation::Vertical;
use mg::{
    Mg,
    NoSpecialCommands,
    StatusBar,
    StatusBarItem,
    View,
};
use relm::Widget;
use relm_attributes::widget;

use AppCommand::*;
use Msg::*;

#[derive(Clone)]
pub struct Model {
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    Command(AppCommand),
}

#[derive(Commands)]
pub enum AppCommand {
    Insert,
    Normal,
    Open(String),
    Quit,
}

static MODES: &[(&str, &str)] = &[
    ("i", "insert"),
];

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            text: "Mg App".to_string(),
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Command(command) => {
                match command {
                    Insert => (), //self.app.set_mode("insert"),
                    Normal => (), //self.app.set_mode("normal"),
                    Open(url) => model.text = format!("Opening URL {}", url),
                    Quit => gtk::main_quit(),
                }
            },
        }
    }

    view! {
        Mg<AppCommand>((MODES, "examples/main.conf")) {
            dark_theme: true,
            title: "First Mg Program",
            View {
                gtk::Box {
                    orientation: Vertical,
                    gtk::Label {
                        text: &model.text,
                    },
                    gtk::Entry {
                    },
                }
            },
            StatusBar {
                StatusBarItem {
                    text: "Rightmost",
                },
                StatusBarItem {
                    text: "Test",
                },
            },
            //Command => Command, // TODO
        }
    }
}

/*struct App {
    app: Box<mg::Application<AppCommand, NoSettings, NoSpecialCommands>>,
    label: Label,
}

impl App {
    fn new() -> Box<Self> {
        let mut app: Box<Application<AppCommand>> = SimpleApplicationBuilder::new()
            .modes(hash! {
                "i" => "insert",
            })
            .build();
        if let Err(error) = app.parse_config("examples/main.conf") {
            app.error(&error.to_string());
        }
        app.add_variable("url", || "http://duckduckgo.com/lite".to_string());

        //connect!(app.app, connect_command(command), app, handle_command(command));

        entry.grab_focus();

        app
    }
}*/

fn main() {
    Win::run(()).unwrap();
}
