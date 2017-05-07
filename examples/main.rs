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

use gtk::{ButtonExt, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{
    CustomCommand,
    Mg,
    Modes,
    ModeChanged,
    SettingChanged,
    StatusBarItem,
    Variables,
};
use relm::Widget;
use relm_attributes::widget;

use AppSettingsVariant::*;

use AppCommand::*;
use Msg::*;

pub struct Model {
    text: String,
    title: String,
}

#[derive(Msg)]
pub enum Msg {
    Alert,
    Command(AppCommand),
    Info,
    Mode(String),
    Setting(AppSettingsVariant),
    Warning,
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
            {
                let widget = self.mg.widget();
                let title = &widget.settings().title;
                self.model.text = format!("Title was: {}", title);
            }
            self.mg.widget_mut().set_setting(Title(mode.to_string()));
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
            Alert => self.mg.widget_mut().alert("Blue Alert"),
            Command(command) => {
                match command {
                    BackwardSearch(input) => println!("Searching backward for {}", input),
                    Follow => (),
                    Insert => self.mg.widget_mut().set_mode("insert"),
                    Normal => self.mg.widget_mut().set_mode("normal"),
                    Open(url) => self.model.text = format!("Opening URL {}", url),
                    Quit => gtk::main_quit(),
                    Search(input) => println!("Searching for {}", input),
                    ShowCount(count) => println!("Count: {}", count),
                    WinOpen(url) => self.model.text = format!("Opening URL {} in new window", url),
                }
            },
            Info => self.mg.widget_mut().info("Info"),
            Mode(mode) => self.mode_changed(&mode),
            Setting(setting) => self.setting_changed(setting),
            Warning => self.mg.widget_mut().warning("Warning"),
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
                gtk::Button {
                    label: "Alert",
                    clicked => Alert,
                },
                gtk::Button {
                    label: "Info",
                    clicked => Info,
                },
                gtk::Button {
                    label: "Warning",
                    clicked => Warning,
                },
            },
            StatusBarItem {
                text: "Rightmost",
            },
            StatusBarItem {
                text: "Test",
            },
            CustomCommand(command) => Command(command),
            ModeChanged(mode) => Mode(mode),
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
    #[special_command(identifier="?")]
    BackwardSearch(String),
    #[completion(hidden)]
    Follow,
    Insert,
    Normal,
    #[help(text="Open the url")]
    Open(String),
    #[help(text="Quit the application")]
    Quit,
    #[special_command(incremental, identifier="/")]
    Search(String),
    #[count]
    ShowCount(i32),
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
