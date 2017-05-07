/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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

pub mod dialog;
pub mod settings;
mod shortcut;
pub mod status_bar;

use std::borrow::Cow;
use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use futures_glib::Timeout;
use gdk::enums::key::Key as GdkKey;
use gdk::{EventKey, RGBA};
use gdk::enums::key::{Escape, colon};
use glib::translate::ToGlib;
use gtk;
use gtk::{
    BoxExt,
    Inhibit,
    OrientableExt,
    PackType,
    Settings,
    TreeSelection,
    WidgetExt,
    WindowExt,
    STATE_FLAG_NORMAL,
};
use gtk::Orientation::Vertical;
use mg_settings::{self, Config, EnumFromStr, EnumMetaData, Parser, SettingCompletion, SpecialCommand};
use mg_settings::Command::{self, App, Custom, Map, Set, Unmap};
use mg_settings::error::{Error, Result};
use mg_settings::error::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use mg_settings::key::Key;
use relm::{Relm, Widget};
use relm_attributes::widget;

use app::shortcut::shortcut_to_string;
use completion::{
    CommandCompleter,
    Completer,
    CompletionView,
    SettingCompleter,
    DEFAULT_COMPLETER_IDENT,
    NO_COMPLETER_IDENT,
};
use self::ActivationType::{Current, Final};
use self::ShortcutCommand::{Complete, Incomplete};
use self::status_bar::StatusBar;
use self::status_bar::Msg::*;
use self::Msg::*;
pub use self::status_bar::StatusBarItem;
use super::{Modes, Variables};

#[derive(PartialEq)]
enum ActivationType {
    Current,
    Final,
}

type Completers = HashMap<String, Box<Completer>>;
type Mappings = HashMap<&'static str, HashMap<Vec<Key>, String>>;
type ModesHash = HashMap<&'static str, &'static str>;

//type Modes = HashMap<String, String>;

/// A command from a map command.
#[derive(Debug)]
enum ShortcutCommand {
    /// A complete command that is to be executed.
    Complete(String),
    /// An incomplete command where the user needs to complete it and press Enter.
    Incomplete(String),
}

const TRANSPARENT: &RGBA = &RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 };

const BLOCKING_INPUT_MODE: &str = "blocking-input";
pub const COMMAND_MODE: &str = "command";
const COMPLETE_NEXT_COMMAND: &str = "complete-next";
const COMPLETE_PREVIOUS_COMMAND: &str = "complete-previous";
const ENTRY_DELETE_NEXT_CHAR: &str = "entry-delete-next-char";
const ENTRY_DELETE_NEXT_WORD: &str = "entry-delete-next-word";
const ENTRY_DELETE_PREVIOUS_WORD: &str = "entry-delete-previous-word";
const ENTRY_END: &str = "entry-end";
const ENTRY_NEXT_CHAR: &str = "entry-next-char";
const ENTRY_NEXT_WORD: &str = "entry-next-word";
const ENTRY_PREVIOUS_CHAR: &str = "entry-previous-char";
const ENTRY_PREVIOUS_WORD: &str = "entry-previous-word";
const ENTRY_SMART_HOME: &str = "entry-smart-home";
const INFO_MESSAGE_DURATION: u64 = 5;
const INPUT_MODE: &str = "input";
const NORMAL_MODE: &str = "normal";

pub struct Model<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    answer: Option<String>,
    choices: Vec<char>,
    close_callback: Option<Box<Fn() + Send + Sync>>,
    completion_shown: bool,
    current_command_mode: char,
    current_mode: String,
    current_shortcut: Vec<Key>,
    entry_shown: bool,
    foreground_color: RGBA,
    initial_error: Option<Error>,
    input_callback: Option<Box<Fn(Option<String>)>>,
    mappings: Mappings,
    message: String,
    mode_label: String,
    modes: ModesHash,
    relm: Relm<Mg<COMM, SETT>>,
    settings: SETT,
    settings_parser: Box<Parser<COMM>>,
    shortcuts: HashMap<Key, String>,
    shortcut_pressed: bool,
    variables: HashMap<String, fn() -> String>,
}

#[allow(missing_docs)]
#[derive(SimpleMsg)]
pub enum Msg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: mg_settings::settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    CustomCommand(COMM),
    EnterCommandMode,
    EnterNormalMode,
    EnterNormalModeAndReset,
    HideColoredMessage(String),
    HideInfo(String),
    KeyPress(GdkKey),
    KeyRelease(GdkKey),
    ModeChanged(String),
    SetMode(&'static str),
    Quit,
    SettingChanged(SETT::Variant),
}

/// An Mg application window contains a status bar where the user can type a command and a central widget.
#[widget]
impl<COMM, SETT> Widget for Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Handle the command entry activate event.
    fn command_activate(&mut self, input: Option<String>) -> Option<Msg<COMM, SETT>> {
        let message =
            if self.model.current_mode == INPUT_MODE || self.model.current_mode == BLOCKING_INPUT_MODE {
                let mut should_reset = false;
                if let Some(ref callback) = self.model.input_callback {
                    self.model.answer = input.clone();
                    callback(input);
                    should_reset = true;
                }
                self.model.input_callback = None;
                self.model.choices.clear();
                if should_reset {
                    Some(EnterNormalModeAndReset)
                }
                else {
                    None
                }
            }
            else {
                self.handle_command(input, true)
            };
        message
    }

    /// Handle the key press event for the command mode.
    #[allow(non_upper_case_globals)]
    fn command_key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                (Some(EnterNormalModeAndReset), Inhibit(false))
            },
            _ => self.handle_shortcut(key),
        }
    }

    /// Handle the key release event for the command mode.
    fn command_key_release(&mut self, _key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        if self.model.current_command_mode != ':' && COMM::is_incremental(self.model.current_command_mode) {
            let command = self.status_bar.widget().get_command();
            if let Some(command) = command {
                let msg = self.handle_special_command(Current, &command);
                return (msg, Inhibit(false));
            }
        }
        (None, Inhibit(false))
    }

    fn default_completers() -> Completers {
        let mut completers: HashMap<_, Box<Completer>> = HashMap::new();
        completers.insert(DEFAULT_COMPLETER_IDENT.to_string(), Box::new(CommandCompleter::<COMM>::new()));
        completers.insert("set".to_string(), Box::new(SettingCompleter::<SETT>::new()));
        completers
    }

    /// Show an alert message to the user.
    pub fn alert(&mut self, message: &str) {
        self.model.message = message.to_string();
        self.status_bar.widget().color_blue();
    }

    /// Show an error to the user.
    pub fn error(&mut self, error: &str) {
        error!("{}", error);
        self.model.message = error.to_string();
        self.status_bar.widget_mut().hide_entry();
        self.status_bar.widget().color_red();
    }

    /// Hide the information message.
    fn hide_colored_message(&mut self, message: &str) {
        if self.model.message == message {
            self.model.message = String::new();
            self.reset_colors();
        }
    }

    /// Hide the information message.
    fn hide_info(&mut self, message: &str) {
        if self.model.message == message {
            self.model.message = String::new();
        }
    }

    /// Show an information message to the user for 5 seconds.
    pub fn info(&mut self, message: &str) {
        info!("{}", message);
        let message = message.to_string();
        self.model.message = message.clone();
        self.reset_colors();

        let timeout = Timeout::new(Duration::from_secs(INFO_MESSAGE_DURATION));
        self.model.relm.connect_exec_ignore_err(timeout, HideInfo(message));
    }

    /// Show a message to the user.
    pub fn message(&mut self, message: &str) {
        self.reset_colors();
        self.model.message = message.to_string();
    }

    /// Show a warning message to the user for 5 seconds.
    pub fn warning(&mut self, message: &str) {
        warn!("{}", message);
        let message = message.to_string();
        self.model.message = message.clone();
        self.status_bar.widget().color_orange();

        let timeout = Timeout::new(Duration::from_secs(INFO_MESSAGE_DURATION));
        self.model.relm.connect_exec_ignore_err(timeout, HideColoredMessage(message));
    }

    /// Handle the command activate event.
    fn handle_command(&mut self, command: Option<String>, activated: bool) -> Option<Msg<COMM, SETT>> {
        if let Some(command) = command {
            if self.model.current_command_mode == ':' {
                let result = self.model.settings_parser.parse_line(&command);
                self.call_command(result, activated)
            }
            else {
                self.handle_special_command(Final, &command)
            }
        }
        else {
            None
        }
    }

    /// Handle a special command activate or key press event.
    fn handle_special_command(&mut self, activation_type: ActivationType, command: &str) -> Option<Msg<COMM, SETT>> {
        if let Ok(special_command) = COMM::identifier_to_command(self.model.current_command_mode, command) {
            if activation_type == Final {
                self.return_to_normal_mode();
            }
            Some(CustomCommand(special_command))
        }
        else {
            None
        }
    }

    /// Hide the command entry and the completion view.
    fn hide_entry_and_completion(&mut self) {
        self.model.completion_shown = false;
        self.model.entry_shown = false;
    }

    fn init_view(&mut self) {
        if let Some(error) = self.model.initial_error.take() {
            self.show_parse_error(error);
        }
    }

    /// Handle the key press event for the input mode.
    #[allow(non_upper_case_globals)]
    fn input_key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                if let Some(ref callback) = self.model.input_callback {
                    callback(None);
                }
                self.model.input_callback = None;
                (Some(EnterNormalModeAndReset), Inhibit(false))
            },
            keyval => {
                if self.handle_input_shortcut(key) {
                    return (None, Inhibit(true));
                }
                else if let Some(character) = char::from_u32(keyval) {
                    if self.model.choices.contains(&character) {
                        let msg = self.set_dialog_answer(&character.to_string());
                        return (msg, Inhibit(true));
                    }
                }
                self.handle_shortcut(key)
            },
        }
    }

    /// Handle the key press event.
    fn key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match self.model.current_mode.as_ref() {
            NORMAL_MODE => self.normal_key_press(key),
            COMMAND_MODE => self.command_key_press(key),
            BLOCKING_INPUT_MODE | INPUT_MODE => self.input_key_press(key),
            _ => self.handle_shortcut(key)
        }
    }

    /// Handle the key release event.
    fn key_release(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match self.model.current_mode.as_ref() {
            COMMAND_MODE => self.command_key_release(key),
            _ => (None, Inhibit(false)),
        }
    }

    // TODO: switch from a &'static str to a String.
    fn model(relm: &Relm<Self>, (user_modes, settings_filename, include_path): (Modes, &'static str, Option<String>))
        -> Model<COMM, SETT>
    {
        let mut settings_parser = Box::new(Parser::new());
        let mut mappings = HashMap::new();
        let mut modes = HashMap::new();
        let mut initial_error = None;
        match parse_config(settings_filename, user_modes, include_path.as_ref().map(String::as_str)) {
            Ok((parser, initial_mappings, initial_modes)) => {
                settings_parser = Box::new(parser);
                mappings = initial_mappings;
                modes = initial_modes;
            },
            Err(error) => {
                initial_error = Some(error);
            }
        }
        // TODO: use &'static str instead of String?
        Model {
            answer: None,
            choices: vec![],
            close_callback: None,
            completion_shown: false,
            current_command_mode: ':',
            current_mode: NORMAL_MODE.to_string(),
            current_shortcut: vec![],
            entry_shown: false,
            foreground_color: RGBA::white(),
            initial_error,
            input_callback: None,
            mappings,
            message: String::new(),
            mode_label: String::new(),
            modes,
            relm: relm.clone(),
            settings: SETT::default(),
            settings_parser,
            shortcuts: HashMap::new(),
            shortcut_pressed: false,
            variables: HashMap::new(),
        }
    }

    /// Handle the key press event for the normal mode.
    #[allow(non_upper_case_globals)]
    fn normal_key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match key.get_keyval() {
            colon => (Some(EnterCommandMode), Inhibit(true)),
            Escape => {
                self.reset();
                self.clear_shortcut();
                self.handle_shortcut(key)
            },
            keyval => {
                let character = keyval as u8 as char;
                if COMM::is_identifier(character) {
                    self.completion_view.widget_mut().set_completer(NO_COMPLETER_IDENT, "");
                    self.set_current_identifier(character);
                    self.set_mode(COMMAND_MODE);
                    self.reset();
                    self.clear_shortcut();
                    self.show_entry();
                    (None, Inhibit(true))
                }
                else {
                    self.handle_shortcut(key)
                }
            },
        }
    }

    /// Handle the escape event.
    fn reset(&mut self) {
        self.reset_colors();
        self.hide_entry_and_completion();
        self.model.message = String::new();
        self.clear_shortcut();
    }

    fn return_to_normal_mode(&mut self) {
        self.hide_entry_and_completion();
        self.set_mode(NORMAL_MODE);
        self.set_current_identifier(':');
    }

    fn set_completer(&self, completer: &str) {
        let command_entry_text = self.status_bar.widget().get_command().unwrap_or_default();
        self.completion_view.widget_mut().set_completer(completer, &command_entry_text);
    }

    /// Set the current (special) command identifier.
    fn set_current_identifier(&mut self, identifier: char) {
        self.model.current_command_mode = identifier;
    }

    /// Set the answer to return to the caller of the dialog.
    fn set_dialog_answer(&mut self, answer: &str) -> Option<Msg<COMM, SETT>> {
        let mut should_reset = false;
        if self.model.current_mode == BLOCKING_INPUT_MODE {
            self.model.answer = Some(answer.to_string());
            gtk::main_quit();
        }
        else if let Some(ref callback) = self.model.input_callback {
            callback(Some(answer.to_string()));
            self.model.choices.clear();
            should_reset = true;
        }
        self.model.input_callback = None;
        if should_reset {
            Some(EnterNormalModeAndReset)
        }
        else {
            None
        }
    }

    /// Set the current mode.
    pub fn set_mode(&mut self, mode: &str) {
        self.model.current_mode = mode.to_string();
        if self.model.current_mode != NORMAL_MODE && self.model.current_mode != COMMAND_MODE &&
            self.model.current_mode != INPUT_MODE && self.model.current_mode != BLOCKING_INPUT_MODE
        {
            self.model.mode_label = self.model.current_mode.clone();
        }
        else {
            self.model.mode_label = String::new();
        }
        self.model.relm.stream().emit(ModeChanged(mode.to_string()));
    }

    /// Get the settings.
    pub fn settings(&self) -> &SETT {
        &self.model.settings
    }

    fn update(&mut self, event: Msg<COMM, SETT>) {
        match event {
            // To be listened to by the user.
            CustomCommand(_) => (),
            EnterCommandMode => {
                self.set_completer(DEFAULT_COMPLETER_IDENT);
                self.set_current_identifier(':');
                self.set_mode(COMMAND_MODE);
                self.reset();
                self.clear_shortcut();
                self.model.completion_shown = true;
                self.show_entry();
            },
            EnterNormalMode => {
                self.return_to_normal_mode();
            },
            EnterNormalModeAndReset => {
                self.return_to_normal_mode();
                self.reset();
                self.clear_shortcut();
            },
            HideColoredMessage(message) => self.hide_colored_message(&message),
            HideInfo(message) => self.hide_info(&message),
            KeyPress(_) | KeyRelease(_) => (),
            ModeChanged(_) | SettingChanged(_) => (),
            SetMode(mode) => self.set_mode(mode),
            Quit => {
                if let Some(ref callback) = self.model.close_callback {
                    callback();
                }
                else {
                    gtk::main_quit();
                }
            },
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            #[container]
            gtk::Box {
                orientation: Vertical,
                #[container="status-bar-item"]
                #[name="status_bar"]
                StatusBar {
                    entry_shown: self.model.entry_shown,
                    identifier: &self.model.current_command_mode.to_string(),
                    packing: {
                        pack_type: PackType::End,
                    },
                    #[name="message"]
                    StatusBarItem {
                        text: &self.model.message,
                        packing: {
                            pack_type: PackType::Start,
                        },
                    },
                    #[name="mode"]
                    StatusBarItem {
                        text: &self.model.mode_label,
                        packing: {
                            pack_type: PackType::Start,
                        },
                    },
                    #[name="shortcut"]
                    StatusBarItem {
                        text: &shortcut_to_string(&self.model.current_shortcut),
                    },
                    EntryActivate(input) => self.command_activate(input),
                    EntryChanged(input) => self.update_completions(input),
                },
                gtk::Overlay {
                    packing: {
                        pack_type: PackType::End,
                    },
                    #[name="completion_view"]
                    CompletionView {
                        completers: Self::default_completers(),
                        visible: self.model.completion_shown,
                    },
                },
            },
            key_press_event(_, key) => return self.key_press(&key),
            key_release_event(_, key) => return self.key_release(&key),
            delete_event(_, _) => (Quit, Inhibit(false)),
        },
    }
}

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Convert an action String to a command String.
    fn action_to_command(&self, action: &str) -> ShortcutCommand {
        if let Some(':') = action.chars().next() {
            if let Some(index) = action.find("<Enter>") {
                Complete(action[1..index].to_string())
            }
            else {
                Incomplete(action[1..].to_string())
            }
        }
        else {
            Complete(action.to_string())
        }
    }

    /// Handle an application command.
    fn app_command(&self, command: &str) {
        match command {
            COMPLETE_NEXT_COMMAND => {
                if let (Some(selection), unselect) = self.completion_view.widget().select_next() {
                    self.selection_changed(&selection);
                    if unselect {
                        self.handle_unselect();
                    }
                }
            },
            COMPLETE_PREVIOUS_COMMAND => {
                if let (Some(selection), unselect) = self.completion_view.widget().select_previous() {
                    self.selection_changed(&selection);
                    if unselect {
                        self.handle_unselect();
                    }
                }
            },
            ENTRY_DELETE_NEXT_CHAR => {
                self.status_bar.widget().delete_next_char();
                self.update_completions(self.status_bar.widget().get_command());
            },
            ENTRY_DELETE_NEXT_WORD => {
                self.status_bar.widget().delete_next_word();
                self.update_completions(self.status_bar.widget().get_command());
            },
            ENTRY_DELETE_PREVIOUS_WORD => {
                self.status_bar.widget().delete_previous_word();
                self.update_completions(self.status_bar.widget().get_command());
            },
            ENTRY_END => self.status_bar.widget().end(),
            ENTRY_NEXT_CHAR => self.status_bar.widget().next_char(),
            ENTRY_NEXT_WORD => self.status_bar.widget().next_word(),
            ENTRY_PREVIOUS_CHAR => self.status_bar.widget().previous_char(),
            ENTRY_PREVIOUS_WORD => self.status_bar.widget().previous_word(),
            ENTRY_SMART_HOME => self.status_bar.widget().smart_home(),
            _ => unreachable!(),
        }
    }

    /// Call the callback with the command or show an error if the command cannot be parsed.
    fn call_command(&mut self, command: Result<Command<COMM>>, activated: bool) -> Option<Msg<COMM, SETT>> {
        match command {
            Ok(command) => {
                match command {
                    App(command) => self.app_command(&command),
                    Custom(command) => {
                        if activated {
                            self.return_to_normal_mode();
                        }
                        return Some(CustomCommand(command));
                    },
                    Set(name, value) => {
                        match SETT::to_variant(&name, value) {
                            Ok(setting) => {
                                self.set_setting(setting);
                            },
                            Err(error) => {
                                let message = format!("Error setting value: {}", error);
                                self.error(&message);
                            },
                        }
                        return Some(EnterNormalMode);
                    },
                    _ => unimplemented!(),
                }
            },
            Err(error) => {
                self.show_parse_error(error);
                return Some(EnterNormalMode);
            },
        }
        None
    }

    /// Connect the close event to the specified callback.
    pub fn connect_close<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.model.close_callback = Some(Box::new(callback));
    }

    /// Delete the current completion item.
    pub fn delete_current_completion_item(&self) {
        self.completion_view.widget().delete_current_completion_item();
    }

    /// Get the color of the text.
    fn get_foreground_color(&self) -> RGBA {
        let style_context = self.window.get_style_context().unwrap();
        style_context.get_color(STATE_FLAG_NORMAL)
    }

    /// Handle the unselect event.
    fn handle_unselect(&self) {
        let completion_view = self.completion_view.widget();
        let original_input = completion_view.original_input();
        self.set_input(original_input);
    }

    /// Input the specified command.
    fn input_command(&mut self, command: &str) {
        self.set_mode(COMMAND_MODE);
        self.show_entry();
        let mut command = command.to_string();
        for (variable, function) in &self.model.variables {
            command = command.replace(&format!("<{}>", variable), &function());
        }
        let text: Cow<str> =
            if command.contains(' ') {
                command.into()
            }
            else {
                format!("{} ", command).into()
            };
        self.status_bar.widget().set_input(&text);
    }

    /// Reset the background and foreground colors of the status bar.
    fn reset_colors(&self) {
        let status_bar = self.status_bar.widget().root();
        // TODO: switch to CSS.
        status_bar.override_background_color(STATE_FLAG_NORMAL, TRANSPARENT);
        status_bar.override_color(STATE_FLAG_NORMAL, &self.model.foreground_color);
    }

    /// Handle the selection changed event.
    fn selection_changed(&self, selection: &TreeSelection) -> Option<Msg<COMM, SETT>> {
        if let Some(completion) = self.completion_view.widget().complete_result(selection) {
            self.set_input(&completion);
            //return Some(SetInput(completion); // TODO
        }
        None
    }

    /// Use the dark variant of the theme if available.
    pub fn set_dark_theme(&mut self, use_dark: bool) {
        let settings = Settings::get_default().unwrap();
        settings.set_long_property("gtk-application-prefer-dark-theme", use_dark.to_glib() as _, "");
        self.model.foreground_color = self.get_foreground_color();
    }

    fn set_input(&self, original_input: &str) {
        self.status_bar.widget().set_input(original_input);
    }

    /// Set a setting value.
    pub fn set_setting(&mut self, setting: SETT::Variant) {
        self.model.settings.set_value(setting.clone());
        self.model.relm.stream().emit(SettingChanged(setting));
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Set the variables that will be available in the settings.
    /// A variable can be used in mappings.
    /// The placeholder will be replaced by the value returned by the function.
    pub fn set_variables(&mut self, variables: Variables) {
        self.model.variables = variables.iter()
            .map(|&(string, func)| (string.to_string(), func))
            .collect();
    }

    fn show_entry(&mut self) {
        self.model.entry_shown = true;
        let mut status_bar = self.status_bar.widget_mut();
        status_bar.set_entry_shown(true);
        status_bar.show_entry();
    }

    fn show_parse_error(&mut self, error: Error) {
        match error {
            Error::Parse(error) => {
                let message =
                    match error.typ {
                        MissingArgument => "Argument required".to_string(),
                        NoCommand => return,
                        Parse => format!("Parse error: unexpected {}, expecting: {}", error.unexpected, error.expected),
                        UnknownCommand => format!("Not a command: {}", error.unexpected),
                    };
                self.error(&message);
            },
            Error::Io(error) => self.error(&format!("{}", error)),
        }
    }

    fn update_completions(&self, input: Option<String>) -> Option<Msg<COMM, SETT>> {
        let input = input.unwrap_or_default();
        self.completion_view.widget_mut().update_completions(&self.model.current_mode, &input);
        None
    }
}

/// Parse a configuration file.
pub fn parse_config<P: AsRef<Path>, COMM: EnumFromStr>(filename: P, user_modes: Modes, include_path: Option<&str>)
    -> Result<(Parser<COMM>, Mappings, ModesHash)>
{
    let mut mappings = HashMap::new();
    let file = File::open(filename)?;
    let buf_reader = BufReader::new(file);

    let mut modes = HashMap::new();
    for &(key, value) in user_modes {
        modes.insert(key, value);
    }
    assert!(modes.insert("n", NORMAL_MODE).is_none(), "Duplicate mode prefix n.");
    assert!(modes.insert("c", COMMAND_MODE).is_none(), "Duplicate mode prefix c.");
    let config = Config {
        application_commands: vec![COMPLETE_NEXT_COMMAND, COMPLETE_PREVIOUS_COMMAND, ENTRY_DELETE_NEXT_CHAR,
            ENTRY_DELETE_NEXT_WORD, ENTRY_DELETE_PREVIOUS_WORD, ENTRY_END, ENTRY_NEXT_CHAR, ENTRY_NEXT_WORD,
            ENTRY_PREVIOUS_CHAR, ENTRY_PREVIOUS_WORD, ENTRY_SMART_HOME],
        mapping_modes: modes.keys().cloned().collect(),
    };
    let mut parser = Parser::new_with_config(config);
    if let Some(include_path) = include_path {
        parser.set_include_path(include_path);
    }

    let commands = parser.parse(buf_reader)?;
    for command in commands {
        match command {
            //App(command) => self.app_command(&command),
            //Custom(command) => {
            //if let Some(ref callback) = self.command_callback {
            //callback(command);
            //}
            //},
            Map { action, keys, mode } => {
                let mode_mappings = mappings.entry(modes[mode.as_str()]).or_insert_with(HashMap::new);
                mode_mappings.insert(keys, action);
            },
            _ => (), // TODO: to remove.
            //Set(name, value) => self.set_setting(Sett::to_variant(&name, value)?),
            Unmap { .. } => panic!("not yet implemented"), // TODO
        }
    }
    Ok((parser, mappings, modes))
}
