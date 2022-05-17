/*
 * Copyright (c) 2016-2020 Boucher, Antoni <bouanto@zoho.com>
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

use glib::{Cast, Object};
use glib::object::IsA;
use gtk;
use gtk::{
    traits::{
        CellRendererTextExt,
        GtkListStoreExt,
        ScrolledWindowExt,
        TreeModelExt,
        TreeSelectionExt,
        TreeViewColumnExt,
        TreeViewExt,
        WidgetExt,
    },
    Align,
    CellRendererText,
    ListStore,
    TreeIter,
    TreeModel,
    TreeViewColumn,
};
use gtk::PolicyType::{Automatic, Never};
use pango::EllipsizeMode;
use relm::{Relm, Widget};
use relm_derive::widget;

use app::COMMAND_MODE;
use completion::Completers;
use completion::Column::{self, Expand};
use self::Msg::*;
use super::{Completer, Completion, DEFAULT_COMPLETER_IDENT, NO_COMPLETER_IDENT};

const COMPLETION_VIEW_MAX_HEIGHT: i32 = 300;

#[allow(missing_docs)]
pub struct Model {
    completion: Completion,
    original_input: String,
    relm: Relm<CompletionView>,
    visible: bool,
}

pub type Mode = String;
pub type Text = String;

#[derive(Msg)]
pub enum Msg {
    AddCompleters(Completers),
    Completer(String),
    CompletionChange(String),
    DeleteCurrentCompletionItem,
    SelectNext,
    SelectPrevious,
    SetOriginalInput(String),
    ShowCompletion,
    UpdateCompletions(Mode, Text, bool),
    Visible(bool),
}

/// A widget to show completions for the command entry.
#[widget]
impl Widget for CompletionView {
    fn add_completers(&mut self, completers: Completers) {
        for (ident, completer) in completers {
            self.model.completion.add_completer(ident, completer);
        }
    }

    fn init_view(&mut self) {
        self.add_columns(2);
    }

    fn model(relm: &Relm<Self>, completers: Completers) -> Model {
        let mut completion = Completion::new();
        completion.set_completers(completers);
        Model {
            completion,
            original_input: String::new(),
            relm: relm.clone(),
            visible: false,
        }
    }

    /// Show the completion view.
    fn show_completion(&mut self) {
        self.unselect();
        self.scroll_to_first();
        self.model.visible = true;
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            AddCompleters(completers) => self.add_completers(completers),
            Completer(completer) => self.set_completer(&completer, ""),
            // NOTE: to be listened by the user.
            CompletionChange(_) => (),
            DeleteCurrentCompletionItem => self.delete_current_completion_item(),
            SelectNext => self.select_next(),
            SelectPrevious => self.select_previous(),
            SetOriginalInput(input) => self.set_original_input(&input),
            ShowCompletion => self.show_completion(),
            UpdateCompletions(mode, text, is_normal_command) =>
                self.update_completions(&mode, &text, is_normal_command),
            Visible(visible) => self.model.visible = visible,
        }
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
            cell.set_ellipsize(EllipsizeMode::End);
            view_column.set_expand(true);
        }
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", index);
        view_column.add_attribute(&cell, "foreground", foreground_index);
        self.widgets.tree_view.append_column(&view_column);
    }

    /// Add the specified number of columns.
    fn add_columns(&self, column_count: i32) {
        self.remove_columns();
        for i in 0 .. column_count {
            self.add_column(i, column_count + i, Expand);
        }
    }

    /// Add the specified number of columns.
    fn add_columns_from_completer(&self, completer: &dyn Completer) {
        self.remove_columns();
        let columns = completer.columns();
        let column_count = columns.len() as i32;
        for (i, column) in columns.iter().enumerate() {
            let i = i as i32;
            self.add_column(i, column_count + i, *column);
        }
    }

    /// Adjust the columns from the completer.
    fn adjust_columns(&self, completer: &dyn Completer) {
        self.add_columns_from_completer(completer);
    }

    /// Adjust the policy of the scrolled window to avoid having extra space around the tree view.
    fn adjust_policy<M: IsA<Object> + IsA<TreeModel>>(&self, model: &M) {
        self.widgets.tree_view.set_model(Some(model));
        let policy =
            if model.iter_n_children(None) < 2 {
                Never
            }
            else {
                Automatic
            };
        self.widgets.scrolled_window.set_policy(Never, policy);
    }

    /// Complete the result for the selection using the current completer.
    fn complete_result(&self) {
        let selection = self.widgets.tree_view.selection();
        if let Some(completion) = self.model.completion.complete_result(&selection) {
            self.model.relm.stream().emit(CompletionChange(completion));
        }
    }

    /// Delete the current completion item.
    fn delete_current_completion_item(&self) {
        if let Some((model, iter)) = self.widgets.tree_view.selection().selected() {
            if let Ok(model) = model.downcast::<ListStore>() {
                self.select_next();
                model.remove(&iter);
                self.adjust_policy(&model);
            }
        }
    }

    /// Adjust the policy of the scrolled window to avoid having extra space around the tree view.
    fn disable_scrollbars(&self) {
        self.widgets.scrolled_window.set_policy(Never, Never);
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

    /// Remove all the columns.
    fn remove_columns(&self) {
        for column in &self.widgets.tree_view.columns() {
            self.widgets.tree_view.remove_column(column);
        }
    }

    /// Scroll to the selected row.
    fn scroll(&self, model: &TreeModel, iter: &TreeIter) {
        if let Some(path) = model.path(iter) {
            self.widgets.tree_view.scroll_to_cell(Some(&path), None::<&TreeViewColumn>, false, 0.0, 0.0);
        }
    }

    /// Scroll to the first row.
    fn scroll_to_first(&self) {
        if let Some(model) = self.widgets.tree_view.model() {
            if let Some(iter) = model.iter_first() {
                if let Some(path) = model.path(&iter) {
                    self.widgets.tree_view.scroll_to_cell(Some(&path), None::<&TreeViewColumn>, false, 0.0, 0.0);
                }
            }
        }
    }

    /// Select the completer based on the currently typed command.
    fn select_completer(&mut self, command_entry_text: &str, is_normal_command: bool) {
        let text = command_entry_text.trim_start();
        let completer =
            if let Some(space_index) = text.find(' ') {
                &text[..space_index]
            }
            else if is_normal_command {
                DEFAULT_COMPLETER_IDENT
            }
            else {
                NO_COMPLETER_IDENT
            };
        self.set_completer(completer, command_entry_text);
    }

    /// Select the next item.
    /// This loops with the value that started the completion.
    fn select_next(&self) {
        if let Some(model) = self.widgets.tree_view.model() {
            let selection = self.widgets.tree_view.selection();
            if let Some((model, selected_iter)) = selection.selected() {
                if model.iter_next(&selected_iter) {
                    selection.select_iter(&selected_iter);
                    self.scroll(&model, &selected_iter);
                }
                else {
                    self.unselect();
                    self.model.relm.stream().emit(CompletionChange(self.model.original_input.clone()));
                }
            }
            else if let Some(iter) = model.iter_first() {
                self.scroll(&model, &iter);
                selection.select_iter(&iter);
            }
            self.complete_result();
        }
    }

    /// Select the previous item.
    /// This loops with the value that started the completion.
    fn select_previous(&self) {
        if let Some(model) = self.widgets.tree_view.model() {
            let selection = self.widgets.tree_view.selection();
            if let Some((model, selected_iter)) = selection.selected() {
                if model.iter_previous(&selected_iter) {
                    selection.select_iter(&selected_iter);
                    self.scroll(&model, &selected_iter);
                }
                else {
                    self.unselect();
                    self.model.relm.stream().emit(CompletionChange(self.model.original_input.clone()));
                }
            }
            else if let Some(iter) = model.iter_nth_child(None, max(0, model.iter_n_children(None) - 1)) {
                self.scroll(&model, &iter);
                selection.select_iter(&iter);
            }
            self.complete_result();
        }
    }

    /// Set the current command completer.
    fn set_completer(&mut self, completer: &str, command_entry_text: &str) {
        if self.model.completion.adjust_model(completer) {
            let model: Option<&ListStore> = None;
            self.widgets.tree_view.set_model(model);
        }
        {
            let completer = self.model.completion.current_completer().expect("completer should be set");
            self.adjust_columns(completer);
        }
        self.filter(command_entry_text);
    }

    /// Set the original input.
    fn set_original_input(&mut self, input: &str) {
        self.model.original_input = input.to_string();
    }

    /// Unselect the item.
    fn unselect(&self) {
        let selection = self.widgets.tree_view.selection();
        selection.unselect_all();
    }

    /// Update the completions.
    fn update_completions(&mut self, current_mode: &str, command_entry_text: &str, is_normal_command: bool) {
        if current_mode == COMMAND_MODE {
            // In command mode, the completer can change when the user type.
            // For instance, after typing "set ", the completer switch to the settings
            // completer.
            // TODO: add command_entry_text in the model?
            self.select_completer(command_entry_text, is_normal_command);
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
