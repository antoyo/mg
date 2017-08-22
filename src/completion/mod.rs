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
#[allow(missing_docs)]
pub mod completion_view;

use std::collections::HashMap;

use glib::ToValue;
use gtk::{
    ListStore,
    ListStoreExt,
    ListStoreExtManual,
    TreeModelExt,
    TreeSelection,
    TreeSelectionExt,
    Type,
};

use self::Column::Expand;
pub use self::completers::{CommandCompleter, NoCompleter, SettingCompleter};
pub use self::completion_view::CompletionView;

/// The identifier of the default completer.
pub const DEFAULT_COMPLETER_IDENT: &str = "__mg_default";

/// The identifier of the null completer.
pub const NO_COMPLETER_IDENT: &str = "__mg_no_completer";

#[doc(hidden)]
pub type Completers = HashMap<&'static str, Box<Completer>>;

/// The type of a column.
#[derive(Clone, Copy, PartialEq)]
pub enum Column {
    /// Specifies that the column does not expand, but won't be ellipsized.
    AllVisible,
    /// Specifies that the column will expand, but can be truncated (ellipsized).
    Expand,
}

/// The trait completer is an interface to be satisfied by input completers.
pub trait Completer {
    /// The number of columns of the completer.
    fn columns(&self) -> Vec<Column> {
        vec![Expand, Expand]
    }

    /// The number of column.
    fn column_count(&self) -> usize {
        self.columns().len()
    }

    /// From the selected text entry, return the text that should be written in the text input.
    fn complete_result(&self, value: &str) -> String {
        value.to_string()
    }

    /// From the user input, return the completion results.
    /// The results are on two columns, hence the 2-tuple.
    fn completions(&mut self, input: &str) -> Vec<CompletionResult>;

    /// Set the column to use as the result of a selected text entry.
    fn text_column(&self) -> i32 {
        0
    }
}

/// Completion to use with a text Entry.
pub struct Completion {
    completer_ident: String,
    completers: Completers,
}

impl Completion {
    /// Create a new completion widget.
    #[allow(new_without_default_derive, unknown_lints)]
    pub fn new() -> Self {
        Completion {
            completer_ident: String::new(),
            completers: HashMap::new(),
        }
    }

    /// Add a new completer.
    pub fn add_completer(&mut self, ident: &'static str, completer: Box<Completer>) {
        let _ = self.completers.insert(ident, completer);
    }

    /// Adjust the model by using the specified completer.
    pub fn adjust_model(&mut self, completer_ident: &str) -> bool {
        if completer_ident != self.completer_ident {
            self.completer_ident = completer_ident.to_string();
            if completer_ident == NO_COMPLETER_IDENT || !self.completers.contains_key(completer_ident) {
                self.completer_ident = NO_COMPLETER_IDENT.to_string();
                return true;
            }
        }
        false
    }

    /// Complete the result for the selection using the current completer.
    pub fn complete_result(&self, selection: &TreeSelection) -> Option<String> {
        let mut completion = None;
        if self.current_completer_ident() != NO_COMPLETER_IDENT {
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
        self.completers.get(self.completer_ident.as_str())
            .map(AsRef::as_ref)
    }

    /// Get the current completer.
    #[allow(borrowed_box)]
    pub fn current_completer_mut(&mut self) -> Option<&mut Box<Completer>> {
        self.completers.get_mut(self.completer_ident.as_str())
    }

    /// Get the current completer ident.
    pub fn current_completer_ident(&self) -> &str {
        &self.completer_ident
    }

    /// Filter the rows from the input.
    pub fn filter(&mut self, input: &str) -> Option<ListStore> {
        self.current_completer_mut()
            .map(|completer| {
                // Multiply by 2 because each column has a foreground column.
                let columns = vec![Type::String; completer.column_count() * 2];
                let model = ListStore::new(&columns);

                let key =
                    if let Some(index) = input.find(' ') {
                        input[index + 1 ..].trim_left()
                    }
                    else {
                        input
                    };

                for &CompletionResult { ref columns } in &completer.completions(key) {
                    let row = model.insert(-1);
                    let start_column = columns.len();
                    for (index, cell) in columns.iter().enumerate() {
                        model.set_value(&row, index as u32, &cell.value.to_value());
                        if let Some(ref foreground) = cell.foreground {
                            model.set_value(&row, (index + start_column) as u32, &foreground.to_value());
                        }
                    }
                }
                model
            })
    }

    /// Set all the completers.
    pub fn set_completers(&mut self, mut completers: Completers) {
        let _ = completers.insert(NO_COMPLETER_IDENT, Box::new(NoCompleter::new()));
        self.completers = completers;
    }
}

/// A completion cell is the value with attributes of one data in a row.
#[derive(Clone)]
pub struct CompletionCell {
    /// The foreground color of the cell or None if using the default color.
    pub foreground: Option<String>,
    /// The text value to show on the cell.
    pub value: String,
}

impl CompletionCell {
    /// Create a new cell.
    pub fn new(value: &str) -> Self {
        CompletionCell {
            foreground: None,
            value: value.to_string(),
        }
    }

    /// Set the foreground color of the cell.
    pub fn foreground(mut self, foreground: &str) -> Self {
        self.foreground = Some(foreground.to_string());
        self
    }
}

/// Trait to specify that a type can be converted to a `CompletionCell`.
pub trait ToCell {
    /// Convert a value to a `CompletionCell`.
    fn to_cell(&self) -> CompletionCell;
}

impl ToCell for CompletionCell {
    fn to_cell(&self) -> CompletionCell {
        self.clone()
    }
}

impl ToCell for str {
    fn to_cell(&self) -> CompletionCell {
        CompletionCell::new(self)
    }
}

impl ToCell for String {
    fn to_cell(&self) -> CompletionCell {
        CompletionCell::new(self)
    }
}

/// A result to show in the completion view.
pub struct CompletionResult {
    /// The columns data.
    pub columns: Vec<CompletionCell>,
}

impl CompletionResult {
    /// Create a new completion result.
    pub fn new(cols: &[&str]) -> Self {
        let cols: Vec<_> = cols.iter().map(|col| CompletionCell::new(col)).collect();
        CompletionResult {
            columns: cols,
        }
    }

    /// Create a new completion result with foregrounds.
    pub fn from_cells(cols: &[&ToCell]) -> Self {
        CompletionResult {
            columns: cols.iter().map(|value| value.to_cell()).collect(),
        }
    }
}
