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
extern crate pretty_env_logger;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{ButtonExt, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{
    AppClose,
    CustomCommand,
    Mg,
    Modes,
    ModeChanged,
    SetMode,
    SettingChanged,
    StatusBarItem,
    Variables,
};
use relm::{Relm, Widget};
use relm_attributes::widget;

use AppSettingsVariant::*;

use AppCommand::*;
use Msg::*;

pub struct Model {
    relm: Relm<Win>,
    text: String,
    title: String,
}

#[derive(Msg)]
pub enum Msg {
    Alert,
    CheckQuit(Option<String>),
    Command(AppCommand),
    Echo(Option<String>),
    Info,
    Input,
    Mode(String),
    Question,
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
    fn init_view(&mut self) {
        self.entry.grab_focus();
    }

    fn model(relm: &Relm<Self>, _model: ()) -> Model {
        Model {
            relm: relm.clone(),
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
            CheckQuit(answer) => {
                if answer == Some("y".to_string()) {
                    gtk::main_quit();
                }
            },
            Command(command) => {
                match command {
                    BackwardSearch(input) => println!("Searching backward for {}", input),
                    DeleteEntry => self.mg.widget().delete_current_completion_item(),
                    Follow => (),
                    Insert | Normal => (),
                    Open(url) => self.model.text = format!("Opening URL {}", url),
                    Quit => gtk::main_quit(),
                    Search(input) => println!("Searching for {}", input),
                    ShowCount(count) => println!("Count: {}", count),
                    WinOpen(url) => self.model.text = format!("Opening URL {} in new window", url),
                }
            },
            Echo(answer) => self.model.text = format!("You said: {}", answer.unwrap_or("Nothing".to_string())),
            Info => self.mg.widget_mut().info("Info"),
            Input => self.mg.widget_mut().input(&self.model.relm, "Say something", "Oh yeah?", Echo),
            Mode(mode) => self.mode_changed(&mode),
            Question => self.mg.widget_mut().question(&self.model.relm, "Do you want to quit?", &['y', 'n'], CheckQuit),
            Setting(setting) => self.setting_changed(setting),
            Warning => self.mg.widget_mut().warning("Warning"),
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>((MODES, Ok("examples/main.conf".into()), Some("/home/bouanto".into()), vec![])) {
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
                gtk::Button {
                    label: "Question",
                    clicked => Question,
                },
                gtk::Button {
                    label: "Input",
                    clicked => Input,
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
            CustomCommand(command) => Command(command),
            ModeChanged(mode) => Mode(mode),
            SettingChanged(setting) => Setting(setting),
        }
    }
}

fn main() {
    pretty_env_logger::init().unwrap();
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
    DeleteEntry,
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
