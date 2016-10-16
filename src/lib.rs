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

/*
 * TODO: fix the size of the status bar.
 * TODO: supports shortcuts like <C-a> and <C-w> in the command entry.
 * TODO: smart home (with Ctrl-A) in the command text entry.
 * TODO: support non-"always" in special commands.
 * TODO: different event for activate event of special commands.
 * TODO: Disable focusing next element when hitting tab in the command entry.
 * TODO: Show the current shortcut in the status bar.
 * TODO: Try to return an Application directly instead of an Rc<Application>.
 * TODO: support shortcuts with number like "50G".
 * TODO: Associate a color with modes.
 */

//! Minimal UI library based on GTK+.

#![warn(missing_docs)]

extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate mg_settings;

mod key_converter;
mod gobject;
mod style_context;
#[macro_use]
mod widget;
mod status_bar;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

use gdk::{EventKey, RGBA, CONTROL_MASK};
use gdk::enums::key::{Escape, colon};
use gdk_sys::GdkRGBA;
use gtk::{ContainerExt, Grid, Inhibit, IsA, Settings, Widget, WidgetExt, Window, WindowExt, WindowType, STATE_FLAG_NORMAL};
use gtk::prelude::WidgetExtManual;
use mg_settings::{Config, EnumFromStr, Parser};
use mg_settings::Command::{self, Custom, Include, Map, Set, Unmap};
use mg_settings::error::{Error, Result};
use mg_settings::error::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use mg_settings::key::Key;

use key_converter::gdk_key_to_key;
use gobject::ObjectExtManual;
use self::ActivationType::{Current, Final};
use self::ShortcutCommand::{Complete, Incomplete};
use status_bar::StatusBar;
pub use status_bar::StatusBarItem;
use style_context::StyleContextExtManual;

#[macro_export]
macro_rules! hash {
    ($($key:expr => $value:expr),* $(,)*) => {{
        let mut hashmap = ::std::collections::HashMap::new();
        $(hashmap.insert($key.into(), $value.into());)*
        hashmap
    }};
}

#[macro_export]
macro_rules! special_commands {
    // TODO: use $(always) instead of always.
    ($enum_name:ident { $( $command:ident ( $identifier:expr , always ),)* } ) => {
        enum $enum_name {
            $( $command(String), )*
        }

        impl $crate::SpecialCommand for $enum_name {
            fn identifier_to_command(identifier: char, input: &str) -> ::std::result::Result<Self, String> {
                match identifier {
                    $( $identifier => Ok($command(input.to_string())), )*
                    _ => Err(format!("unknown identifier {}", identifier)),
                }
            }

            fn is_always(identifier: char) -> bool {
                match identifier {
                    $( $identifier )|* => true,
                    _ => false,
                }
            }

            fn is_identifier(character: char) -> bool {
                match character {
                    $( $identifier )|* => true,
                    _ => false,
                }
            }
        }
    };
}

/// Trait for converting an identifier like "/" to a special command.
pub trait SpecialCommand
    where Self: Sized
{
    /// Convert an identifier like "/" to a special command.
    fn identifier_to_command(identifier: char, input: &str) -> std::result::Result<Self, String>;

    /// Check if the identifier is declared `always`.
    /// The always option means that the command is activated every time a character is typed (like
    /// incremental search).
    fn is_always(identifier: char) -> bool;

    /// Check if a character is a special command identifier.
    fn is_identifier(character: char) -> bool;
}

type Modes = HashMap<String, String>;

const BLUE: &'static GdkRGBA = &GdkRGBA { red: 0.0, green: 0.0, blue: 1.0, alpha: 1.0 };
const RED: &'static GdkRGBA = &GdkRGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };
const TRANSPARENT: &'static GdkRGBA = &GdkRGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 };
const WHITE: &'static GdkRGBA = &GdkRGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 };

#[derive(PartialEq)]
enum ActivationType {
    Current,
    Final,
}

/// A command from a map command.
#[derive(Debug)]
enum ShortcutCommand {
    /// A complete command that is to be executed.
    Complete(String),
    /// An incomplete command where the user needs to complete it and press Enter.
    Incomplete(String),
}

/// Create a new MG application window.
/// This window contains a status bar where the user can type a command and a central widget.
pub struct Application<S, T> {
    answer: RefCell<Option<String>>,
    command_callback: RefCell<Option<Box<Fn(T)>>>,
    choices: RefCell<Vec<char>>,
    current_command_mode: Cell<char>,
    current_mode: RefCell<String>,
    current_shortcut: RefCell<Vec<Key>>,
    foreground_color: RefCell<RGBA>,
    input_callback: RefCell<Option<Box<Fn(Option<String>)>>>,
    mappings: RefCell<HashMap<String, HashMap<Vec<Key>, String>>>,
    modes: Modes,
    message: StatusBarItem,
    mode_changed_callback: RefCell<Option<Box<Fn(&str)>>>,
    mode_label: StatusBarItem,
    settings_parser: RefCell<Parser<T>>,
    special_command_callback: RefCell<Option<Box<Fn(S)>>>,
    status_bar: StatusBar,
    vbox: Grid,
    variables: RefCell<HashMap<String, Box<Fn() -> String>>>,
    window: Window,
}

impl<S: SpecialCommand + 'static, T: EnumFromStr + 'static> Application<S, T> {
    /// Create a new application.
    pub fn new() -> Rc<Self> {
        Application::new_with_config(HashMap::new())
    }

    /// Create a new application with configuration.
    pub fn new_with_config(mut modes: Modes) -> Rc<Self> {
        modes.insert("n".to_string(), "normal".to_string());
        modes.insert("c".to_string(), "command".to_string());
        let config = Config {
            mapping_modes: modes.keys().cloned().collect(),
        };
        let window = Window::new(WindowType::Toplevel);
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let grid = Grid::new();
        window.add(&grid);

        let status_bar = StatusBar::new();
        grid.attach(&status_bar, 0, 1, 1, 1);
        window.show_all();
        status_bar.hide();

        let foreground_color = Application::<S, T>::get_foreground_color(&window);

        let mode_label = StatusBarItem::new().left();
        let message = StatusBarItem::new().left();

        let app = Rc::new(Application {
            answer: RefCell::new(None),
            command_callback: RefCell::new(None),
            choices: RefCell::new(vec![]),
            current_command_mode: Cell::new(':'),
            current_mode: RefCell::new("normal".to_string()),
            current_shortcut: RefCell::new(vec![]),
            foreground_color: RefCell::new(foreground_color),
            input_callback: RefCell::new(None),
            mappings: RefCell::new(HashMap::new()),
            modes: modes,
            message: message,
            mode_changed_callback: RefCell::new(None),
            mode_label: mode_label,
            settings_parser: RefCell::new(Parser::new_with_config(config)),
            special_command_callback: RefCell::new(None),
            status_bar: status_bar,
            vbox: grid,
            variables: RefCell::new(HashMap::new()),
            window: window,
        });

        app.status_bar.add_item(&app.mode_label);
        app.status_bar.add_item(&app.message);

        {
            let instance = app.clone();
            app.status_bar.connect_activate(move |input| {
                if instance.get_mode() == "input" {
                    if let Some(ref callback) = *instance.input_callback.borrow() {
                        *instance.answer.borrow_mut() = input.clone();
                        callback(input);
                    }
                    instance.set_current_identifier(':');
                    *instance.input_callback.borrow_mut() = None;
                }
                else {
                    instance.set_mode("normal");
                    instance.handle_command(input);
                }
            });
        }

        {
            let instance = app.clone();
            app.window.connect_key_press_event(move |_, key| instance.key_press(key));
        }

        {
            let instance = app.clone();
            app.window.connect_key_release_event(move |_, key| instance.key_release(key));
        }

        app
    }

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

    /// Create a new status bar item.
    pub fn add_statusbar_item(&self) -> StatusBarItem {
        let item = StatusBarItem::new();
        self.status_bar.add_item(&item);
        item
    }

    /// Add the key to the current shortcut.
    fn add_to_shortcut(&self, key: Key) {
        let mut shortcut = self.current_shortcut.borrow_mut();
        shortcut.push(key);
    }

    /// Add a variable that can be used in mappings.
    /// The placeholder will be replaced by the value return by the function.
    pub fn add_variable<F: Fn() -> String + 'static>(&self, variable_name: &str, function: F) {
        let mut variables = self.variables.borrow_mut();
        variables.insert(variable_name.to_string(), Box::new(function));
    }

    /// Ask a question to the user and block until the user provides it (or cancel).
    pub fn blocking_input(&self, message: &str, default_answer: &str) -> Option<String> {
        self.status_bar.set_identifier(&format!("{} ", message));
        self.status_bar.show_entry();
        self.status_bar.set_input(default_answer);
        *self.answer.borrow_mut() = None;
        *self.input_callback.borrow_mut() = Some(Box::new(|_| {
            gtk::main_quit();
        }));
        self.set_mode("input");
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, BLUE);
        self.status_bar.override_color(STATE_FLAG_NORMAL, WHITE);
        gtk::main();
        self.set_mode("normal");
        self.reset();
        self.set_current_identifier(':');
        self.answer.borrow().clone()
    }

    /// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
    pub fn blocking_question(&self, message: &str, choices: &[char]) -> Option<char> {
        {
            let app_choices = &mut *self.choices.borrow_mut();
            app_choices.clear();
            app_choices.extend_from_slice(choices);
        }
        let choices: Vec<_> = choices.iter().map(|c| c.to_string()).collect();
        let choices = choices.join("/");
        self.status_bar.set_identifier(&format!("{} ({}) ", message, choices));
        self.status_bar.show_identifier();
        *self.answer.borrow_mut() = None;
        *self.input_callback.borrow_mut() = Some(Box::new(|_| {
            gtk::main_quit();
        }));
        self.set_mode("input");
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, BLUE);
        self.status_bar.override_color(STATE_FLAG_NORMAL, WHITE);
        gtk::main();
        let app_choices = &mut *self.choices.borrow_mut();
        app_choices.clear();
        self.set_mode("normal");
        self.reset();
        self.set_current_identifier(':');
        (*self.answer.borrow())
            .as_ref()
            .and_then(|answer| answer.chars().next())
    }

    /// Ask a yes-no question to the user and block until the user provides it (or cancel).
    pub fn blocking_yes_no_question(&self, message: &str) -> bool {
        self.blocking_question(message, &['y', 'n']) == Some('y')
    }

    /// Call the callback with the command or show an error if the command cannot be parsed.
    fn call_command(&self, callback: &Box<Fn(T)>, command: Result<Command<T>>) {
        match command {
            Ok(command) => {
                match command {
                    Custom(command) => callback(command),
                    _ => unimplemented!(),
                }
            },
            Err(error) => {
                if let Some(error) = error.downcast_ref::<Error>() {
                    let message =
                        match error.typ {
                            MissingArgument => "Argument required".to_string(),
                            NoCommand => return,
                            Parse => format!("Parse error: unexpected {}, expecting: {}", error.unexpected, error.expected),
                            UnknownCommand => format!("Not a command: {}", error.unexpected),
                        };
                    self.error(&message);
                }
            },
        }
    }

    /// Handle the key press event for the command mode.
    #[allow(non_upper_case_globals)]
    fn command_key_press(&self, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            Escape => {
                self.return_to_normal_mode();
                Inhibit(false)
            },
            _ => self.handle_shortcut(key),
        }
    }

    /// Handle the key release event for the command mode.
    fn command_key_release(&self, _key: &EventKey) -> Inhibit {
        let identifier = self.current_command_mode.get();
        if identifier != ':' && S::is_always(identifier) {
            if let Some(command) = self.status_bar.get_command() {
                self.handle_special_command(Current, &command);
            }
        }
        Inhibit(false)
    }

    /// Add a callback to the command event.
    pub fn connect_command<F: Fn(T) + 'static>(&self, callback: F) {
        *self.command_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Add a callback to the window key press event.
    pub fn connect_key_press_event<F: Fn(&Window, &EventKey) -> Inhibit + 'static>(&self, callback: F) {
        self.window.connect_key_press_event(callback);
    }

    /// Add a callback to change mode event.
    pub fn connect_mode_changed<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.mode_changed_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Add a callback to the special command event.
    pub fn connect_special_command<F: Fn(S) + 'static>(&self, callback: F) {
        *self.special_command_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Show an error to the user.
    pub fn error(&self, error: &str) {
        self.message.set_text(error);
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, RED);
        self.status_bar.override_color(STATE_FLAG_NORMAL, WHITE);
    }

    /// Get the color of the text.
    fn get_foreground_color(window: &Window) -> RGBA {
        let style_context = window.get_style_context().unwrap();
        style_context.get_color(STATE_FLAG_NORMAL)
    }

    /// Get the current mode.
    pub fn get_mode(&self) -> String {
        self.current_mode.borrow().clone()
    }

    /// Handle the command activate event.
    fn handle_command(&self, command: Option<String>) {
        if let Some(command) = command {
            let identifier = self.current_command_mode.get();
            if identifier == ':' {
                if let Some(ref callback) = *self.command_callback.borrow() {
                    let result = self.settings_parser.borrow_mut().parse_line(&command);
                    self.call_command(callback, result);
                }
            }
            else {
                self.handle_special_command(Final, &command);
            }
            self.status_bar.hide_entry();
        }
    }

    /// Handle a possible input of a shortcut.
    fn handle_shortcut(&self, key: &EventKey) -> Inhibit {
        let mode = self.get_mode();
        if !self.status_bar.entry_shown() {
            let control_pressed = key.get_state() & CONTROL_MASK == CONTROL_MASK;
            if let Some(key) = gdk_key_to_key(key.get_keyval(), control_pressed) {
                self.add_to_shortcut(key);
                let action = {
                    let shortcut = self.current_shortcut.borrow();
                    let mappings = self.mappings.borrow();
                    mappings.get(&*self.current_mode.borrow())
                        .and_then(|mappings| mappings.get(&*shortcut).cloned())
                };
                if let Some(action) = action {
                    self.reset();
                    match self.action_to_command(&action) {
                        Complete(command) => self.handle_command(Some(command)),
                        Incomplete(command) => {
                            self.input_command(&command);
                            return Inhibit(true);
                        },
                    }
                }
                else if self.no_possible_shortcut() {
                    self.reset();
                }
            }
        }
        if mode == "normal" {
            Inhibit(true)
        }
        else {
            Inhibit(false)
        }
    }

    /// Handle a special command activate or key press event.
    fn handle_special_command(&self, activation_type: ActivationType, command: &str) {
        if let Some(ref callback) = *self.special_command_callback.borrow() {
            let identifier = self.current_command_mode.get();
            if let Ok(special_command) = S::identifier_to_command(identifier, command) {
                callback(special_command);
                if activation_type == Final {
                    self.set_current_identifier(':');
                }
            }
        }
    }

    /// Ask a question to the user.
    pub fn input<F: Fn(Option<String>) + 'static>(&self, message: &str, default_answer: &str, callback: F) {
        self.set_mode("input");
        self.status_bar.set_identifier(&format!("{} ", message));
        self.status_bar.show_entry();
        self.status_bar.set_input(default_answer);
        *self.input_callback.borrow_mut() = Some(Box::new(callback));
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, BLUE);
        self.status_bar.override_color(STATE_FLAG_NORMAL, WHITE);
    }

    /// Input the specified command.
    fn input_command(&self, command: &str) {
        self.set_mode("command");
        self.status_bar.show_entry();
        let variables = self.variables.borrow();
        let mut command = command.to_string();
        for (variable, function) in variables.iter() {
            command = command.replace(&format!("<{}>", variable), &function());
        }
        let text: Cow<str> =
            if command.contains(' ') {
                command.into()
            }
            else {
                format!("{} ", command).into()
            };
        self.status_bar.set_input(&text);
    }

    /// Handle the key press event for the input mode.
    #[allow(non_upper_case_globals)]
    fn input_key_press(&self, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            Escape => {
                self.return_to_normal_mode();
                if let Some(ref callback) = *self.input_callback.borrow() {
                    callback(None);
                }
                *self.input_callback.borrow_mut() = None;
                Inhibit(false)
            },
            keyval => {
                if let Some(character) = char::from_u32(keyval) {
                    if (*self.choices.borrow()).contains(&character) {
                        *self.answer.borrow_mut() = Some(character.to_string());
                        gtk::main_quit();
                        return Inhibit(true);
                    }
                }
                Inhibit(false)
            },
        }
    }

    /// Handle the key press event.
    fn key_press(&self, key: &EventKey) -> Inhibit {
        match self.get_mode().as_ref() {
            "normal" => self.normal_key_press(key),
            "command" => self.command_key_press(key),
            "input" => self.input_key_press(key),
            _ => self.handle_shortcut(key)
        }
    }

    /// Handle the key release event.
    fn key_release(&self, key: &EventKey) -> Inhibit {
        match self.get_mode().as_ref() {
            "command" => self.command_key_release(key),
            _ => Inhibit(false),
        }
    }

    /// Show a message to the user.
    pub fn message(&self, message: &str) {
        self.message.set_text(message);
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, BLUE);
        self.status_bar.override_color(STATE_FLAG_NORMAL, WHITE);
    }

    /// Handle the key press event for the normal mode.
    #[allow(non_upper_case_globals)]
    fn normal_key_press(&self, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            colon => {
                self.set_current_identifier(':');
                self.set_mode("command");
                self.reset();
                self.status_bar.show_entry();
                Inhibit(true)
            },
            Escape => {
                self.reset();
                self.handle_shortcut(key)
            },
            keyval => {
                let character = keyval as u8 as char;
                if S::is_identifier(character) {
                    self.set_current_identifier(character);
                    self.set_mode("command");
                    self.reset();
                    self.status_bar.show_entry();
                    Inhibit(true)
                }
                else {
                    self.handle_shortcut(key)
                }
            },
        }
    }

    /// Check if there are no possible shortcuts.
    fn no_possible_shortcut(&self) -> bool {
        let current_shortcut = self.current_shortcut.borrow();
        let mappings = self.mappings.borrow();
        if let Some(mappings) = mappings.get(&*self.current_mode.borrow()) {
            for key in mappings.keys() {
                if key.starts_with(&*current_shortcut) {
                    return false;
                }
            }
        }
        true
    }

    /// Parse a configuration file.
    pub fn parse_config<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        let file = try!(File::open(filename));
        let buf_reader = BufReader::new(file);
        let commands = try!(self.settings_parser.borrow_mut().parse(buf_reader));
        for command in commands {
            match command {
                Custom(_) => (), // TODO: call the callback?
                Include(_) => (), // TODO: parse the included file.
                Map { action, keys, mode } => {
                    let mut mappings = self.mappings.borrow_mut();
                    let mappings = mappings.entry(self.modes[&mode].clone()).or_insert_with(HashMap::new);
                    mappings.insert(keys, action);
                },
                Set(_, _) => (), // TODO: set settings.
                Unmap { .. } => (), // TODO
            }
        }
        Ok(())
    }

    /// Handle the escape event.
    fn reset(&self) {
        self.status_bar.override_background_color(STATE_FLAG_NORMAL, TRANSPARENT);
        self.status_bar.override_color(STATE_FLAG_NORMAL, &self.foreground_color.borrow());
        self.status_bar.hide();
        self.message.set_text("");
        self.show_mode();
        let mut shortcut = self.current_shortcut.borrow_mut();
        shortcut.clear();
    }

    /// Go back to the normal mode from command or input mode.
    fn return_to_normal_mode(&self) {
        self.set_mode("normal");
        self.set_current_identifier(':');
        self.reset();
    }

    /// Set the current (special) command identifier.
    fn set_current_identifier(&self, identifier: char) {
        self.current_command_mode.set(identifier);
        self.status_bar.set_identifier(&identifier.to_string());
    }

    /// Set the current mode.
    pub fn set_mode(&self, mode: &str) {
        *self.current_mode.borrow_mut() = mode.to_string();
        self.show_mode();
        if let Some(ref callback) = *self.mode_changed_callback.borrow() {
            callback(mode);
        }
    }

    /// Set the main widget.
    pub fn set_view<W: IsA<Widget> + WidgetExt>(&self, view: &W) {
        view.set_hexpand(true);
        view.set_vexpand(true);
        view.show_all();
        self.vbox.attach(view, 0, 0, 1, 1);
    }

    /// Set the window title.
    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Show the current mode if it is not the normal mode.
    fn show_mode(&self) {
        let mode = self.get_mode();
        if mode != "normal" && mode != "command" && mode != "input" {
            self.mode_label.set_text(&mode);
        }
        else {
            self.mode_label.set_text("");
        }
    }

    /// Use the dark variant of the theme if available.
    pub fn use_dark_theme(&self) {
        let settings = Settings::get_default().unwrap();
        settings.set_data("gtk-application-prefer-dark-theme", 1);
        *self.foreground_color.borrow_mut() = Application::<S, T>::get_foreground_color(&self.window);
    }

    /// Get the application window.
    pub fn window(&self) -> &Window {
        &self.window
    }
}
