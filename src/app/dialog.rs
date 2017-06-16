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

//! Non-modal input dialogs.

use std::collections::HashMap;

use gtk;
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion, SpecialCommand};
use mg_settings::key::Key;
use mg_settings::settings;
use relm::{ContainerComponent, EventStream, Relm, Widget};

use app::{Mg, BLOCKING_INPUT_MODE, INPUT_MODE};
use app::color::color_blue;
use app::Msg::{Input, Question};
use app::status_bar::Msg::{Identifier, ShowIdentifier};
use completion::completion_view::Msg::{SetOriginalInput, ShowCompletion};
use self::DialogResult::{Answer, Shortcut};

pub trait Responder {
    fn respond(&self, answer: Option<String>);
}

pub struct InputDialog<WIDGET: Widget> {
    callback: fn(Option<String>) -> WIDGET::Msg,
    stream: EventStream<WIDGET::Msg>,
}

impl<WIDGET: Widget> InputDialog<WIDGET> {
    fn new(relm: &Relm<WIDGET>, callback: fn(Option<String>) -> WIDGET::Msg) -> Self {
        InputDialog {
            callback,
            stream: relm.stream().clone(),
        }
    }
}

impl<WIDGET: Widget> Responder for InputDialog<WIDGET> {
    fn respond(&self, answer: Option<String>) {
        self.stream.emit((self.callback)(answer));
    }
}

/// Builder to create a new dialog.
pub struct DialogBuilder {
    /// Whether the dialog should block the calling function.
    blocking: bool,
    /// The available choices to the question.
    choices: Vec<char>,
    /// The text completer identifier for the input.
    completer: Option<String>,
    /// The default answer to the question.
    default_answer: String,
    /// The message/question to show to the user.
    message: String,
    /// The wrapper over the callback function to call for an asynchronous input dialog.
    responder: Option<Box<Responder>>,
    /// The available shortcuts.
    shortcuts: HashMap<Key, String>,
}

impl DialogBuilder {
    /// Create a new dialog builder.
    #[allow(new_without_default_derive, unknown_lints)]
    pub fn new() -> Self {
        DialogBuilder {
            blocking: false,
            choices: vec![],
            completer: None,
            default_answer: String::new(),
            message: String::new(),
            responder: None,
            shortcuts: HashMap::new(),
        }
    }

    /// Set whether the dialog will block or not.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = blocking;
        self
    }

    /// Set the choices available for the input.
    pub fn choices(mut self, choices: &[char]) -> Self {
        self.choices = choices.to_vec();
        self
    }

    /// Set the text completer for the input.
    pub fn completer(mut self, completer: &str) -> Self {
        self.completer = Some(completer.to_string());
        self
    }

    /// Set the default answer for the input.
    pub fn default_answer(mut self, answer: &str) -> Self {
        self.default_answer = answer.to_string();
        self
    }

    /// Set the message/question to show to the user.
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set a responder for an asynchronous dialog.
    pub fn responder(mut self, responder: Box<Responder>) -> Self {
        self.responder = Some(responder);
        self
    }

    /// Add a shortcut.
    pub fn shortcut(mut self, shortcut: Key, value: &str) -> Self {
        self.shortcuts.insert(shortcut, value.to_string());
        self
    }
}

/// Struct representing a dialog result.
/// A dialog result is either what the user typed in the input (Answer) or the string associated
/// with the shortcut.
pub enum DialogResult {
    /// A string typed by the user or None if the user closed the dialog (with Escape).
    Answer(Option<String>),
    /// A shortcut pressd by the user.
    Shortcut(String),
}

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    /// Ask a question to the user and block until the user provides it (or cancel).
    pub fn blocking_input(&mut self, message: &str, default_answer: &str) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog_without_shortcuts(builder)
    }

    /// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
    pub fn blocking_question(&mut self, message: &str, choices: &[char]) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(choices);
        self.show_dialog_without_shortcuts(builder)
    }

    /// Show a blocking yes/no question.
    pub fn blocking_yes_no_question(&mut self, message: &str) -> bool {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(&['y', 'n']);
        self.show_dialog_without_shortcuts(builder) == Some("y".to_string())
    }

    /// Ask a question to the user.
    pub fn input(&mut self, responder: Box<Responder>, message: &str, default_answer: &str) {
        let builder = DialogBuilder::new()
            .responder(responder)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog(builder);
    }

    /// Ask a multiple-choice question to the user.
    pub fn question(&mut self, responder: Box<Responder>, message: &str, choices: &[char]) {
        let builder = DialogBuilder::new()
            .responder(responder)
            .message(message)
            .choices(choices);
        self.show_dialog(builder);
    }

    /// Show a dialog created with a `DialogBuilder`.
    pub fn show_dialog(&mut self, mut dialog_builder: DialogBuilder) -> DialogResult {
        self.model.shortcut_pressed = false;
        {
            self.model.shortcuts.clear();
            for (key, value) in dialog_builder.shortcuts {
                self.model.shortcuts.insert(key, value);
            }
        }

        let choices = dialog_builder.choices.clone();
        if !choices.is_empty() {
            self.model.choices.clear();
            self.model.choices.append(&mut dialog_builder.choices);
            let choices: Vec<_> = choices.iter().map(|c| c.to_string()).collect();
            let choices = choices.join("/");
            self.status_bar.emit(Identifier(format!("{} ({}) ", dialog_builder.message, choices)));
            self.status_bar.emit(ShowIdentifier);
        }
        else {
            self.status_bar.emit(Identifier(format!("{} ", dialog_builder.message)));
            self.show_entry();
            self.set_input(&dialog_builder.default_answer);
        }

        if let Some(completer) = dialog_builder.completer {
            self.set_completer(&completer);
            self.completion_view.emit(SetOriginalInput(dialog_builder.default_answer));
            self.completion_view.emit(ShowCompletion);
        }

        self.model.answer = None;
        if dialog_builder.blocking {
            self.model.input_callback = Some(Box::new(|_| {
                gtk::main_quit();
            }));
        }
        if dialog_builder.blocking {
            self.set_mode(BLOCKING_INPUT_MODE);
        }
        else {
            if let Some(responder) = dialog_builder.responder {
                self.model.input_callback = Some(Box::new(move |answer| {
                    responder.respond(answer);
                }));
            }
            self.set_mode(INPUT_MODE);

        }
        color_blue(self.status_bar.widget());
        if dialog_builder.blocking {
            gtk::main();
            self.reset();
            self.return_to_normal_mode();
            self.model.choices.clear();
            if self.model.shortcut_pressed {
                if let Some(answer) = self.model.answer.clone() {
                    Shortcut(answer)
                }
                else {
                    Answer(None)
                }
            }
            else {
                Answer(self.model.answer.clone())
            }
        }
        else {
            Answer(None)
        }
    }

    /// Show a dialog created with a `DialogBuilder` which does not contain shortcut.
    pub fn show_dialog_without_shortcuts(&mut self, dialog_builder: DialogBuilder) -> Option<String> {
        match self.show_dialog(dialog_builder) {
            Answer(answer) => answer,
            Shortcut(_) => panic!("cannot return a shortcut in show_dialog_without_shortcuts()"),
        }
    }
}

/// Ask a question to the user.
pub fn input<COMM, SETT, WIDGET: Widget>(mg: &ContainerComponent<Mg<COMM, SETT>>, relm: &Relm<WIDGET>, msg: String,
    default_answer: String, callback: fn(Option<String>) -> WIDGET::Msg)
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
          WIDGET: 'static,
{
    let responder = Box::new(InputDialog::new(relm, callback));
    mg.emit(Input(responder, msg, default_answer));
}

/// Ask a multiple-choice question to the user.
pub fn question<COMM, SETT, WIDGET: Widget>(mg: &ContainerComponent<Mg<COMM, SETT>>, relm: &Relm<WIDGET>, msg: String,
    choices: &'static [char], callback: fn(Option<String>) -> WIDGET::Msg)
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
          WIDGET: 'static,
{
    let responder = Box::new(InputDialog::new(relm, callback));
    mg.emit(Question(responder, msg, choices));
}
