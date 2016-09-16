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

use std::cell::Cell;

use gtk::{BoxExt, ContainerExt, CssProvider, EditableExt, Entry, EntryExt, Label, WidgetExt, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::Orientation::Horizontal;

pub type StatusBarWidget = HBox;
pub type HBox = ::gtk::Box;

/// The window status bar.
pub struct StatusBar {
    colon_label: Label,
    entry: Entry,
    entry_shown: Cell<bool>,
    hbox: HBox,
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new() -> Self {
        let hbox = HBox::new(Horizontal, 0);
        hbox.set_size_request(1, 20);

        let colon_label = Label::new(Some(":"));
        hbox.add(&colon_label);

        let entry = Entry::new();
        StatusBar::adjust_entry(&entry);
        hbox.add(&entry);

        StatusBar {
            colon_label: colon_label,
            entry: entry,
            entry_shown: Cell::new(false),
            hbox: hbox,
        }
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
        let style_context = entry.get_style_context().unwrap();
        style_context.add_class("mg-input-command");
        let style = ".mg-input-command {
            box-shadow: none;
            padding: 0;
        }";
        let provider = CssProvider::new();
        provider.load_from_data(style).unwrap();
        style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
        entry.set_has_frame(false);
        entry.set_hexpand(true);
    }

    /// Connect the active entry event.
    pub fn connect_activate<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        self.entry.connect_activate(move |entry| callback(entry.get_text()));
    }

    /// Get whether the entry is shown or not.
    pub fn entry_shown(&self) -> bool {
        self.entry_shown.get()
    }

    /// Hide all the widgets.
    pub fn hide(&self) {
        self.hide_entry();
    }

    /// Hide the entry.
    pub fn hide_entry(&self) {
        self.entry_shown.set(false);
        self.entry.hide();
        self.colon_label.hide();
    }

    /// Set the text of the command entry.
    pub fn set_command(&self, command: &str) {
        self.entry.set_text(command);
        self.entry.set_position(command.len() as i32);
    }

    /// Show the entry.
    pub fn show_entry(&self) {
        self.entry_shown.set(true);
        self.entry.set_text("");
        self.entry.show();
        self.entry.grab_focus();
        self.colon_label.show();
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
        StatusBarItem {
            label: Label::new(None),
            left: false,
        }
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
