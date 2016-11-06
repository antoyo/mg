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

//! Non-modal message dialogs.

use std::rc::Rc;

use gtk::{self, Continue};
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion};
use mg_settings::settings;

use super::{Application, SpecialCommand};

const INFO_MESSAGE_DURATION: u32 = 5000;

impl<S, T, U> Application<S, T, U>
    where S: SpecialCommand + 'static,
          T: EnumFromStr + EnumMetaData + 'static,
          U: settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    /// Show an error to the user.
    pub fn error(&self, error: &str) {
        error!("{}", error);
        self.message.set_text(error);
        self.status_bar.hide_entry();
        self.status_bar.color_red();
    }

    /// Show an information message to the user for 5 seconds.
    pub fn info(app: &Rc<Self>, message: &str) {
        info!("{}", message);
        app.message.set_text(message);
        app.reset_colors();
        let app = app.clone();
        let message = Some(message.to_string());
        gtk::timeout_add(INFO_MESSAGE_DURATION, move || {
            if app.message.get_text() == message {
                app.message.set_text("");
            }
            Continue(false)
        });
    }

    /// Show a message to the user.
    pub fn message(&self, message: &str) {
        self.message.set_text(message);
        self.status_bar.color_blue();
    }

    /// Show a warning message to the user for 5 seconds.
    pub fn warning(app: &Rc<Self>, message: &str) {
        warn!("{}", message);
        app.message.set_text(message);
        let app = app.clone();
        let message = Some(message.to_string());
        app.status_bar.color_orange();
        gtk::timeout_add(INFO_MESSAGE_DURATION, move || {
            if app.message.get_text() == message {
                app.message.set_text("");
                app.reset_colors();
            }
            Continue(false)
        });
    }
}
