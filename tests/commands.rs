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

extern crate gtk;
extern crate libxdo;
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

mod utils;

use std::thread;

use gtk::WidgetExt;
use libxdo::XDo;
use mg::{
    CustomCommand,
    Mg,
    Modes,
};
use relm::{Widget, init_test};
use relm_attributes::widget;

use self::AppCommand::*;
use self::Msg::*;
use utils::XDoExt;

#[derive(Commands)]
pub enum AppCommand {
    #[help(text="Show the text in the label")]
    Show(String),
    Quit,
}

#[derive(Default, Settings)]
pub struct AppSettings {
    boolean: bool,
}

pub struct Model {
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    Command(AppCommand),
}

static MODES: Modes = &[
    ("i", "insert"),
];

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            text: "Label".to_string(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Command(command) => {
                match command {
                    Show(text) => self.model.text = format!("Showing text: {}", text),
                    Quit => gtk::main_quit(),
                }
            },
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>((MODES, "examples/main.conf")) {
            #[name="label"]
            gtk::Label {
                text: &self.model.text,
            },
            CustomCommand(command) => Command(command),
        }
    }
}

#[test]
fn test_basic_command() {
    gtk::init().unwrap();

    let win = init_test::<Win>(()).unwrap();

    assert_eq!(Some("Label".to_string()), win.widget().label.get_text());

    thread::spawn(|| {
        let xdo = XDo::new(None).unwrap();
        xdo.enter_command("show test");
        xdo.enter_command("quit");
    });

    gtk::main();

    assert_eq!(Some("Showing text: test".to_string()), win.widget().label.get_text());
}
