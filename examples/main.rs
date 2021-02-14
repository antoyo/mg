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

extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate pretty_env_logger;
extern crate relm;
#[macro_use]
extern crate relm_derive;

use gdk::RGBA;
use gtk::{ButtonExt, LabelExt, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{
    Alert,
    AppClose,
    Color,
    CustomCommand,
    DarkTheme,
    DeleteCompletionItem,
    DialogBuilder,
    Info,
    Mg,
    Mode,
    Modes,
    ModeChanged,
    SetMode,
    SetSetting,
    SettingChanged,
    StatusBarItem,
    StatusBarVisible,
    Title,
    Variables,
    Warning,
    blocking_dialog,
    input,
    question,
};
use relm::{Relm, Widget};
use relm_derive::widget;

use AppSettingsVariant::*;

use AppCommand::*;
use Msg::*;

pub struct Model {
    relm: Relm<Win>,
    statusbar_visible: bool,
    text: String,
    title: String,
}

#[derive(Msg)]
pub enum Msg {
    CheckQuit(Option<String>),
    Command(AppCommand),
    Echo(Option<String>),
    NewMode(String),
    Setting(AppSettingsVariant),
    ShowAlert,
    ShowBlockingInput,
    ShowInfo,
    ShowInput,
    ShowQuestion,
    ShowWarning,
    ToggleStatusBar,
}

static MODES: Modes = &[
    Mode { name: "foo", prefix: "f", show_count: true },
    Mode { name: "insert", prefix: "i", show_count: false },
];

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.widgets.entry.grab_focus();
    }

    fn model(relm: &Relm<Self>, _model: ()) -> Model {
        Model {
            relm: relm.clone(),
            statusbar_visible: true,
            text: "Mg App".to_string(),
            title: "First Mg Program".to_string(),
        }
    }

    fn mode_changed(&mut self, mode: &str) {
        if mode != "normal" {
            self.model.text = format!("Title was: {}", self.model.title);
            self.streams.mg.emit(SetSetting(WindowTitle(mode.to_string())));
        }
    }

    fn setting_changed(&mut self, setting: AppSettingsVariant) {
        match setting {
            CustomSet(setting) => self.model.text = format!("custom setting is: {:?}", setting),
            WindowTitle(title) => self.model.title = title,
            TitleLen(len) => self.model.title = format!("New title len: {}", len),
            Boolean(_) | Width(_) => println!("boolean or width"),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            CheckQuit(answer) => {
                if answer == Some("y".to_string()) {
                    gtk::main_quit();
                }
            },
            Command(command) => {
                match command {
                    BackwardSearch(input) => println!("Searching backward for {}", input),
                    DeleteEntry => self.streams.mg.emit(DeleteCompletionItem),
                    Follow => (),
                    Insert | Normal => (),
                    Open(url) => self.model.text = format!("Opening URL {}", url),
                    Quit => gtk::main_quit(),
                    Search(input) => println!("Searching for {}", input),
                    ShowCount(count) => println!("Count: {:?}", count),
                    WinOpen(url) => self.model.text = format!("Opening URL {} in new window", url),
                }
            },
            Echo(answer) => self.model.text = format!("You said: {}", answer.unwrap_or("Nothing".to_string())),
            NewMode(mode) => self.mode_changed(&mode),
            Setting(setting) => self.setting_changed(setting),
            ShowAlert => self.streams.mg.emit(Alert("Blue Alert".to_string())),
            ShowBlockingInput => {
                let builder = DialogBuilder::new()
                    .message("Do you wanto to quit?".to_string());
                let res = blocking_dialog(&self.streams.mg, builder);
                if res == Some("yes".to_string()) {
                    gtk::main_quit();
                }
            },
            ShowInfo => self.streams.mg.emit(Info("Info".to_string())),
            ShowInput => input(&self.streams.mg, &self.model.relm, "Say something".to_string(),
                "Oh yeah?".to_string(), Echo),
            ShowQuestion => question(&self.streams.mg, &self.model.relm, "Do you want to quit?".to_string(),
                char_slice!['y', 'n'], CheckQuit),
            ShowWarning => self.streams.mg.emit(Warning("Warning".to_string())),
            ToggleStatusBar => self.model.statusbar_visible = !self.model.statusbar_visible,
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>(MODES, Ok("examples/main.conf".into()), Some("/home/bouanto".into()), vec![]) {
            DarkTheme: true,
            StatusBarVisible: self.model.statusbar_visible,
            Title: self.model.title.clone(),
            Variables: vec![("url", Box::new(|| "http://duckduckgo.com/lite".to_string()))],
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
                    clicked => ShowAlert,
                },
                gtk::Button {
                    label: "Info",
                    clicked => ShowInfo,
                },
                gtk::Button {
                    label: "Warning",
                    clicked => ShowWarning,
                },
                gtk::Button {
                    label: "Question",
                    clicked => ShowQuestion,
                    focus_on_click: false,
                },
                gtk::Button {
                    label: "Input",
                    clicked => ShowInput,
                },
                gtk::Button {
                    label: "Blocking Input",
                    clicked => ShowBlockingInput,
                },
                gtk::Button {
                    label: "Toggle status bar",
                    clicked => ToggleStatusBar,
                },
            },
            StatusBarItem {
                Color: Some(RGBA::red()),
                text: "Rightmost",
            },
            StatusBarItem {
                text: "Test",
            },
            AppClose => Command(Quit),
            CustomCommand(Insert) => mg@SetMode("insert"),
            CustomCommand(Normal) => mg@SetMode("normal"),
            CustomCommand(ref command) => Command(command.clone()),
            ModeChanged(ref mode) => NewMode(mode.clone()),
            SettingChanged(ref setting) => Setting(setting.clone()),
        }
    }
}

fn main() {
    pretty_env_logger::init();
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
    ShowCount(Option<u32>),
    #[help(text="Open the url in a new window")]
    WinOpen(String),
}

#[derive(Default, Settings)]
pub struct AppSettings {
    boolean: bool,
    custom_set: CustomSetting,
    #[help(text="The window title")]
    title_len: i64,
    width: i64,
    window_title: String,
}
