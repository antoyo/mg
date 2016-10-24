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

use std::cell::RefCell;
use std::cmp::max;
use std::ops::Deref;
use std::rc::Rc;

use glib::Object;
use gtk::{
    CellRendererText,
    ContainerExt,
    IsA,
    ListStore,
    ScrolledWindow,
    ScrolledWindowExt,
    TreeIter,
    TreeModel,
    TreeModelExt,
    TreeSelection,
    TreeView,
    TreeViewColumn,
    Type,
    WidgetExt,
};
use gtk::PolicyType::{Automatic, Never};

use scrolled_window::ScrolledWindowExtManual;
use super::Completer;

const COMPLETION_VIEW_MAX_HEIGHT: i32 = 300;

/// A widget to show completions for the command entry.
pub struct CompletionView {
    unselect_callback: RefCell<Option<Box<Fn()>>>,
    tree_view: TreeView,
    view: ScrolledWindow,
}

impl CompletionView {
    /// Create a new completion view.
    pub fn new() -> Rc<Self> {
        let tree_view = TreeView::new();

        let column1 = TreeViewColumn::new();
        let cell1 = CellRendererText::new();
        column1.pack_start(&cell1, true);
        column1.add_attribute(&cell1, "text", 0);
        tree_view.append_column(&column1);

        let column2 = TreeViewColumn::new();
        let cell2 = CellRendererText::new();
        column2.pack_start(&cell2, true);
        column2.add_attribute(&cell2, "text", 1);
        tree_view.append_column(&column2);

        tree_view.get_selection().unselect_all();
        tree_view.set_enable_search(false);
        tree_view.set_headers_visible(false);
        tree_view.set_hexpand(true);
        tree_view.set_can_focus(false);

        let scrolled_window = ScrolledWindow::new(None, None);
        scrolled_window.add(&tree_view);
        scrolled_window.set_max_content_height(COMPLETION_VIEW_MAX_HEIGHT);
        scrolled_window.set_propagate_natural_height(true);

        Rc::new(CompletionView {
            unselect_callback: RefCell::new(None),
            tree_view: tree_view,
            view: scrolled_window,
        })
    }

    /// Adjust the policy of the scrolled window to avoid having extra space around the tree view.
    fn adjust_policy<M: IsA<Object> + IsA<TreeModel>>(&self, model: &M) {
        let policy =
            if model.iter_n_children(None) < 2 {
                Never
            }
            else {
                Automatic
            };
        self.view.set_policy(Never, policy);
    }

    /// Add a callback to the selection changed event.
    pub fn connect_selection_changed<F: Fn(&TreeSelection) + 'static>(&self, callback: F) {
        self.tree_view.get_selection().connect_changed(callback);
    }

    /// Add a callback to the unselect event.
    pub fn connect_unselect<F: Fn() + 'static>(&self, callback: F) {
        *self.unselect_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Filter the rows from the input.
    pub fn filter(&self, input: &str, completer: &Completer) {
        let model = ListStore::new(&[Type::String, Type::String]);

        let key =
            if let Some(index) = input.find(' ') {
                input[index + 1 ..].trim_left()
            }
            else {
                input
            };
        let key = key.to_lowercase();
        for &(ref col1, ref col2) in &completer.completions(&key) {
            model.insert_with_values(None, &[0, 1], &[&col1, &col2]);
        }
        self.tree_view.set_model(Some(&model));
        self.adjust_policy(&model);
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

    /// Select the next item.
    /// This loops with the value that started the completion.
    pub fn select_next(&self) {
        if let Some(model) = self.tree_view.get_model() {
            let selection = self.tree_view.get_selection();
            if let Some((model, selected_iter)) = selection.get_selected() {
                if model.iter_next(&selected_iter) {
                    selection.select_iter(&selected_iter);
                    self.scroll(&model, &selected_iter);
                }
                else {
                    self.unselect();
                    self.show_original_input();
                }
            }
            else if let Some(iter) = model.get_iter_first() {
                self.scroll(&model, &iter);
                selection.select_iter(&iter);
            }
        }
    }

    /// Select the previous item.
    /// This loops with the value that started the completion.
    pub fn select_previous(&self) {
        if let Some(model) = self.tree_view.get_model() {
            let selection = self.tree_view.get_selection();
            if let Some((model, selected_iter)) = selection.get_selected() {
                if model.iter_previous(&selected_iter) {
                    selection.select_iter(&selected_iter);
                    self.scroll(&model, &selected_iter);
                }
                else {
                    self.unselect();
                    self.show_original_input();
                }
            }
            else if let Some(iter) = model.iter_nth_child(None, max(0, model.iter_n_children(None) - 1)) {
                self.scroll(&model, &iter);
                selection.select_iter(&iter);
            }
        }
    }

    /// Set the model.
    pub fn set_model<T: IsA<TreeModel>>(&self, model: Option<&T>) {
        self.tree_view.set_model(model);
    }

    /// Emit the event to show the original input.
    /// This is emitted when the user unselect from the completion view.
    fn show_original_input(&self) {
        if let Some(ref callback) = *self.unselect_callback.borrow() {
            callback();
        }
    }

    /// Unselect the item.
    pub fn unselect(&self) {
        let selection = self.tree_view.get_selection();
        selection.unselect_all();
    }
}

impl Deref for CompletionView {
    type Target = ScrolledWindow;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}
