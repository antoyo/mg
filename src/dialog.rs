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

use gtk;
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion};
use mg_settings::settings;

use super::{Application, SpecialCommand};

/// Builder to create a new dialog.
pub struct DialogBuilder {
    /// Whether the dialog should block the calling function.
    pub blocking: bool,
    /// The callback function to call for an asynchronous input dialog.
    pub callback: Option<Box<Fn(Option<&str>) + 'static>>,
    /// The available choices to the question.
    pub choices: Vec<char>,
    /// The default answer to the question.
    pub default_answer: String,
    /// The message/question to show to the user.
    pub message: String,
}

impl DialogBuilder {
    /// Create a new dialog builder.
    #[allow(new_without_default_derive)]
    pub fn new() -> Self {
        DialogBuilder {
            blocking: false,
            callback: None,
            choices: vec![],
            default_answer: String::new(),
            message: String::new(),
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
}

/// Trait to provide function for show dialogs.
pub trait DialogWindow {
    /// Ask a question to the user and block until the user provides it (or cancel).
    fn blocking_input(&self, message: &str, default_answer: &str) -> Option<String>;

    /// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
    fn blocking_question(&self, message: &str, choices: &[char]) -> Option<String>;

    /// Show a blocking yes/no question.
    fn blocking_yes_no_question(&self, message: &str) -> bool;

    /// Ask a question to the user.
    fn input<F: Fn(Option<&str>) + 'static>(&self, message: &str, default_answer: &str, callback: F);

    /// Ask a multiple-choice question to the user.
    fn question<F: Fn(Option<&str>) + 'static>(&self, message: &str, choices: &[char], callback: F);

    /// Show a dialog created with a `DialogBuilder`.
    fn show_dialog(&self, mut dialog_builder: DialogBuilder) -> Option<String>;
}

impl<S, T, U> DialogWindow for Application<S, T, U>
    where S: SpecialCommand + 'static,
          T: EnumFromStr + EnumMetaData + 'static,
          U: settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    fn blocking_input(&self, message: &str, default_answer: &str) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog(builder)
    }

    fn blocking_question(&self, message: &str, choices: &[char]) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(choices);
        self.show_dialog(builder)
    }

    fn blocking_yes_no_question(&self, message: &str) -> bool {
        let builder = DialogBuilder::new()
            .blocking(true)
            .message(message)
            .choices(&['y', 'n']);
        self.show_dialog(builder) == Some("y".to_string())
    }

    fn input<F: Fn(Option<&str>) + 'static>(&self, message: &str, default_answer: &str, callback: F) {
        let builder = DialogBuilder::new()
            .callback(callback)
            .default_answer(default_answer)
            .message(message);
        self.show_dialog(builder);
    }

    fn question<F: Fn(Option<&str>) + 'static>(&self, message: &str, choices: &[char], callback: F) {
        let builder = DialogBuilder::new()
            .callback(callback)
            .message(message)
            .choices(choices);
        self.show_dialog(builder);
    }

    fn show_dialog(&self, mut dialog_builder: DialogBuilder) -> Option<String> {
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
        *self.answer.borrow_mut() = None;
        if dialog_builder.blocking {
            *self.input_callback.borrow_mut() = Some(Box::new(|_| {
                gtk::main_quit();
            }));
        }
        if dialog_builder.blocking {
            self.set_mode("blocking-input");
        }
        else {
            if let Some(callback) = dialog_builder.callback {
                *self.input_callback.borrow_mut() = Some(Box::new(move |answer| {
                    callback(answer.as_ref().map(String::as_ref));
                }));
            }
            self.set_mode("input");

        }
        self.status_bar.color_blue();
        if dialog_builder.blocking {
            gtk::main();
            self.reset();
            self.return_to_normal_mode();
            (*self.choices.borrow_mut()).clear();
            self.answer.borrow().clone()
        }
        else {
            None
        }
    }
}
