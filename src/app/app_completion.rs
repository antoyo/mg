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

use std::collections::HashMap;

use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion, SpecialCommand};
use mg_settings::settings;

use app::Mg;
use completion::{
    self,
    CommandCompleter,
    SettingCompleter,
    DEFAULT_COMPLETER_IDENT,
};
use completion::completion_view::Msg::{
    DeleteCurrentCompletionItem,
    ShowCompletion,
    UpdateCompletions,
};

impl<COMM, SETT> Mg<COMM, SETT>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    /// Get the default completers.
    /// One to complete the commands, the other to complete the settings.
    pub fn default_completers() -> completion::Completers {
        let mut completers: HashMap<_, Box<completion::Completer>> = HashMap::new();
        completers.insert(DEFAULT_COMPLETER_IDENT, Box::new(CommandCompleter::<COMM>::new()));
        completers.insert("set", Box::new(SettingCompleter::<SETT>::new()));
        completers
    }

    /// Delete the current completion item.
    pub fn delete_current_completion_item(&self) {
        self.completion_view.emit(DeleteCurrentCompletionItem);
    }

    /// Show the completion view.
    pub fn show_completion(&self) {
        self.completion_view.stream().emit(ShowCompletion);
        self.update_completions();
    }

    /// Update the items of the completion view.
    pub fn update_completions(&self) {
        let input = self.model.status_bar_command.clone();
        self.completion_view.emit(UpdateCompletions(self.model.mode_string.clone(), input, self.is_normal_command()));
    }
}
