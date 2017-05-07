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

#![feature(closure_to_fn_coercion, proc_macro)]

extern crate gtk;
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{
    CustomCommand,
    Mg,
    Modes,
    SettingChanged,
    StatusBarItem,
    Variables,
};
use relm::Widget;
use relm_attributes::widget;

use AppSettingsVariant::*;
//use SpecialCommand::*;

use AppCommand::*;
use Msg::*;

pub struct Model {
    text: String,
    title: String,
}

#[derive(Msg)]
pub enum Msg {
    Command(AppCommand),
    Setting(AppSettingsVariant),
}

static MODES: Modes = &[
    ("i", "insert"),
];

static VARIABLES: Variables = &[
    ("url", || "http://duckduckgo.com/lite".to_string()),
];

#[widget]
impl Widget for Win {
    fn init_view(&self) {
        self.entry.grab_focus();
    }

    fn model() -> Model {
        Model {
            text: "Mg App".to_string(),
            title: "First Mg Program".to_string(),
        }
    }

    fn mode_changed(&mut self, mode: &str) {
        if mode != "normal" {
            /*{
                let title = &self.app.settings().title;
                self.model.text = format!("Title was: {}", title);
            }
            self.app.set_setting(Title(mode.to_string()));*/
        }
    }

    fn setting_changed(&mut self, setting: AppSettingsVariant) {
        match setting {
            CustomSet(setting) => self.model.text = format!("custom setting is: {:?}", setting),
            Title(title) => self.model.title = title,
            TitleLen(len) => self.model.title = format!("New title len: {}", len),
            Boolean(_) | Width(_) => (),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Command(command) => {
                match command {
                    Follow => (),
                    Insert => self.mg.widget_mut().set_mode("insert"),
                    Normal => self.mg.widget_mut().set_mode("normal"),
                    Open(url) => self.model.text = format!("Opening URL {}", url),
                    WinOpen(url) => self.model.text = format!("Opening URL {} in new window", url),
                    Quit => gtk::main_quit(),
                }
            },
            Setting(setting) => self.setting_changed(setting),
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>((MODES, "examples/main.conf")) {
            dark_theme: true,
            title: &self.model.title,
            variables: VARIABLES,
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
            CustomCommand(command) => Command(command),
            SettingChanged(setting) => Setting(setting),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[derive(Clone, Debug, Setting)]
pub enum CustomSetting {
    #[default]
    Choice,
    OtherChoice,
}

#[derive(Commands)]
pub enum AppCommand {
    #[completion(hidden)]
    Follow,
    Insert,
    Normal,
    #[help(text="Open the url")]
    Open(String),
    #[help(text="Quit the application")]
    Quit,
    #[help(text="Open the url in a new window")]
    WinOpen(String),
}

#[derive(Default, Settings)]
pub struct AppSettings {
    boolean: bool,
    custom_set: CustomSetting,
    #[help(text="The window title")]
    title: String,
    title_len: i64,
    width: i64,
}

/*special_commands!(SpecialCommand {
    BackwardSearch('?', always),
    Search('/', always),
});

impl App {
    fn new() -> Box<Self> {
        connect!(app.app, connect_mode_changed(mode), app, mode_changed(mode));
        connect!(app.app, connect_special_command(command), app, handle_special_command(command));

        app
    }

    fn handle_special_command(&self, command: SpecialCommand) {
        match command {
            BackwardSearch(input) => println!("Searching backward for {}", input),
            Search(input) => println!("Searching for {}", input),
        }
    }
}*/
