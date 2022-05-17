/*
 * Copyright (c) 2016-2021 Boucher, Antoni <bouanto@zoho.com>
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
extern crate gtk_test;
extern crate mg;
extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::time::Duration;

use gdk::keys::constants as keys;
use gtk::traits::{ButtonExt, EntryExt, LabelExt, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use gtk_test::{assert_text, click, enter_key, enter_keys};
use mg::{
    CustomCommand,
    Mg,
    char_slice,
    question,
};
use relm::{Relm, Widget, init_test};
use relm_derive::widget;

use self::AppCommand::*;
use self::Msg::*;

#[derive(Commands)]
pub enum AppCommand {
    #[help(text="Show the text in the label")]
    Show(String),
    Quit,
}

#[derive(Default, Settings)]
pub struct AppSettings {
    boolean: bool,
}

pub struct Model {
    relm: Relm<Win>,
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    CheckQuit(Option<String>),
    Command(AppCommand),
    ShowQuestion,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.widgets.entry.grab_focus();
    }

    fn model(relm: &Relm<Self>, _model: ()) -> Model {
        Model {
            relm: relm.clone(),
            text: "Label".to_string(),
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
                    Show(text) => self.model.text = format!("Showing text: {}", text),
                    Quit => gtk::main_quit(),
                }
            },
            ShowQuestion => question(&self.streams.mg, &self.model.relm, "Do you want to quit?".to_string(),
                char_slice!['y', 'n'], CheckQuit),
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>(&[], Ok("examples/main.conf".into()), None, vec![]) {
            gtk::Box {
                orientation: Vertical,
                #[name="label"]
                gtk::Label {
                    text: &self.model.text,
                },
                #[name="entry"]
                gtk::Entry {
                },
                #[name="button"]
                gtk::Button {
                    label: "Question",
                    clicked => ShowQuestion,
                    focus_on_click: false,
                },
            },
            CustomCommand(ref command) => Command(command.clone()),
        }
    }
}

#[test]
fn test_basic_command() {
    gtk::init().unwrap();

    let (_component, _, widgets) = init_test::<Win>(()).expect("init_test failed");
    let win = widgets.mg.clone();
    let entry = widgets.entry.clone();
    let button = widgets.button.clone();

    assert_text!(widgets.label, "Label");

    glib::timeout_add_local(Duration::from_secs(0), move || {
        click(&button);
        enter_keys(&win, "n");
        assert_text!(entry, "");
        enter_keys(&win, ":show test");
        enter_key(&win, keys::Return);
        enter_keys(&win, ":quit");
        enter_key(&win, keys::Return);
        glib::Continue(false)
    });

    gtk::main();

    assert_text!(widgets.label, "Showing text: test");
}
