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

use std::cell::{RefCell, Cell};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use gdk_sys::GdkRGBA;
use gtk::{
    BoxExt,
    ContainerExt,
    CssProvider,
    EditableExt,
    EditableSignals,
    Entry,
    EntryExt,
    Label,
    WidgetExt,
    STATE_FLAG_NORMAL,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk::prelude::WidgetExtManual;
use gtk::Orientation::Horizontal;
use pango_sys::PangoEllipsizeMode;

use completion::{Completer, Completion, CompletionView, DEFAULT_COMPLETER_IDENT, NO_COMPLETER_IDENT};
use gtk_widgets::LabelExtManual;
use super::COMMAND_MODE;

const BLUE: &'static GdkRGBA = &GdkRGBA { red: 0.0, green: 0.0, blue: 1.0, alpha: 1.0 };
const ORANGE: &'static GdkRGBA = &GdkRGBA { red: 0.9, green: 0.55, blue: 0.0, alpha: 1.0 };
const RED: &'static GdkRGBA = &GdkRGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };
const WHITE: &'static GdkRGBA = &GdkRGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 };

pub type StatusBarWidget = HBox;
pub type HBox = ::gtk::Box;

/// The window status bar.
pub struct StatusBar {
    completion: Completion,
    completion_original_input: RefCell<String>,
    completion_view: Rc<CompletionView>,
    current_mode: Rc<RefCell<String>>,
    entry: Entry,
    entry_shown: Cell<bool>,
    identifier_label: Label,
    inserting_completion: Cell<bool>,
    hbox: HBox,
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new(completion_view: Rc<CompletionView>, completers: HashMap<String, Box<Completer>>, current_mode: Rc<RefCell<String>>) -> Rc<Self> {
        let hbox = HBox::new(Horizontal, 0);
        hbox.set_size_request(1, 20);

        let identifier_label = Label::new(Some(":"));
        hbox.add(&identifier_label);

        let entry = Entry::new();
        let completion = Completion::new(completers, completion_view.clone());

        StatusBar::adjust_entry(&entry);
        hbox.add(&entry);

        let status_bar = Rc::new(StatusBar {
            completion: completion,
            completion_original_input: RefCell::new(String::new()),
            completion_view: completion_view,
            current_mode: current_mode,
            entry: entry,
            entry_shown: Cell::new(false),
            identifier_label: identifier_label,
            inserting_completion: Cell::new(false),
            hbox: hbox,
        });

        {
            let status_bar2 = status_bar.clone();
            status_bar.completion_view.connect_selection_changed(move |selection| {
                if let Some(completion) = status_bar2.completion.complete_result(selection) {
                    status_bar2.set_input(&completion);
                }
            });
        }

        {
            let status_bar2 = status_bar.clone();
            status_bar.completion_view.connect_unselect(move || {
                let text = (*status_bar2.completion_original_input.borrow()).clone();
                status_bar2.set_input(&text);
            });
        }

        {
            let status_bar2 = status_bar.clone();
            status_bar.entry.connect_changed(move |_| {
                status_bar2.update_completions();
            });
        }

        status_bar
    }

    /// Add an item.
    pub fn add_item(&self, item: &StatusBarItem) {
        item.label.show();
        if item.left {
            self.hbox.pack_start(item, false, false, 3);
        }
        else {
            self.hbox.pack_end(item, false, false, 3);
        }
    }

    /// Adjust the look of the entry.
    fn adjust_entry(entry: &Entry) {
        entry.set_name("mg-input-command");
        let style_context = entry.get_style_context().unwrap();
        let style = include_str!("../style/command-input.css");
        let provider = CssProvider::new();
        provider.load_from_data(style).unwrap();
        style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
        entry.set_has_frame(false);
        entry.set_hexpand(true);
    }

    /// Color the status bar in blue.
    pub fn color_blue(&self) {
        self.override_background_color(STATE_FLAG_NORMAL, BLUE);
        self.white_foreground();
    }

    /// Color the status bar in orange.
    pub fn color_orange(&self) {
        self.override_background_color(STATE_FLAG_NORMAL, ORANGE);
        self.white_foreground();
    }

    /// Color the status bar in red.
    pub fn color_red(&self) {
        self.override_background_color(STATE_FLAG_NORMAL, RED);
        self.white_foreground();
    }

    /// Select the next completion entry if it is visible.
    pub fn complete_next(&self) {
        self.completion_view.select_next();
    }

    /// Select the previous completion entry if it is visible.
    pub fn complete_previous(&self) {
        self.completion_view.select_previous();
    }

    /// Connect the active entry event.
    pub fn connect_activate<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        self.entry.connect_activate(move |entry| callback(entry.get_text()));
    }

    /// Get whether the entry is shown or not.
    pub fn entry_shown(&self) -> bool {
        self.entry_shown.get()
    }

    /// Filter the completion view.
    fn filter(&self) {
        // Disable the scrollbars so that commands without completion does not
        // show the completion view.
        self.completion_view.disable_scrollbars();
        if let (Some(text), Some(completer)) = (self.entry.get_text(), self.completion.current_completer()) {
            self.completion_view.filter(&text, completer);
        }
    }

    /// Get the text of the command entry.
    pub fn get_command(&self) -> Option<String> {
        self.entry.get_text()
    }

    /// Hide all the widgets.
    pub fn hide(&self) {
        self.hide_entry();
    }

    /// Hide the entry.
    pub fn hide_entry(&self) {
        self.entry.set_text("");
        self.entry_shown.set(false);
        self.entry.hide();
        self.identifier_label.hide();
    }

    /// Select the completer based on the currently typed command.
    pub fn select_completer(&self) {
        if let Some(text) = self.entry.get_text() {
            let text = text.trim_left();
            if let Some(space_index) = text.find(' ') {
                let command = &text[..space_index];
                self.set_completer(command);
            }
            else {
                self.set_completer(DEFAULT_COMPLETER_IDENT);
            }
        }
    }

    /// Set the current command completer.
    pub fn set_completer(&self, completer: &str) {
        self.completion.adjust_model(completer);
        self.filter();
    }

    /// Set the text of the input entry and move the cursor at the end.
    pub fn set_input(&self, command: &str) {
        self.inserting_completion.set(true);
        self.entry.set_text(command);
        self.entry.set_position(command.len() as i32);
        self.inserting_completion.set(false);
    }

    /// Set the identifier label text.
    pub fn set_identifier(&self, identifier: &str) {
        self.identifier_label.set_text(identifier);
    }

    /// Set the original input.
    pub fn set_original_input(&self, input: &str) {
        *self.completion_original_input.borrow_mut() = input.to_string();
    }

    /// Show the entry.
    pub fn show_entry(&self) {
        self.entry_shown.set(true);
        self.entry.set_text("");
        self.entry.show();
        self.entry.grab_focus();
        self.show_identifier();
    }

    /// Show the identifier label.
    pub fn show_identifier(&self) {
        self.identifier_label.show();
    }

    /// Update the completions.
    pub fn update_completions(&self) {
        let in_command_mode = (*self.current_mode.borrow()) == COMMAND_MODE;
        if !self.inserting_completion.get() {
            if in_command_mode {
                // In command mode, the completer can change when the user type.
                // For instance, after typing "set ", the completer switch to the settings
                // completer.
                self.select_completer();
            }
            else {
                // Do not select another completer when in input mode.
                self.filter();
            }
            let current_completer = self.completion.current_completer_ident();
            if current_completer != NO_COMPLETER_IDENT {
                if let Some(text) = self.entry.get_text() {
                    self.filter();
                    *self.completion_original_input.borrow_mut() = text;
                }
            }
            self.completion_view.unselect();
        }
    }

    /// Set the foreground (text) color to white.
    pub fn white_foreground(&self) {
        self.override_color(STATE_FLAG_NORMAL, WHITE);
    }
}

impl Deref for StatusBar {
    type Target = HBox;

    fn deref(&self) -> &Self::Target {
        &self.hbox
    }
}

is_widget!(StatusBar, hbox);

/// A status bar text item.
pub struct StatusBarItem {
    label: Label,
    left: bool,
}

impl StatusBarItem {
    /// Create a new status bar item.
    #[allow(new_without_default)]
    pub fn new() -> Self {
        let label = Label::new(None);
        label.set_ellipsize(PangoEllipsizeMode::End);
        StatusBarItem {
            label: label,
            left: false,
        }
    }

    /// Get the item text.
    pub fn get_text(&self) -> Option<String> {
        self.label.get_text()
    }

    /// Set the item to be on the left.
    pub fn left(mut self) -> Self {
        self.left = true;
        self
    }

    /// Set the item text.
    pub fn set_text(&self, text: &str) {
        self.label.set_text(text);
        self.label.show();
    }
}

is_widget!(StatusBarItem, label);
