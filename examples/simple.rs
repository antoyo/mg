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

extern crate gtk;
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate relm;
#[macro_use]
extern crate relm_derive;

use gtk::{
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use mg::{
    AppClose,
    CustomCommand,
    DarkTheme,
    Mg,
    NoSettings,
    SetMode,
    StatusBarItem,
    Title,
    Variables,
};
use relm::Widget;
use relm_derive::widget;

use AppCommand::*;
use Msg::*;

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

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.widgets.entry.grab_focus();
    }

    fn model() -> Model {
        Model {
            text: "Mg App".to_string(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Command(command) => {
                match command {
                    Insert | Normal => (),
                    Open(url) => self.model.text = format!("Opening URL {}", url),
                    Quit => gtk::main_quit(),
                }
            },
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, NoSettings>(&[], Ok("examples/main.conf".into()), None, vec![]) {
            DarkTheme: true,
            Title: "First Mg Program".to_string(),
            Variables: vec![("url", Box::new(|| "http://duckduckgo.com/lite".to_string()))],
            gtk::Box {
                orientation: Vertical,
                gtk::Label {
                    text: &self.model.text,
                },
                #[name="entry"]
                gtk::Entry {
                },
            },
            StatusBarItem {
                text: "Rightmost",
            },
            StatusBarItem {
                text: "Test",
            },
            AppClose => Command(Quit),
            CustomCommand(Insert) => mg@SetMode("insert"),
            CustomCommand(Normal) => mg@SetMode("normal"),
            CustomCommand(ref command) => Command(command.clone()),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
