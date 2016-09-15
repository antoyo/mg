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
 * TODO: Try to return an Application directly instead of an Rc<Application>.
 * FIXME: do not clear the entry when typing : in the entry.
 * TODO: support shortcuts with number like "50G".
 */

//! Minimal UI library based on GTK+.

#![warn(missing_docs)]

extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
extern crate mg_settings;

#[macro_use]
mod widget;
mod status_bar;

use std::cell::RefCell;
use std::rc::Rc;

use gdk::EventKey;
use gdk::enums::key::{Escape, colon};
use gtk::{ContainerExt, Grid, Inhibit, IsA, Widget, WidgetExt, Window, WindowExt, WindowType};
use mg_settings::{EnumFromStr, Parser};
use mg_settings::Command::Custom;
use mg_settings::error::Error;
use mg_settings::error::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};

use status_bar::StatusBar;

/// Create a new MG application window.
/// This window contains a status bar where the user can type a command and a central widget.
pub struct Application<T> {
    command_callback: RefCell<Option<Box<Fn(T)>>>,
    settings_parser: RefCell<Parser<T>>,
    status_bar: StatusBar,
    vbox: Grid,
    window: Window,
}

impl<T: EnumFromStr + 'static> Application<T> {
    /// Create a new application.
    #[allow(new_without_default)]
    pub fn new() -> Rc<Self> {
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
            settings_parser: RefCell::new(Parser::new()),
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

    /// Add a callback to the command event.
    pub fn connect_command<F: Fn(T) + 'static>(&self, callback: F) {
        *self.command_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Handle the command activate event.
    fn handle_command(&self, command: Option<String>) {
        if let Some(ref callback) = *self.command_callback.borrow() {
            // NOTE: assume there is always a text in an entry.
            let result = self.settings_parser.borrow_mut().parse_line(&command.unwrap());
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

    /// Handle the key press event.
    #[allow(non_upper_case_globals)]
    fn key_press(&self, key: &EventKey) -> Inhibit {
        match key.get_keyval() {
            colon => {
                self.status_bar.show_entry();
                Inhibit(true)
            },
            Escape => {
                self.status_bar.hide();
                Inhibit(true)
            },
            _ => Inhibit(false),
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

    /// Get the application window.
    pub fn window(&self) -> &Window {
        &self.window
    }
}
