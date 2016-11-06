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
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion};
use mg_settings::key::Key;
use mg_settings::settings;

use self::DialogResult::{Answer, Shortcut};
use super::{Application, SpecialCommand, BLOCKING_INPUT_MODE, INPUT_MODE};

/// Builder to create a new dialog.
pub struct DialogBuilder {
    /// Whether the dialog should block the calling function.
    blocking: bool,
    /// The callback function to call for an asynchronous input dialog.
    callback: Option<Box<Fn(Option<&str>) + 'static>>,
    /// The available choices to the question.
    choices: Vec<char>,
    /// The text completer identifier for the input.
    completer: Option<String>,
    /// The default answer to the question.
    default_answer: String,
    /// The message/question to show to the user.
    message: String,
    /// The available shortcuts.
    shortcuts: HashMap<Key, String>,
}

impl DialogBuilder {
    /// Create a new dialog builder.
    #[allow(new_without_default_derive)]
    pub fn new() -> Self {
        DialogBuilder {
            blocking: false,
            callback: None,
            choices: vec![],
            completer: None,
            default_answer: String::new(),
            message: String::new(),
            shortcuts: HashMap::new(),
        }
    }

    /// Set whether the dialog will block or not.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = blocking;
        self
    }

    /// Set a callback for an asynchronous dialog.
    pub fn callback<F: Fn(Option<&str>) + 'static>(mut self, callback: F) -> Self {
        self.callback = Some(Box::new(callback));
        self
    }

    /// Set the choices available for the input.
    pub fn choices(mut self, choices: &[char]) -> Self {
        self.choices = choices.iter().cloned().collect();
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

impl<S, T, U> Application<S, T, U>
    where S: SpecialCommand + 'static,
          T: EnumFromStr + EnumMetaData + 'static,
          U: settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    /// Ask a question to the user and block until the user provides it (or cancel).
    pub fn blocking_input(&self, message: &str, default_answer: &str) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog_without_shortcuts(builder)
    }

    /// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
    pub fn blocking_question(&self, message: &str, choices: &[char]) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(choices);
        self.show_dialog_without_shortcuts(builder)
    }

    /// Show a blocking yes/no question.
    pub fn blocking_yes_no_question(&self, message: &str) -> bool {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(&['y', 'n']);
        self.show_dialog_without_shortcuts(builder) == Some("y".to_string())
    }

    /// Ask a question to the user.
    pub fn input<F: Fn(Option<&str>) + 'static>(&self, message: &str, default_answer: &str, callback: F) {
        let builder = DialogBuilder::new()
            .callback(callback)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog(builder);
    }

    /// Ask a multiple-choice question to the user.
    pub fn question<F: Fn(Option<&str>) + 'static>(&self, message: &str, choices: &[char], callback: F) {
        let builder = DialogBuilder::new()
            .callback(callback)
            .message(message)
            .choices(choices);
        self.show_dialog(builder);
    }

    /// Show a dialog created with a `DialogBuilder`.
    pub fn show_dialog(&self, mut dialog_builder: DialogBuilder) -> DialogResult {
        self.shortcut_pressed.set(false);
        {
            let shortcuts = &mut *self.shortcuts.borrow_mut();
            shortcuts.clear();
            for (key, value) in dialog_builder.shortcuts {
                shortcuts.insert(key, value);
            }
        }

        let choices = dialog_builder.choices.clone();
        if !choices.is_empty() {
            {
                let app_choices = &mut *self.choices.borrow_mut();
                app_choices.clear();
                app_choices.append(&mut dialog_builder.choices);
            }
            let choices: Vec<_> = choices.iter().map(|c| c.to_string()).collect();
            let choices = choices.join("/");
            self.status_bar.set_identifier(&format!("{} ({}) ", dialog_builder.message, choices));
            self.status_bar.show_identifier();
        }
        else {
            self.status_bar.set_identifier(&format!("{} ", dialog_builder.message));
            self.status_bar.show_entry();
            self.status_bar.set_input(&dialog_builder.default_answer);
        }

        if let Some(completer) = dialog_builder.completer {
            self.status_bar.set_completer(&completer);
            self.status_bar.set_original_input(&dialog_builder.default_answer);
            self.show_completion_view();
        }

        *self.answer.borrow_mut() = None;
        if dialog_builder.blocking {
            *self.input_callback.borrow_mut() = Some(Box::new(|_| {
                gtk::main_quit();
            }));
        }
        if dialog_builder.blocking {
            self.set_mode(BLOCKING_INPUT_MODE);
        }
        else {
            if let Some(callback) = dialog_builder.callback {
                *self.input_callback.borrow_mut() = Some(Box::new(move |answer| {
                    callback(answer.as_ref().map(String::as_ref));
                }));
            }
            self.set_mode(INPUT_MODE);

        }
        self.status_bar.color_blue();
        if dialog_builder.blocking {
            gtk::main();
            self.reset();
            self.return_to_normal_mode();
            (*self.choices.borrow_mut()).clear();
            let answer = self.answer.borrow().clone();
            if self.shortcut_pressed.get() {
                if let Some(answer) = answer {
                    Shortcut(answer)
                }
                else {
                    Answer(None)
                }
            }
            else {
                Answer(self.answer.borrow().clone())
            }
        }
        else {
            Answer(None)
        }
    }

    /// Show a dialog created with a `DialogBuilder` which does not contain shortcut.
    pub fn show_dialog_without_shortcuts(&self, dialog_builder: DialogBuilder) -> Option<String> {
        match self.show_dialog(dialog_builder) {
            Answer(answer) => answer,
            Shortcut(_) => panic!("cannot return a shortcut in show_dialog_without_shortcuts()"),
        }
    }
}
