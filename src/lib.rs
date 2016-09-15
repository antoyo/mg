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
 * TODO: Show the current shortcut in the status bar.
 * TODO: Try to return an Application directly instead of an Rc<Application>.
 * TODO: support shortcuts with number like "50G".
 */

//! Minimal UI library based on GTK+.

#![warn(missing_docs)]

extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
extern crate mg_settings;

mod key_converter;
#[macro_use]
mod widget;
mod status_bar;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

use gdk::{CONTROL_MASK, EventKey};
use gdk::enums::key::{Escape, colon};
use gtk::{ContainerExt, Grid, Inhibit, IsA, Widget, WidgetExt, Window, WindowExt, WindowType};
use mg_settings::{Config, EnumFromStr, Parser};
use mg_settings::Command::{Custom, Include, Map, Set, Unmap};
use mg_settings::error::{Error, Result};
use mg_settings::error::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use mg_settings::key::Key;

use key_converter::gdk_key_to_key;
use self::ShortcutCommand::{Complete, Incomplete};
use status_bar::StatusBar;

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
pub struct Application<T> {
    command_callback: RefCell<Option<Box<Fn(T)>>>,
    current_mode: String,
    current_shortcut: RefCell<Vec<Key>>,
    mappings: RefCell<HashMap<String, HashMap<Vec<Key>, String>>>,
    settings_parser: RefCell<Parser<T>>,
    status_bar: StatusBar,
    vbox: Grid,
    window: Window,
}

impl<T: EnumFromStr + 'static> Application<T> {
    /// Create a new application.
    #[allow(new_without_default)]
    pub fn new() -> Rc<Self> {
        Application::new_with_config(Config::default())
    }

    /// Create a new application with configuration.
    pub fn new_with_config(config: Config) -> Rc<Self> {
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

        let app = Rc::new(Application {
            command_callback: RefCell::new(None),
            current_mode: "n".to_string(),
            current_shortcut: RefCell::new(vec![]),
            mappings: RefCell::new(HashMap::new()),
            settings_parser: RefCell::new(Parser::new_with_config(config)),
            status_bar: status_bar,
            vbox: grid,
            window: window,
        });

        {
            let instance = app.clone();
            app.status_bar.connect_activate(move |command| instance.handle_command(command));
        }

        {
            let instance = app.clone();
            app.window.connect_key_press_event(move |_, key| instance.key_press(key));
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

    /// Add the key to the current shortcut.
    fn add_to_shortcut(&self, key: Key) {
        let mut shortcut = self.current_shortcut.borrow_mut();
        shortcut.push(key);
    }

    /// Add a callback to the command event.
    pub fn connect_command<F: Fn(T) + 'static>(&self, callback: F) {
        *self.command_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Handle the command activate event.
    fn handle_command(&self, command: Option<String>) {
        if let Some(command) = command {
            if let Some(ref callback) = *self.command_callback.borrow() {
                let result = self.settings_parser.borrow_mut().parse_line(&command);
                match result {
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
                            self.status_bar.show_error(&message);
                        }
                    },
                }
            }
            self.status_bar.hide_entry();
        }
    }

    /// Handle a possible input of a shortcut.
    fn handle_shortcut(&self, key: &EventKey) -> Inhibit {
        if !self.status_bar.entry_shown() {
            let control_pressed = key.get_state() & CONTROL_MASK == CONTROL_MASK;
            if let Some(key) = gdk_key_to_key(key.get_keyval(), control_pressed) {
                self.add_to_shortcut(key);
                let action = {
                    let shortcut = self.current_shortcut.borrow();
                    let mappings = self.mappings.borrow();
                    mappings.get(&self.current_mode)
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
        Inhibit(false)
    }

    /// Input the specified command.
    fn input_command(&self, command: &str) {
        self.status_bar.show_entry();
        self.status_bar.set_command(&format!("{} ", command));
    }

    /// Handle the key press event.
    #[allow(non_upper_case_globals)]
    fn key_press(&self, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            colon => {
                if !self.status_bar.entry_shown() {
                    self.reset();
                    self.status_bar.show_entry();
                    Inhibit(true)
                }
                else {
                    Inhibit(false)
                }
            },
            Escape => {
                self.reset();
                Inhibit(true)
            },
            _ => self.handle_shortcut(key),
        }
    }

    /// Check if there are no possible shortcuts.
    fn no_possible_shortcut(&self) -> bool {
        let current_shortcut = self.current_shortcut.borrow();
        let mappings = self.mappings.borrow();
        if let Some(mappings) = mappings.get(&self.current_mode) {
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
                    let mappings = mappings.entry(mode).or_insert_with(HashMap::new);
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
        self.status_bar.hide();
        let mut shortcut = self.current_shortcut.borrow_mut();
        shortcut.clear();
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

    /// Get the application window.
    pub fn window(&self) -> &Window {
        &self.window
    }
}
