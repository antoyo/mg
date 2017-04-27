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

extern crate gtk;
extern crate libxdo;
#[macro_use]
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;

mod utils;

use std::thread;

use gtk::Label;
use libxdo::XDo;
use mg::{Application, ApplicationBuilder};

use self::AppCommand::*;
use utils::XDoExt;

#[derive(Commands)]
enum AppCommand {
    #[help(text="Show the text in the label")]
    Show(String),
    Quit,
}

#[derive(Default, Settings)]
pub struct AppSettings {
    boolean: bool,
}

struct App {
    app: Box<Application<AppCommand, AppSettings>>,
    label: Label,
}

impl App {
    fn new() -> Box<Self> {
        let app = ApplicationBuilder::new()
            .settings(AppSettings::default())
            .build();

        let label = Label::new(Some("Label"));

        app.set_view(&label);

        let mut app = Box::new(App {
            app: app,
            label: label,
        });

        connect!(app.app, connect_command(command), app, handle_command(command));

        app
    }

    fn handle_command(&self, command: AppCommand) {
        match command {
            Show(text) => self.label.set_text(&format!("Showing text: {}", text)),
            Quit => gtk::main_quit(),
        }
    }
}

#[test]
fn test_basic_command() {
    gtk::init().unwrap();

    let app = App::new();

    assert_eq!(Some("Label".to_string()), app.label.get_text());

    thread::spawn(|| {
        let xdo = XDo::new(None).unwrap();
        xdo.enter_command("show test");
        xdo.enter_command("quit");
    });

    gtk::main();

    assert_eq!(Some("Showing text: test".to_string()), app.label.get_text());
}
