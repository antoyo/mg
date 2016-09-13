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

use gtk::{ContainerExt, Entry, EntryExt, Label, WidgetExt};
use gtk::Orientation::Horizontal;

pub type StatusBarWidget = HBox;
pub type HBox = ::gtk::Box;

/// The window status bar.
pub struct StatusBar {
    entry: Entry,
    label: Label,
    hbox: HBox,
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new() -> Self {
        let hbox = HBox::new(Horizontal, 0);

        let label = Label::new(Some(":"));
        hbox.add(&label);

        let entry = Entry::new();
        entry.set_has_frame(false);
        hbox.add(&entry);

        StatusBar {
            entry: entry,
            label: label,
            hbox: hbox,
        }
    }

    /// Connect the active entry event.
    pub fn connect_activate<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        self.entry.connect_activate(move |entry| callback(entry.get_text()));
    }

    /// Hide the entry.
    pub fn hide_entry(&self) {
        self.entry.hide();
        self.label.hide();
    }

    /// Show the entry.
    pub fn show_entry(&self) {
        self.entry.set_text("");
        self.entry.show();
        self.entry.grab_focus();
        self.label.show();
    }

    /// Convert to a widget.
    pub fn widget(&self) -> &StatusBarWidget {
        &self.hbox
    }
}
