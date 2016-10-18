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

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;

use gtk::{ContainerExt, Entry, Label, WidgetExt};
use gtk::Orientation::Vertical;
use mg::Application;

use AppCommand::*;
use SpecialCommand::*;

#[derive(Commands)]
enum AppCommand {
    #[completion(hidden)]
    Follow,
    Insert,
    Normal,
    Open(String),
    Quit,
    WinOpen(String),
}

special_commands!(SpecialCommand {
    BackwardSearch('?', always),
    Search('/', always),
});

fn main() {
    gtk::init().unwrap();

    let app = Application::new_with_config(hash! {
        "i" => "insert",
    });
    app.use_dark_theme();
    let item = app.add_statusbar_item();
    item.set_text("Item");
    let item2 = app.add_statusbar_item();
    item2.set_text("Test");
    item.set_text("Rightmost");
    if let Err(error) = app.parse_config("main.conf") {
        app.error(&error.to_string());
    }
    app.add_variable("url", || "http://duckduckgo.com/lite".to_string());
    app.set_window_title("First Mg Program");

    let vbox = gtk::Box::new(Vertical, 0);
    let label = Label::new(Some("Mg App"));
    vbox.add(&label);
    let entry = Entry::new();
    vbox.add(&entry);
    app.set_view(&vbox);

    app.connect_mode_changed(|mode| println!("{}", mode));

    {
        let mg_app = app.clone();
        app.connect_command(move |command| {
            match command {
                Follow => (),
                Insert => mg_app.set_mode("insert"),
                Normal => mg_app.set_mode("normal"),
                Open(url) => label.set_text(&format!("Opening URL {}", url)),
                WinOpen(url) => label.set_text(&format!("Opening URL {} in new window", url)),
                Quit => gtk::main_quit(),
            }
        });
    }

    {
        app.connect_special_command(move |command| {
            match command {
                BackwardSearch(input) => println!("Searching backward for {}", input),
                Search(input) => println!("Searching for {}", input),
            }
        });
    }

    entry.grab_focus();

    gtk::main();
}
