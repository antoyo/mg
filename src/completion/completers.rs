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
use std::marker::PhantomData;

use mg_settings::{EnumMetaData, SettingCompletion};

use completion::{Completer, CompletionResult};

/// A command completer.
pub struct CommandCompleter<T: Clone> {
    metadata: Vec<(String, String)>,
    _phantom: PhantomData<T>,
}

impl<T: Clone + EnumMetaData> CommandCompleter<T> {
    /// Create a new command completer.
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

impl<T: Clone> Completer for CommandCompleter<T> {
    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        self.metadata.iter()
            .filter(|&&(ref command, ref help)|
                    command.to_lowercase().contains(&input) ||
                    help.to_lowercase().contains(&input))
            .map(|&(ref col1, ref col2)| CompletionResult::new(&[col1, col2]))
            .collect()
    }
}

/// A nop completer.
pub struct NoCompleter {
}

impl NoCompleter {
    /// Create a new nop completer.
    #[allow(new_without_default_derive)]
    pub fn new() -> Self {
        NoCompleter {
        }
    }
}

impl Completer for NoCompleter {
    fn complete_result(&self, _value: &str) -> String {
        String::new()
    }

    fn completions(&mut self, _input: &str) -> Vec<CompletionResult> {
        vec![]
    }
}

/// A setting completer.
pub struct SettingCompleter<T> {
    selected_name: Option<String>,
    setting_names: Vec<(String, String)>,
    setting_values: HashMap<String, Vec<String>>,
    _phantom: PhantomData<T>,
}

impl<T: EnumMetaData + SettingCompletion> SettingCompleter<T> {
    /// Create a new setting completer.
    pub fn new() -> Self {
        let mut data: Vec<_> =
            T::get_metadata().iter()
                .filter(|&(_, metadata)| !metadata.completion_hidden)
                .map(|(setting_name, metadata)| (setting_name.clone(), metadata.help_text.clone()))
                .collect();
        data.sort();
        SettingCompleter {
            selected_name: None,
            setting_names: data,
            setting_values: T::get_value_completions(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Completer for SettingCompleter<T> {
    fn complete_result(&self, value: &str) -> String {
        if let Some(ref name) = self.selected_name {
            format!("set {} = {}", name, value)
        }
        else {
            format!("set {} =", value)
        }
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        if input.contains("= ") {
            let mut iter = input.split_whitespace();
            if let Some(name) = iter.next() {
                if let Some(values) = self.setting_values.get(name) {
                    iter.next(); // Skip the equal token.
                    let input_value = iter.next().unwrap_or_default();
                    self.selected_name = Some(name.to_string());
                    return values.iter()
                        .filter(|value| value.contains(input_value))
                        .map(|value| CompletionResult::new(&[value, &String::new()]))
                        .collect();
                }
            }
            vec![]
        }
        else {
            let input = input.trim();
            self.selected_name = None;
            self.setting_names.iter()
                .filter(|&&(ref setting, ref help)|
                        setting.to_lowercase().contains(input) ||
                        help.to_lowercase().contains(input))
                .map(|&(ref col1, ref col2)| CompletionResult::new(&[col1, col2]))
                .collect()
        }
    }
}
