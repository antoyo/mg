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

use gtk::Continue;
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion};
use mg_settings::settings;

use super::{Application, SpecialCommand};

const INFO_MESSAGE_DURATION: u32 = 5000;

impl<Comm, Sett, Spec> Application<Comm, Sett, Spec>
    where Spec: SpecialCommand + 'static,
          Comm: EnumFromStr + EnumMetaData + 'static,
          Sett: settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    /// Show an alert message to the user.
    pub fn alert(&self, message: &str) {
        self.message.set_text(message);
        self.status_bar.color_blue();
    }

    /// Show an error to the user.
    pub fn error(&mut self, error: &str) {
        error!("{}", error);
        self.message.set_text(error);
        self.status_bar.hide_entry();
        self.status_bar.color_red();
    }

    /// Hide the information message.
    fn hide_colored_message(&self, message: &str) -> Continue {
        if let Some(current_message) = self.message.get_text() {
            if current_message == message {
                self.message.set_text("");
                self.reset_colors();
            }
        }
        Continue(false)
    }

    /// Hide the information message.
    fn hide_info(&self, message: &str) -> Continue {
        if let Some(current_message) = self.message.get_text() {
            if current_message == message {
                self.message.set_text("");
            }
        }
        Continue(false)
    }

    /// Show an information message to the user for 5 seconds.
    pub fn info(&mut self, message: &str) {
        info!("{}", message);
        self.message.set_text(message);
        self.reset_colors();
        let message = message.to_string();
        timeout_add!(INFO_MESSAGE_DURATION, self, hide_info(&message));
    }

    /// Show a message to the user.
    pub fn message(&self, message: &str) {
        self.message.set_text(message);
    }

    /// Show a warning message to the user for 5 seconds.
    pub fn warning(&mut self, message: &str) {
        warn!("{}", message);
        self.message.set_text(message);
        let message = message.to_string();
        self.status_bar.color_orange();
        timeout_add!(INFO_MESSAGE_DURATION, self, hide_colored_message(&message));
    }
}
