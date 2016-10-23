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

mod completers;
mod completion_view;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::{
    ListStore,
    TreeModelExt,
    TreeModelFilter,
    TreeSelection,
    Type,
};

pub use self::completers::{CommandCompleter, SettingCompleter};
pub use self::completion_view::CompletionView;

pub const DEFAULT_COMPLETER_IDENT: &'static str = "__mg_default";
pub const NO_COMPLETER_IDENT: &'static str = "__mg_no_completer";
pub const VISIBLE_COLUMN: u32 = 2;

pub trait Completer {
    fn complete_result(&self, input: &str) -> String;
    fn data(&self) -> Vec<(String, String)>;
    fn text_column(&self) -> i32;
}

/// Entry completion.
pub struct Completion {
    completer_ident: RefCell<String>,
    completers: RefCell<HashMap<String, Box<Completer>>>,
    view: Rc<CompletionView>,
}

impl Completion {
    pub fn new(default_completer: Box<Completer>, view: Rc<CompletionView>) -> Self {
        let mut completers = HashMap::new();
        completers.insert(DEFAULT_COMPLETER_IDENT.to_string(), default_completer);

        Completion {
            completer_ident: RefCell::new(String::new()),
            completers: RefCell::new(completers),
            view: view,
        }
    }

    /// Add a new completer for the `command_name` command.
    pub fn add_completer<C: Completer + 'static>(&self, command_name: &str, completer: C) {
        (*self.completers.borrow_mut()).insert(command_name.to_string(), Box::new(completer));
    }

    /// Adjust the model by using the specified completer.
    pub fn adjust_model(&self, completer_ident: &str) {
        if completer_ident != *self.completer_ident.borrow() {
            *self.completer_ident.borrow_mut() = completer_ident.to_string();
            if completer_ident == NO_COMPLETER_IDENT {
                self.view.set_model::<ListStore>(None);
            }
            else if let Some(completer) = (*self.completers.borrow()).get(completer_ident) {
                let list_store = ListStore::new(&[Type::String, Type::String, Type::Bool]);
                let model = TreeModelFilter::new(&list_store, None);
                model.set_visible_column(VISIBLE_COLUMN as i32);

                for &(ref col1, ref col2) in &completer.data() {
                    list_store.insert_with_values(None, &[0, 1, VISIBLE_COLUMN], &[&col1, &col2, &true]);
                }

                self.view.set_model(Some(&model));
            }
            else {
                *self.completer_ident.borrow_mut() = NO_COMPLETER_IDENT.to_string();
                self.view.set_model::<ListStore>(None);
            }
        }
    }

    /// Complete the result for the selection using the current completer.
    pub fn complete_result(&self, selection: &TreeSelection) -> Option<String> {
        let mut completion = None;
        let current_completer = self.current_completer();
        if current_completer != NO_COMPLETER_IDENT {
            if let Some((model, iter)) = selection.get_selected() {
                let completer_ident = &*self.completer_ident.borrow();
                if let Some(completer) = (*self.completers.borrow()).get(completer_ident) {
                    let value: Option<String> = model.get_value(&iter, completer.text_column()).get();
                    if let Some(value) = value {
                        completion = Some(completer.complete_result(&value));
                    }
                }
            }
        }
        completion
    }

    /// Get the current completer ident.
    pub fn current_completer(&self) -> String {
        self.completer_ident.borrow().clone()
    }
}
