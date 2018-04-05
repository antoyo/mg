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

mod app_completion;
mod color;
mod command;
mod config;
pub mod dialog;
mod keypress;
pub mod settings;
mod shortcut;
pub mod status_bar;

use std::cell::Cell;
use std::char;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use gdk::{EventKey, RGBA};
use gdk::enums::key::{Escape, colon};
use gtk;
use gtk::{
    BoxExt,
    GtkWindowExt,
    Inhibit,
    OrientableExt,
    PackType,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use mg_settings::{
    self,
    EnumFromStr,
    EnumMetaData,
    Parser,
    SettingCompletion,
    SpecialCommand,
};
use mg_settings::ParseResult;
use mg_settings::errors;
use mg_settings::key::Key;
use relm::{Relm, Widget, timeout};
use relm_attributes::widget;

use app::config::create_default_config;
pub use app::config::parse_config;
use app::dialog::Responder;
use app::settings::DefaultConfig;
use app::shortcut::shortcut_to_string;
use completion::{
    self,
    CompletionView,
    DEFAULT_COMPLETER_IDENT,
    NO_COMPLETER_IDENT,
};
use completion::completion_view::Msg::{
    AddCompleters,
    Completer,
    CompletionChange,
    Visible,
};
use self::color::{color_blue, color_orange, color_red};
use self::dialog::DialogBuilder;
use self::status_bar::StatusBar;
use self::status_bar::Msg::{
    EntryActivate,
    EntryChanged,
    EntryShown,
    EntryText,
    Identifier,
};
use self::status_bar::ItemMsg::Text;
use self::Msg::*;
pub use self::status_bar::StatusBarItem;
use super::Modes;

type Mappings = HashMap<&'static str, HashMap<Vec<Key>, String>>;
type ModesHash = HashMap<&'static str, super::Mode>;
type Variables = Vec<(&'static str, Box<Fn() -> String>)>;

/// A known mode or an unknown mode.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    BlockingInput,
    Command,
    Input,
    Normal,
    Unknown,
}

/// A command from a map command.
#[derive(Debug)]
pub enum ShortcutCommand {
    /// A complete command that is to be executed.
    Complete(String),
    /// An incomplete command where the user needs to complete it and press Enter.
    Incomplete(String),
}

const BLOCKING_INPUT_MODE: &str = "blocking-input";
pub const COMMAND_MODE: &str = "command";
const COMPLETE_NEXT_COMMAND: &str = "complete-next";
const COMPLETE_PREVIOUS_COMMAND: &str = "complete-previous";
const COPY: &str = "entry-copy";
const CUT: &str = "entry-cut";
const ENTRY_DELETE_NEXT_CHAR: &str = "entry-delete-next-char";
const ENTRY_DELETE_NEXT_WORD: &str = "entry-delete-next-word";
const ENTRY_DELETE_PREVIOUS_WORD: &str = "entry-delete-previous-word";
const ENTRY_END: &str = "entry-end";
const ENTRY_NEXT_CHAR: &str = "entry-next-char";
const ENTRY_NEXT_WORD: &str = "entry-next-word";
const ENTRY_PREVIOUS_CHAR: &str = "entry-previous-char";
const ENTRY_PREVIOUS_WORD: &str = "entry-previous-word";
const ENTRY_SMART_HOME: &str = "entry-smart-home";
const INFO_MESSAGE_DURATION: u32 = 5000;
const INPUT_MODE: &str = "input";
const NORMAL_MODE: &str = "normal";
const PASTE: &str = "entry-paste";
const PASTE_SELECTION: &str = "entry-paste-selection";

#[derive(PartialEq)]
pub enum ActivationType {
    Current,
    Final,
}

pub struct Model<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    answer: Option<String>,
    choices: Vec<char>,
    completer: String,
    completion_shown: bool,
    current_command_mode: char,
    current_mode: Rc<Cell<Mode>>,
    current_shortcut: Vec<Key>,
    entry_shown: bool,
    foreground_color: RGBA,
    initial_errors: Vec<errors::Error>,
    initial_parse_result: Option<ParseResult<COMM>>,
    input_callback: Option<Box<Fn(Option<String>, bool)>>,
    mappings: Mappings,
    message: String,
    mode_label: String,
    mode_string: String,
    modes: ModesHash,
    relm: Relm<Mg<COMM, SETT>>,
    settings: SETT,
    settings_parser: Box<Parser<COMM>>,
    shortcuts: HashMap<Key, String>,
    shortcut_pressed: bool,
    show_count: bool,
    status_bar_command: String,
    variables: HashMap<String, Box<Fn() -> String>>,
}

#[allow(missing_docs)]
#[derive(Msg)]
pub enum Msg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + mg_settings::settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    Alert(String),
    AppClose,
    BlockingCustomDialog(Box<Responder>, DialogBuilder),
    BlockingInput(Box<Responder>, String, String),
    BlockingQuestion(Box<Responder>, String, Vec<char>),
    BlockingYesNoQuestion(Box<Responder>, String),
    CloseWin,
    Completers(HashMap<&'static str, Box<completion::Completer>>),
    CompletionViewChange(String),
    CustomCommand(COMM),
    CustomDialog(DialogBuilder),
    DarkTheme(bool),
    DeleteCompletionItem,
    EnterCommandMode,
    EnterNormalMode,
    EnterNormalModeAndReset,
    Error(errors::Error),
    HideColoredMessage(String),
    HideInfo(String),
    Info(String),
    InitAfter,
    Input(Box<Responder>, String, String),
    KeyPress(EventKey),
    KeyRelease(EventKey),
    Message(String),
    ModeChanged(String),
    Question(Box<Responder>, String, &'static [char]),
    ResetInput,
    SetMode(&'static str),
    SetSetting(SETT::Variant),
    SettingChanged(SETT::Variant),
    StatusBarEntryActivate(Option<String>),
    StatusBarEntryChanged(Option<String>),
    Title(String),
    Variables(Variables),
    Warning(String),
    YesNoQuestion(Box<Responder>, String),
}

/// An Mg application window contains a status bar where the user can type a command and a central widget.

#[widget]
impl<COMM, SETT> Widget for Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    fn after_children_added(&mut self) {
        // NOTE: This code is not in init_view() because the SettingChanged signal would be sent
        // before the user's code connected to this event.
        let parse_result = self.model.initial_parse_result.take().expect("initial parse result");
        self.execute_commands(parse_result, false);
        let errors: Vec<_> = self.model.initial_errors.drain(..).collect();
        for error in errors {
            self.error(error);
        }
    }

    /// Show an alert message to the user.
    fn alert(&mut self, message: &str) {
        self.model.message = message.to_string();
        color_blue(self.status_bar.widget());
    }

    /// Show an error to the user.
    fn error(&mut self, error: errors::Error) {
        let mut message = String::new();
        let error_str = error.to_string();
        message.push_str(&error_str);
        error!("{}", message);

        self.model.message = error_str;
        self.model.entry_shown = false;
        color_red(self.status_bar.widget());
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
    fn info(&mut self, message: &str) {
        info!("{}", message);
        let message = message.to_string();
        self.model.message = message.clone();
        self.reset_colors();

        timeout(self.model.relm.stream(), INFO_MESSAGE_DURATION, move || HideInfo(message.clone()));
    }

    /// Show a message to the user.
    fn message(&mut self, message: &str) {
        self.reset_colors();
        self.model.message = message.to_string();
    }

    /// Show a warning message to the user for 5 seconds.
    fn warning(&mut self, message: &str) {
        warn!("{}", message);
        let message = message.to_string();
        self.model.message = message.clone();
        color_orange(self.status_bar.widget());

        timeout(self.model.relm.stream(), INFO_MESSAGE_DURATION, move || HideColoredMessage(message.clone()));
    }

    /// Hide the command entry and the completion view.
    fn hide_entry_and_completion(&mut self) {
        self.model.completion_shown = false;
        self.model.entry_shown = false;
    }

    /// Check if the key should be inhibitted for a normal mode.
    #[allow(non_upper_case_globals)]
    fn inhibit_normal_key_press(current_mode: &Rc<Cell<Mode>>, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            colon | Escape => Inhibit(true),
            keyval => {
                let character = keyval as u8 as char;
                if COMM::is_identifier(character) {
                    Inhibit(true)
                }
                else {
                    Self::inhibit_handle_shortcut(current_mode, key)
                }
            },
        }
    }

    fn init_view(&mut self) {
        self.model.foreground_color = self.get_foreground_color();
        self.model.relm.stream().emit(InitAfter);
    }

    /// Input the specified command.
    fn input_command(&mut self, mut command: String) {
        self.set_mode(COMMAND_MODE);
        self.show_entry();
        for (variable, function) in &self.model.variables {
            command = command.replace(&format!("<{}>", variable), &function());
        }
        if !command.contains(' ') {
            command.push(' ');
        }
        self.model.status_bar_command = command;
    }

    fn model(relm: &Relm<Self>, (user_modes, settings_filename, include_path, default_config):
             (Modes, io::Result<PathBuf>, Option<PathBuf>, Vec<DefaultConfig>)) -> Model<COMM, SETT>
    {
        let mut initial_errors = vec![];
        if let Err(error) = create_default_config(default_config) {
            initial_errors.push(error.into());
        }
        let (settings_parser, initial_parse_result, modes) =
            match settings_filename {
                Ok(settings_filename) => {
                    let (parser, parse_result, modes) = parse_config(settings_filename, user_modes, include_path);
                    (Box::new(parser), Some(parse_result), modes)
                },
                Err(error) => {
                    initial_errors.push(error.into());
                    (Box::new(Parser::<COMM>::new()), None, HashMap::new())
                },
            };
        Model {
            answer: None,
            choices: vec![],
            completer: DEFAULT_COMPLETER_IDENT.to_string(),
            completion_shown: false,
            current_command_mode: ':',
            current_mode: Rc::new(Cell::new(Mode::Normal)),
            current_shortcut: vec![],
            entry_shown: false,
            foreground_color: RGBA::white(),
            initial_errors,
            initial_parse_result,
            input_callback: None,
            mappings: HashMap::new(),
            message: String::new(),
            mode_label: String::new(),
            mode_string: NORMAL_MODE.to_string(),
            modes,
            relm: relm.clone(),
            settings: SETT::default(),
            settings_parser,
            shortcuts: HashMap::new(),
            shortcut_pressed: false,
            show_count: true,
            status_bar_command: String::new(),
            variables: HashMap::new(),
        }
    }

    /// Handle the key press event for the normal mode.
    #[allow(non_upper_case_globals)]
    fn normal_key_press(&mut self, key: &EventKey) -> Option<Msg<COMM, SETT>> {
        match key.get_keyval() {
            colon => Some(EnterCommandMode),
            Escape => {
                self.reset();
                self.clear_shortcut();
                self.handle_shortcut(key)
            },
            keyval => {
                let character = keyval as u8 as char;
                if COMM::is_identifier(character) {
                    self.model.completer = NO_COMPLETER_IDENT.to_string();
                    self.set_current_identifier(character);
                    self.set_mode(COMMAND_MODE);
                    self.reset();
                    self.clear_shortcut();
                    self.show_entry();
                    None
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

    /// Reset the input after closing a input dialog.
    fn reset_input(&mut self) {
        self.reset();
        self.return_to_normal_mode();
        self.model.choices.clear();
    }

    fn return_to_normal_mode(&mut self) {
        self.hide_entry_and_completion();
        self.set_mode(NORMAL_MODE);
        self.set_current_identifier(':');
    }

    /// Set the current (special) command identifier.
    fn set_current_identifier(&mut self, identifier: char) {
        self.model.current_command_mode = identifier;
    }

    fn set_input(&mut self, original_input: &str) {
        self.model.status_bar_command = original_input.to_string();
    }

    /// Set the current mode.
    fn set_mode(&mut self, mode: &str) {
        self.model.mode_string = mode.to_string();
        let current_mode =
            match mode {
                BLOCKING_INPUT_MODE => Mode::BlockingInput,
                COMMAND_MODE => Mode::Command,
                INPUT_MODE => Mode::Input,
                NORMAL_MODE => Mode::Normal,
                _ => Mode::Unknown,
            };
        if current_mode == Mode::Unknown {
            self.model.mode_label = mode.to_string();
        }
        else {
            self.model.mode_label = String::new();
        }
        self.model.current_mode.set(current_mode);
        self.model.relm.stream().emit(ModeChanged(mode.to_string()));

        for mode in self.model.modes.values() {
            if mode.name == self.model.mode_string {
                self.model.show_count = mode.show_count;
            }
        }
    }

    fn show_entry(&mut self) {
        self.model.entry_shown = true;
    }

    fn update(&mut self, event: Msg<COMM, SETT>) {
        match event {
            Alert(msg) => self.alert(&msg),
            // To be listened to by the user.
            AppClose => (),
            BlockingCustomDialog(responder, builder) =>
                self.blocking_custom_dialog(responder, builder),
            BlockingInput(responder, question, default_answer) =>
                self.blocking_input(responder, question, default_answer),
            BlockingQuestion(responder, question, choices) => self.blocking_question(responder, question, choices),
            BlockingYesNoQuestion(responder, question) => self.blocking_yes_no_question(responder, question),
            CloseWin => self.window.destroy(),
            Completers(completers) => self.completion_view.emit(AddCompleters(completers)),
            CompletionViewChange(completion) => self.set_input(&completion),
            // To be listened to by the user.
            CustomCommand(_) => (),
            CustomDialog(builder) => self.show_dialog(builder),
            DarkTheme(dark) => self.set_dark_theme(dark),
            DeleteCompletionItem => self.delete_current_completion_item(),
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
            Info(msg) => self.info(&msg),
            InitAfter => self.after_children_added(),
            Input(responder, input, default_answer) => self.input(responder, input, default_answer),
            Message(msg) => self.message(&msg),
            KeyPress(key) => self.key_press(&key),
            KeyRelease(key) => self.key_release(&key),
            Error(error) => self.error(error),
            HideColoredMessage(message) => self.hide_colored_message(&message),
            HideInfo(message) => self.hide_info(&message),
            // To be listened by the user.
            ModeChanged(_) | SettingChanged(_) => (),
            Question(responder, question, choices) => self.question(responder, question, choices),
            ResetInput => self.reset_input(),
            SetMode(mode) => self.set_mode(mode),
            SetSetting(setting) => self.set_setting(setting),
            StatusBarEntryActivate(input) => self.command_activate(input),
            StatusBarEntryChanged(input) => {
                // NOTE: Lock to prevent moving the cursor of the command entry.
                let _lock = self.status_bar.stream().lock();
                self.model.status_bar_command = input.unwrap_or_default();
                self.update_completions()
            },
            Title(title) => self.set_title(&title),
            Variables(variables) => self.set_variables(variables),
            Warning(message) => self.warning(&message),
            YesNoQuestion(responder, question) => self.yes_no_question(responder, question),
        }
    }

    /// Set the current status bar input completer.
    fn set_completer(&mut self, completer: &str) {
        self.model.completer = completer.to_string();
    }

    // TODO: try to replace emit() calls to view! connection.
    view! {
        #[name="window"]
        gtk::Window {
            #[container]
            gtk::Box {
                orientation: Vertical,
                #[container="status-bar-item"]
                #[name="status_bar"]
                StatusBar {
                    EntryShown: self.model.entry_shown,
                    EntryText: self.model.status_bar_command.clone(),
                    Identifier: self.model.current_command_mode.to_string(),
                    child: {
                        pack_type: PackType::End,
                    },
                    #[name="message"]
                    StatusBarItem {
                        Text: self.model.message.clone(),
                        child: {
                            pack_type: PackType::Start,
                        },
                    },
                    #[name="mode"]
                    StatusBarItem {
                        Text: self.model.mode_label.clone(),
                        child: {
                            pack_type: PackType::Start,
                        },
                    },
                    #[name="shortcut"]
                    StatusBarItem {
                        Text: shortcut_to_string(&self.model.current_shortcut, self.model.show_count),
                    },
                    EntryActivate(ref input) => StatusBarEntryActivate(input.clone()),
                    EntryChanged(ref text) => StatusBarEntryChanged(text.clone()),
                },
                gtk::Overlay {
                    child: {
                        pack_type: PackType::End,
                    },
                    #[name="completion_view"]
                    CompletionView(Self::default_completers()) {
                        Completer: self.model.completer.clone(),
                        Visible: self.model.completion_shown,
                        CompletionChange(ref completion) => CompletionViewChange(completion.clone()),
                    },
                },
            },
            key_press_event(_, key) with(current_mode) =>
                (KeyPress(key.clone()), Self::inhibit_key_press(&current_mode, key)),
            key_release_event(_, key) => (KeyRelease(key.clone()), Inhibit(false)),
            delete_event(_, _) => (AppClose, Inhibit(true)),
        },
    }
}

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    fn is_normal_command(&self) -> bool {
        self.model.current_command_mode == ':'
    }

    /// Set a setting value.
    fn set_setting(&mut self, setting: SETT::Variant) {
        self.model.settings.set_value(setting.clone());
        self.model.relm.stream().emit(SettingChanged(setting));
    }

    /// Set the window title.
    fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Set the variables that will be available in the settings.
    /// A variable can be used in mappings.
    /// The placeholder will be replaced by the value returned by the function.
    fn set_variables(&mut self, variables: Variables) {
        self.model.variables = variables.into_iter()
            .map(|(string, func)| (string.to_string(), func))
            .collect();
    }
}
