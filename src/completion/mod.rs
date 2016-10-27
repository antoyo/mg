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

//! Trait and widget for input completion.

mod completers;
mod completion_view;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::{
    ListStore,
    TreeModelExt,
    TreeSelection,
};

pub use self::completers::{CommandCompleter, SettingCompleter};
pub use self::completion_view::CompletionView;

/// The identifier of the default completer.
pub const DEFAULT_COMPLETER_IDENT: &'static str = "__mg_default";

/// The identifier of the null completer.
pub const NO_COMPLETER_IDENT: &'static str = "__mg_no_completer";

/// The trait completer is an interface to be satisfied by input completers.
pub trait Completer {
    /// From the selected text entry, return the text that should be written in the text input.
    fn complete_result(&self, input: &str) -> String;

    /// From the user input, return the completion results.
    /// The results are on two columns, hence the 2-tuple.
    fn completions(&self, input: &str) -> Vec<CompletionResult>;

    /// Set the column to use as the result of a selected text entry.
    fn text_column(&self) -> i32 {
        0
    }
}

/// Completion to use with a text Entry.
pub struct Completion {
    completer_ident: RefCell<String>,
    completers: HashMap<String, Box<Completer>>,
    view: Rc<CompletionView>,
}

impl Completion {
    /// Create a new completion widget.
    pub fn new(completers: HashMap<String, Box<Completer>>, view: Rc<CompletionView>) -> Self {
        Completion {
            completer_ident: RefCell::new(String::new()),
            completers: completers,
            view: view,
        }
    }

    /// Adjust the model by using the specified completer.
    pub fn adjust_model(&self, completer_ident: &str) {
        if completer_ident != *self.completer_ident.borrow() {
            *self.completer_ident.borrow_mut() = completer_ident.to_string();
            if completer_ident == NO_COMPLETER_IDENT || !self.completers.contains_key(completer_ident) {
                *self.completer_ident.borrow_mut() = NO_COMPLETER_IDENT.to_string();
                self.view.set_model::<ListStore>(None);
            }
        }
    }

    /// Complete the result for the selection using the current completer.
    pub fn complete_result(&self, selection: &TreeSelection) -> Option<String> {
        let mut completion = None;
        let current_completer = self.current_completer_ident();
        if current_completer != NO_COMPLETER_IDENT {
            if let Some((model, iter)) = selection.get_selected() {
                if let Some(completer) = self.current_completer() {
                    let value: Option<String> = model.get_value(&iter, completer.text_column()).get();
                    if let Some(value) = value {
                        completion = Some(completer.complete_result(&value));
                    }
                }
            }
        }
        completion
    }

    /// Get the current completer.
    pub fn current_completer(&self) -> Option<&Completer> {
        let completer_ident = &*self.completer_ident.borrow();
        self.completers.get(completer_ident)
            .map(|completer| &**completer)
    }

    /// Get the current completer ident.
    pub fn current_completer_ident(&self) -> String {
        self.completer_ident.borrow().clone()
    }
}

/// A result to show in the completion view.
pub struct CompletionResult {
    /// The left column.
    pub col1: String,
    /// The right column.
    pub col2: String,
    /// The foreground color of the text.
    pub foreground: String,
}

impl CompletionResult {
    /// Create a new completion result.
    pub fn new(col1: &str, col2: &str) -> Self {
        CompletionResult {
            col1: col1.to_string(),
            col2: col2.to_string(),
            foreground: "white".to_string(),
        }
    }

    /// Add a cell property.
    pub fn new_with_foreground(col1: &str, col2: &str, foreground: &str) -> Self {
        CompletionResult {
            col1: col1.to_string(),
            col2: col2.to_string(),
            foreground: foreground.to_string(),
        }
    }
}
