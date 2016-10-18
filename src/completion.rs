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

use gtk::{
    CellLayoutExt,
    CellRendererText,
    EntryCompletion,
    ListStore,
    TreeIter,
    TreeModelExt,
    Type,
};

use entry_completion::EntryCompletionExtManual;
use gobject::ObjectExtManual;

pub const DEFAULT_COMPLETER_IDENT: &'static str = "__mg_default";
pub const NO_COMPLETER_IDENT: &'static str = "__mg_no_completer";

pub trait Completer {
    fn data(&self) -> Vec<(String, String)>;
    fn text_column(&self) -> i32;
}

/// Entry completion.
pub struct Completion {
    completers: HashMap<String, Box<Completer>>,
    entry_completion: EntryCompletion,
}

impl Completion {
    pub fn new(default_completer: Box<Completer>) -> Self {
        let entry_completion = EntryCompletion::new();
        entry_completion.set_match_func(match_func);
        entry_completion.set_inline_selection(true);

        let cell1 = CellRendererText::new();
        entry_completion.pack_start(&cell1, true);
        entry_completion.add_attribute(&cell1, "text", 0);

        let cell2 = CellRendererText::new();
        entry_completion.pack_start(&cell2, true);
        entry_completion.add_attribute(&cell2, "text", 1);

        let mut completers = HashMap::new();
        completers.insert(DEFAULT_COMPLETER_IDENT.to_string(), default_completer);

        let completion = Completion {
            completers: completers,
            entry_completion: entry_completion,
        };

        completion.adjust_model(DEFAULT_COMPLETER_IDENT);

        completion
    }

    /// Adjust the model by using the specified completer.
    pub fn adjust_model(&self, completer_ident: &str) {
        if completer_ident == NO_COMPLETER_IDENT {
            self.entry_completion.set_model::<ListStore>(None);
        }
        else {
            let model = ListStore::new(&[Type::String, Type::String]);

            let completer = &self.completers[completer_ident];
            self.entry_completion.set_data("text-column", completer.text_column());
            for &(ref col1, ref col2) in &completer.data() {
                model.insert_with_values(None, &[0, 1], &[&col1, &col2]);
            }

            self.entry_completion.set_model(Some(&model));
        }
    }

    /// Get the `EntryCompletion`.
    pub fn get_entry_completion(&self) -> &EntryCompletion {
        &self.entry_completion
    }
}

/// Match case insensitively on all the columns.
fn match_func(entry_completion: &EntryCompletion, key: &str, iter: &TreeIter) -> bool {
    let model = entry_completion.get_model().unwrap();
    let key = key.to_lowercase();
    let column1: Option<String> = model.get_value(iter, 0).get();
    let column2: Option<String> = model.get_value(iter, 1).get();
    let column1 = column1.unwrap_or_default().to_lowercase();
    let column2 = column2.unwrap_or_default().to_lowercase();
    column1.contains(&key) || column2.contains(&key)
}
