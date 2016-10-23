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
    _phantom: PhantomData<T>,
}

impl<T> CommandCompleter<T> {
    pub fn new() -> CommandCompleter<T> {
        CommandCompleter {
            _phantom: PhantomData,
        }
    }
}

impl<T: EnumMetaData> Completer for CommandCompleter<T> {
    fn complete_result(&self, input: &str) -> String {
        input.to_string()
    }

    fn data(&self) -> Vec<(String, String)> {
        let mut data: Vec<_> =
            T::get_metadata().iter()
                .filter(|&(_, metadata)| !metadata.completion_hidden)
                .map(|(setting_name, metadata)| (setting_name.clone(), metadata.help_text.clone()))
                .collect();
        data.push(("map".to_string(), "Create a new key binding".to_string()));
        data.push(("set".to_string(), "Change the value of a setting".to_string()));
        data.push(("unmap".to_string(), "Delete a key binding".to_string()));
        data.sort();
        data
    }

    fn text_column(&self) -> i32 {
        0
    }
}

/// A setting completer.
pub struct SettingCompleter<T> {
    _phantom: PhantomData<T>,
}

impl<T> SettingCompleter<T> {
    pub fn new() -> Self {
        SettingCompleter {
            _phantom: PhantomData,
        }
    }
}

impl<T: EnumMetaData> Completer for SettingCompleter<T> {
    fn complete_result(&self, input: &str) -> String {
        format!("set {}", input)
    }

    fn data(&self) -> Vec<(String, String)> {
        let mut data: Vec<_> =
            T::get_metadata().iter()
                .filter(|&(_, metadata)| !metadata.completion_hidden)
                .map(|(setting_name, metadata)| (setting_name.clone(), metadata.help_text.clone()))
                .collect();
        data.sort();
        data
    }

    fn text_column(&self) -> i32 {
        0
    }
}
