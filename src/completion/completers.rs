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

use std::marker::PhantomData;

use mg_settings::EnumMetaData;

use completion::Completer;

/// A command completer.
pub struct CommandCompleter<T> {
    metadata: Vec<(String, String)>,
    _phantom: PhantomData<T>,
}

impl<T: EnumMetaData> CommandCompleter<T> {
    pub fn new() -> CommandCompleter<T> {
        let mut data: Vec<_> =
            T::get_metadata().iter()
                .filter(|&(_, metadata)| !metadata.completion_hidden)
                .map(|(setting_name, metadata)| (setting_name.clone(), metadata.help_text.clone()))
                .collect();
        data.push(("map".to_string(), "Create a new key binding".to_string()));
        data.push(("set".to_string(), "Change the value of a setting".to_string()));
        data.push(("unmap".to_string(), "Delete a key binding".to_string()));
        data.sort();
        CommandCompleter {
            metadata: data,
            _phantom: PhantomData,
        }
    }
}

impl<T> Completer for CommandCompleter<T> {
    fn complete_result(&self, input: &str) -> String {
        input.to_string()
    }

    fn completions(&self, input: &str) -> Vec<(String, String)> {
        self.metadata.iter()
            .filter(|&&(ref command, ref help)|
                    command.to_lowercase().contains(&input) ||
                    help.to_lowercase().contains(&input))
            .cloned()
            .collect()
    }

    fn text_column(&self) -> i32 {
        0
    }
}

/// A setting completer.
pub struct SettingCompleter<T> {
    metadata: Vec<(String, String)>,
    _phantom: PhantomData<T>,
}

impl<T: EnumMetaData> SettingCompleter<T> {
    pub fn new() -> Self {
        let mut data: Vec<_> =
            T::get_metadata().iter()
                .filter(|&(_, metadata)| !metadata.completion_hidden)
                .map(|(setting_name, metadata)| (setting_name.clone(), metadata.help_text.clone()))
                .collect();
        data.sort();
        SettingCompleter {
            metadata: data,
            _phantom: PhantomData,
        }
    }
}

impl<T> Completer for SettingCompleter<T> {
    fn complete_result(&self, input: &str) -> String {
        format!("set {} =", input)
    }

    fn completions(&self, input: &str) -> Vec<(String, String)> {
        self.metadata.iter()
            .filter(|&&(ref setting, ref help)|
                    setting.to_lowercase().contains(&input) ||
                    help.to_lowercase().contains(&input))
            .cloned()
            .collect()
    }

    fn text_column(&self) -> i32 {
        0
    }
}
