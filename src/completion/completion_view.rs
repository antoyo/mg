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

use std::cmp::max;
use std::collections::HashMap;

use glib::{Cast, Object};
use gtk;
use gtk::{
    Align,
    CellLayoutExt,
    CellRendererText,
    CellRendererTextExt,
    IsA,
    ListStore,
    ScrolledWindowExt,
    TreeIter,
    TreeModel,
    TreeModelExt,
    TreeSelection,
    TreeViewColumn,
    WidgetExt,
};
use gtk::PolicyType::{Automatic, Never};
use pango::EllipsizeMode;
use relm::{Relm, Widget};
use relm_attributes::widget;

use app::COMMAND_MODE;
use completion::Column::{self, Expand};
use super::{Completer, Completion, DEFAULT_COMPLETER_IDENT, NO_COMPLETER_IDENT};

const COMPLETION_VIEW_MAX_HEIGHT: i32 = 300;

#[allow(missing_docs)]
pub struct Model {
    completion: Completion,
    original_input: String,
    visible: bool,
}

/// A widget to show completions for the command entry.
#[widget]
impl Widget for CompletionView {
    fn init_view(&self) {
        self.add_columns(2);
    }

    fn model(_relm: &Relm<Self>, _: ()) -> Model {
        Model {
            completion: Completion::new(),
            original_input: String::new(),
            visible: false,
        }
    }

    /// Set the visibility of the view.
    pub fn set_visible(&mut self, visible: bool) {
        self.model.visible = visible;
    }

    fn update(&mut self, _event: ()) {
    }

    view! {
        #[name="scrolled_window"]
        gtk::ScrolledWindow {
            max_content_height: COMPLETION_VIEW_MAX_HEIGHT,
            propagate_natural_height: true,
            valign: Align::End,
            visible: self.model.visible,
            #[name="tree_view"]
            gtk::TreeView {
                can_focus: false,
                enable_search: false,
                headers_visible: false,
            }
        }
    }
}

impl CompletionView {
    /// Add a column to the tree view.
    fn add_column(&self, index: i32, foreground_index: i32, column: Column) {
        let view_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        if column == Expand {
            cell.set_property_ellipsize(EllipsizeMode::End);
            view_column.set_expand(true);
        }
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", index);
        view_column.add_attribute(&cell, "foreground", foreground_index);
        self.tree_view.append_column(&view_column);
    }

    /// Add the specified number of columns.
    fn add_columns(&self, column_count: i32) {
        self.remove_columns();
        for i in 0 .. column_count {
            self.add_column(i, column_count + i, Expand);
        }
    }

    /// Add the specified number of columns.
    fn add_columns_from_completer(&self, completer: &Completer) {
        self.remove_columns();
        let columns = completer.columns();
        let column_count = columns.len() as i32;
        for (i, column) in columns.iter().enumerate() {
            let i = i as i32;
            self.add_column(i, column_count + i, *column);
        }
    }

    /// Adjust the columns from the completer.
    pub fn adjust_columns(&self, completer: &Completer) {
        self.add_columns_from_completer(completer);
    }

    /// Adjust the policy of the scrolled window to avoid having extra space around the tree view.
    pub fn adjust_policy<M: IsA<Object> + IsA<TreeModel>>(&self, model: &M) {
        self.tree_view.set_model(Some(model));
        let policy =
            if model.iter_n_children(None) < 2 {
                Never
            }
            else {
                Automatic
            };
        self.scrolled_window.set_policy(Never, policy);
    }

    /// Complete the result for the selection using the current completer.
    pub fn complete_result(&self, selection: &TreeSelection) -> Option<String> {
        self.model.completion.complete_result(selection)
    }

    /// Delete the current completion item.
    pub fn delete_current_completion_item(&self) {
        if let Some((model, iter)) = self.tree_view.get_selection().get_selected() {
            if let Ok(model) = model.downcast::<ListStore>() {
                self.select_next();
                model.remove(&iter);
            }
        }
    }

    /// Adjust the policy of the scrolled window to avoid having extra space around the tree view.
    pub fn disable_scrollbars(&self) {
        self.scrolled_window.set_policy(Never, Never);
    }

    /// Filter the completion view.
    fn filter(&mut self, command_entry_text: &str) {
        // Disable the scrollbars so that commands without completion does not
        // show the completion view.
        self.disable_scrollbars();
        let model = self.model.completion.filter(command_entry_text);
        if let Some(model) = model {
            self.adjust_policy(&model);
        }
    }

    /// Get the original input.
    pub fn original_input(&self) -> &str {
        &self.model.original_input
    }

    /// Remove all the columns.
    fn remove_columns(&self) {
        for column in &self.tree_view.get_columns() {
            self.tree_view.remove_column(column);
        }
    }

    /// Scroll to the selected row.
    fn scroll(&self, model: &TreeModel, iter: &TreeIter) {
        if let Some(path) = model.get_path(iter) {
            self.tree_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
        }
    }

    /// Scroll to the first row.
    pub fn scroll_to_first(&self) {
        if let Some(model) = self.tree_view.get_model() {
            if let Some(iter) = model.get_iter_first() {
                if let Some(path) = model.get_path(&iter) {
                    self.tree_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
                }
            }
        }
    }

    /// Select the completer based on the currently typed command.
    pub fn select_completer(&mut self, command_entry_text: &str) {
        let text = command_entry_text.trim_left();
        if let Some(space_index) = text.find(' ') {
            let command = &text[..space_index];
            self.set_completer(command, command_entry_text);
        }
        else {
            self.set_completer(DEFAULT_COMPLETER_IDENT, command_entry_text);
        }
    }

    /// Select the next item.
    /// This loops with the value that started the completion.
    pub fn select_next(&self) -> (Option<TreeSelection>, bool) {
        let mut unselect = false;
        let selection =
            if let Some(model) = self.tree_view.get_model() {
                let selection = self.tree_view.get_selection();
                if let Some((model, selected_iter)) = selection.get_selected() {
                    if model.iter_next(&selected_iter) {
                        selection.select_iter(&selected_iter);
                        self.scroll(&model, &selected_iter);
                    }
                    else {
                        self.unselect();
                        unselect = true;
                    }
                }
                else if let Some(iter) = model.get_iter_first() {
                    self.scroll(&model, &iter);
                    selection.select_iter(&iter);
                }
                Some(selection)
            }
            else {
                None
            };
        (selection, unselect)
    }

    /// Select the previous item.
    /// This loops with the value that started the completion.
    pub fn select_previous(&self) -> (Option<TreeSelection>, bool) {
        let mut unselect = false;
        let selection =
            if let Some(model) = self.tree_view.get_model() {
                let selection = self.tree_view.get_selection();
                if let Some((model, selected_iter)) = selection.get_selected() {
                    if model.iter_previous(&selected_iter) {
                        selection.select_iter(&selected_iter);
                        self.scroll(&model, &selected_iter);
                    }
                    else {
                        self.unselect();
                        unselect = true;
                    }
                }
                else if let Some(iter) = model.iter_nth_child(None, max(0, model.iter_n_children(None) - 1)) {
                    self.scroll(&model, &iter);
                    selection.select_iter(&iter);
                }
                Some(selection)
            }
            else {
                None
            };
        (selection, unselect)
    }

    /// Set the current command completer.
    pub fn set_completer(&mut self, completer: &str, command_entry_text: &str) {
        if self.model.completion.adjust_model(completer) {
            let model: Option<&ListStore> = None;
            self.tree_view.set_model(model);
        }
        {
            let completer = self.model.completion.current_completer().expect("completer should be set");
            self.adjust_columns(&**completer);
        }
        self.filter(command_entry_text);
    }

    /// Set all the completers.
    pub fn set_completers(&mut self, completers: HashMap<String, Box<Completer>>) {
        self.model.completion.set_completers(completers);
    }

    /// Set the original input.
    pub fn set_original_input(&mut self, input: &str) {
        self.model.original_input = input.to_string();
    }

    /// Show the completion view.
    pub fn show_completion(&mut self) {
        self.unselect();
        self.scroll_to_first();
        self.model.visible = true;
    }

    /// Unselect the item.
    pub fn unselect(&self) {
        let selection = self.tree_view.get_selection();
        selection.unselect_all();
    }

    /// Update the completions.
    pub fn update_completions(&mut self, current_mode: &str, command_entry_text: &str) {
        if current_mode == COMMAND_MODE {
            // In command mode, the completer can change when the user type.
            // For instance, after typing "set ", the completer switch to the settings
            // completer.
            // TODO: add command_entry_text in the model?
            self.select_completer(command_entry_text);
        }
        else {
            // Do not select another completer when in input mode.
            self.filter(command_entry_text);
        }
        if self.model.completion.current_completer_ident() != NO_COMPLETER_IDENT {
            self.filter(command_entry_text);
            // return Some(SetOriginalInput(text); // TODO
            self.set_original_input(command_entry_text);
        }
        self.unselect();
    }
}
