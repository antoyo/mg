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

use gdk::{EventKey, CONTROL_MASK, MOD1_MASK};
use gdk::enums::key::{Escape, Tab, ISO_Left_Tab};
use gtk::Inhibit;
use mg_settings::{self, EnumFromStr, EnumMetaData, SettingCompletion, SpecialCommand};
use mg_settings::key::Key;
use relm::Widget;

use app::{Mg, Msg, BLOCKING_INPUT_MODE, COMMAND_MODE, INPUT_MODE, NORMAL_MODE};
use app::ShortcutCommand::{Complete, Incomplete};
use key_converter::gdk_key_to_key;

/// Convert a shortcut of keys to a `String`.
pub fn shortcut_to_string(keys: &[Key]) -> String {
    let strings: Vec<_> = keys.iter().map(ToString::to_string).collect();
    strings.join("")
}

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Add the key to the current shortcut.
    pub fn add_to_shortcut(&mut self, key: Key) {
        self.model.current_shortcut.push(key);
        self.update_shortcut_label();
    }

    /// Clear the current shortcut buffer.
    pub fn clear_shortcut(&mut self) {
        self.model.current_shortcut.clear();
        self.update_shortcut_label();
    }

    /// Handle a possible input of a shortcut.
    pub fn handle_shortcut(&mut self, key: &EventKey) -> (Option<Msg<COMM, SETT>>, Inhibit) {
        let keyval = key.get_keyval();
        let alt_pressed = key.get_state() & MOD1_MASK == MOD1_MASK;
        let control_pressed = key.get_state() & CONTROL_MASK == CONTROL_MASK;
        let mut should_inhibit = self.model.current_mode == NORMAL_MODE ||
            (self.model.current_mode == COMMAND_MODE && (keyval == Tab || keyval == ISO_Left_Tab)) ||
            self.inhibit_command_shortcut(key) || key.get_keyval() == Escape;
        if !self.model.entry_shown || alt_pressed || control_pressed || keyval == Tab || keyval == ISO_Left_Tab {
            if let Some(key) = gdk_key_to_key(key) {
                self.add_to_shortcut(key);
                let action = {
                    let mut current_mode = self.model.current_mode.clone();
                    // The input modes have the same mappings as the command mode.
                    if current_mode == INPUT_MODE || current_mode == BLOCKING_INPUT_MODE {
                        current_mode = COMMAND_MODE.to_string();
                    }
                    self.model.mappings.get(&current_mode.as_ref())
                        .and_then(|mappings| mappings.get(&self.model.current_shortcut).cloned())
                };
                if let Some(action) = action {
                    // FIXME: this is copied a couple of lines below.
                    if !self.model.entry_shown {
                        self.reset();
                    }
                    self.clear_shortcut();
                    match self.action_to_command(&action) {
                        Complete(command) => return (self.handle_command(Some(command)), Inhibit(should_inhibit)),
                        Incomplete(command) => {
                            self.input_command(&command);
                            //self.status_bar.show_completion();
                            //self.update_completions();
                            should_inhibit = true;
                        },
                    }
                }
                else if self.no_possible_shortcut() {
                    if !self.model.entry_shown {
                        self.reset();
                    }
                    self.clear_shortcut();
                }
            }
        }
        (None, Inhibit(should_inhibit))
    }

    /// Check if the current key is in the config file.
    fn inhibit_command_shortcut(&self, key: &EventKey) -> bool {
        if let Some(mappings) = self.model.mappings.get(COMMAND_MODE) {
            if let Some(typed_key) = gdk_key_to_key(key) {
                for keys in mappings.keys() {
                    for key in keys {
                        if *key == typed_key {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if there are no possible shortcuts.
    fn no_possible_shortcut(&self) -> bool {
        if let Some(mappings) = self.model.mappings.get(&self.model.current_mode.as_ref()) {
            for key in mappings.keys() {
                if key.starts_with(&self.model.current_shortcut) {
                    return false;
                }
            }
        }
        true
    }

    // TODO: remove this when updating the model in methods outside the trait will update the view.
    /// Update the shortcut label.
    fn update_shortcut_label(&self) {
        self.shortcut.widget().root().set_text(&shortcut_to_string(&self.model.current_shortcut));
    }
}

/*impl<Spec, Comm, Sett> Application<Comm, Sett, Spec>
    where Spec: SpecialCommand + 'static,
          Comm: EnumFromStr + EnumMetaData + 'static,
          Sett: mg_settings::settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    /// Handle a shortcut in input mode.
    pub fn handle_input_shortcut(&mut self, key: &EventKey) -> bool {
        let keyval = key.get_keyval();
        let control_pressed = key.get_state() & CONTROL_MASK == CONTROL_MASK;
        if let Some(key) = gdk_key_to_key(keyval, control_pressed) {
            if self.shortcuts.contains_key(&key) {
                let answer = &self.shortcuts[&key].clone();
                self.set_dialog_answer(answer);
                self.shortcut_pressed = true;
                return true;
            }
        }
        false
    }
}*/
