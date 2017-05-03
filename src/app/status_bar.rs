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
use std::ops::Deref;

use gdk::RGBA;
use gtk;
use gtk::{
    BoxExt,
    ContainerExt,
    CssProvider,
    EditableExt,
    Entry,
    EntryExt,
    Label,
    OrientableExt,
    PackType,
    TreeSelection,
    WidgetExt,
    STATE_FLAG_NORMAL,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk::prelude::WidgetExtManual;
use gtk::Orientation::Horizontal;
use pango_sys::PangoEllipsizeMode;
use relm::{Container, Widget};
use relm::gtk_ext::BoxExtManual;
use relm_attributes::widget;

use app::COMMAND_MODE;
use completion::{Completer, Completion, CompletionView, DEFAULT_COMPLETER_IDENT, NO_COMPLETER_IDENT};
use gtk_widgets::LabelExtManual;
use self::Msg::*;

const BLUE: &RGBA = &RGBA { red: 0.0, green: 0.0, blue: 1.0, alpha: 1.0 };
const ORANGE: &RGBA = &RGBA { red: 0.9, green: 0.55, blue: 0.0, alpha: 1.0 };
const RED: &RGBA = &RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };
const WHITE: &RGBA = &RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 };

pub type HBox = ::gtk::Box;

#[derive(Msg)]
pub enum Msg {
    Activate(Option<String>),
}

#[derive(Clone)]
pub struct Model {
    identifier_label: &'static str,
}

#[widget]
impl Widget for StatusBar {
    fn init_view(&self) {
        // Adjust the look of the entry.
        let style_context = self.command_entry.get_style_context().unwrap();
        // TODO: remove the next line when relm supports css.
        let style = include_str!("../../style/command-input.css");
        let provider = CssProvider::new();
        provider.load_from_data(style).unwrap();
        style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    fn model() -> Model {
        Model {
            identifier_label: ":",
        }
    }

    fn update(&mut self, msg: Msg) {
    }

    view! {
        #[container]
        gtk::Box {
            property_height_request: 20, // TODO: is this still useful?
            orientation: Horizontal,
            #[name="identifier_label"]
            gtk::Label {
                text: self.model.identifier_label,
            },
            #[name="command_entry"]
            gtk::Entry {
                activate(entry) => Activate(entry.get_text()),
                has_frame: false,
                hexpand: true,
                name: "mg-input-command",
            },
        }
    }
}

impl StatusBar {
    /// Color the status bar in red.
    pub fn color_red(&self) {
        self.root().override_background_color(STATE_FLAG_NORMAL, &RED);
        self.white_foreground();
    }

    /// Select the next completion entry if it is visible.
    pub fn complete_next(&self) {
        self.completion.view.select_next();
    }

    /// Select the previous completion entry if it is visible.
    pub fn complete_previous(&self) {
        self.completion.view.select_previous();
    }

    /// Connect the active entry event.
    pub fn connect_activate<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        self.command_entry.connect_activate(move |entry| callback(entry.get_text()));
    }

    /// Delete the current completion item.
    pub fn delete_current_completion_item(&self) {
        self.completion.delete_current_completion_item();
    }

    /// Filter the completion view.
    fn filter(&mut self) {
        // Disable the scrollbars so that commands without completion does not
        // show the completion view.
        self.completion.view.disable_scrollbars();
        if let Some(text) = self.entry.get_text() {
            self.completion.filter(&text);
        }
    }

    /// Get the text of the command entry.
    pub fn get_command(&self) -> Option<String> {
        self.entry.get_text()
    }

    /// Handle the unselect event.
    fn handle_unselect(&mut self) {
        let original_input = self.completion_original_input.clone();
        self.set_input(&original_input);
    }

    /// Hide the completion view.
    pub fn hide_completion(&self) {
        self.completion.view.hide();
    }

    /// Hide the entry.
    pub fn hide_entry(&self) {
        self.set_entry_shown(false);
    }

    /// Hide the completion view and the entry.
    pub fn hide_widgets(&self) {
        //self.hide_completion();
        self.hide_entry();
    }

    /// Select the completer based on the currently typed command.
    pub fn select_completer(&mut self) {
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

    /// Handle the selection changed event.
    fn selection_changed(&mut self, selection: &TreeSelection) {
        if let Some(completion) = self.completion.complete_result(selection) {
            self.set_input(&completion);
        }
    }

    /// Set the current command completer.
    pub fn set_completer(&mut self, completer: &str) {
        self.completion.adjust_model(completer);
        self.filter();
    }

    // TODO: merge with show_entry()?
    pub fn set_entry_shown(&self, visible: bool) {
        self.command_entry.set_text("");
        self.identifier_label.set_visible(visible);
        self.command_entry.set_visible(visible);
    }

    pub fn set_identifier(&self, identifier: &str) {
        self.identifier_label.set_text(identifier);
    }

    /// Set the text of the input entry and move the cursor at the end.
    pub fn set_input(&self, command: &str) {
        // Prevent updating the completions when the user selects a completion entry.
        // TODO: use a kind of lock for that?
        //self.inserting_completion = true;
        self.command_entry.set_text(command);
        self.command_entry.set_position(command.len() as i32);
        //self.inserting_completion = false;
    }

    /// Set the original input.
    pub fn set_original_input(&mut self, input: &str) {
        self.completion_original_input = input.to_string();
    }

    /// Show the completion view.
    pub fn show_completion(&self) {
        self.completion.view.unselect();
        self.completion.view.scroll_to_first();
        self.completion.view.show();
    }

    /// Show the entry.
    pub fn show_entry(&self) {
        self.command_entry.grab_focus();
    }

    /// Update the completions.
    pub fn update_completions(&mut self, current_mode: &str) {
        if !self.inserting_completion {
            if current_mode == COMMAND_MODE {
                // In command mode, the completer can change when the user type.
                // For instance, after typing "set ", the completer switch to the settings
                // completer.
                self.select_completer();
            }
            else {
                // Do not select another completer when in input mode.
                self.filter();
            }
            if self.completion.current_completer_ident() != NO_COMPLETER_IDENT {
                if let Some(text) = self.entry.get_text() {
                    self.filter();
                    self.completion_original_input = text;
                }
            }
            self.completion.view.unselect();
        }
    }

    /// Set the foreground (text) color to white.
    pub fn white_foreground(&self) {
        self.root().override_color(STATE_FLAG_NORMAL, &WHITE);
    }
}

/// A status bar text item.
#[widget]
impl Widget for StatusBarItem {
    fn model() -> () {
        ()
    }

    fn update(&mut self, msg: ()) {
    }

    view! {
        #[parent="status-bar-item"]
        #[name="label"]
        gtk::Label {
            ellipsize: PangoEllipsizeMode::End,
            packing: {
                pack_type: PackType::End,
                padding: 3,
            },
        }
    }
}

impl StatusBarItem {
    pub fn set_text(&self, text: &str) {
        self.label.set_text(text);
    }
}

/*
/// The window status bar.
pub struct StatusBar {
    completion: Completion,
    completion_original_input: String,
    pub entry: Entry,
    entry_shown: bool,
    identifier_label: Label,
    inserting_completion: bool,
    hbox: HBox,
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new(completion_view: Box<CompletionView>, completers: HashMap<String, Box<Completer>>) -> Box<Self> {
        let completion = Completion::new(completers, completion_view);

        let mut status_bar = Box::new(StatusBar {
            completion: completion,
            completion_original_input: String::new(),
            entry: entry,
            entry_shown: false,
            identifier_label: identifier_label,
            inserting_completion: false,
            hbox: hbox,
        });

        //connect!(status_bar.completion.view, connect_selection_changed(selection), status_bar, selection_changed(selection));
        //connect!(status_bar.completion.view, connect_unselect, status_bar, handle_unselect);

        status_bar
    }

    /// Add an item.
    pub fn add_item(&self, item: &StatusBarItem) {
        item.label.show();
        if item.left {
            self.hbox.pack_start(&**item, false, false, 3);
        }
        else {
            self.hbox.pack_end(&**item, false, false, 3);
        }
    }

    /// Color the status bar in blue.
    pub fn color_blue(&self) {
        self.override_background_color(STATE_FLAG_NORMAL, &BLUE);
        self.white_foreground();
    }

    /// Color the status bar in orange.
    pub fn color_orange(&self) {
        self.override_background_color(STATE_FLAG_NORMAL, &ORANGE);
        self.white_foreground();
    }
}
*/
