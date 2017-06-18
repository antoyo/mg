/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

use std::char;

use gdk::EventKey;
use gdk::enums::key::Escape;
use gtk::Inhibit;
use mg_settings::{
    self,
    EnumFromStr,
    EnumMetaData,
    SettingCompletion,
    SpecialCommand,
};
use relm::Resolver;

use app::{Mg, BLOCKING_INPUT_MODE, COMMAND_MODE, INPUT_MODE, NORMAL_MODE};
use app::ActivationType::Current;
use app::Msg::{self, EnterNormalModeAndReset};

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Handle the key press event for the command mode.
    #[allow(non_upper_case_globals)]
    fn command_key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                (Some(EnterNormalModeAndReset), Inhibit(false))
            },
            _ => self.handle_shortcut(key),
        }
    }

    /// Handle the key release event for the command mode.
    fn command_key_release(&mut self, _key: &EventKey) -> Option<Msg<COMM, SETT>> {
        if self.model.current_command_mode != ':' && COMM::is_incremental(self.model.current_command_mode) {
            let command = self.model.status_bar_command.clone(); // TODO: remove this useless clone.
            let msg = self.handle_special_command(Current, &command);
            return msg;
        }
        None
    }

    /// Handle the key press event for the input mode.
    #[allow(non_upper_case_globals)]
    fn input_key_press(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                if let Some(callback) = self.model.input_callback.take() {
                    callback(None, self.model.shortcut_pressed);
                }
                (Some(EnterNormalModeAndReset), Inhibit(false))
            },
            keyval => {
                if self.handle_input_shortcut(key) {
                    return (None, Inhibit(true));
                }
                else if let Some(character) = char::from_u32(keyval) {
                    if self.model.choices.contains(&character) {
                        self.set_dialog_answer(&character.to_string());
                        return (None, Inhibit(true));
                    }
                }
                self.handle_shortcut(key)
            },
        }
    }

    /// Handle the key press event.
    pub fn key_press(&mut self, key: &EventKey, mut resolver: Resolver<Inhibit>) {
        let (msg, inhibit) =
            match self.model.current_mode.as_ref() {
                NORMAL_MODE => self.normal_key_press(key),
                COMMAND_MODE => self.command_key_press(key),
                BLOCKING_INPUT_MODE | INPUT_MODE => self.input_key_press(key),
                _ => self.handle_shortcut(key)
            };
        if let Some(msg) = msg {
            self.model.relm.stream().emit(msg);
        }
        resolver.resolve(inhibit)
    }

    /// Handle the key release event.
    pub fn key_release(&mut self, key: &EventKey) {
        let msg =
            match self.model.current_mode.as_ref() {
                COMMAND_MODE => self.command_key_release(key),
                _ => None,
            };
        if let Some(msg) = msg {
            self.model.relm.stream().emit(msg);
        }
    }
}
