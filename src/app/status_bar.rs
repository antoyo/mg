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

use gdk::{RGBA, SELECTION_PRIMARY};
use gtk;
use gtk::{
    BoxExt,
    Clipboard,
    CssProvider,
    CssProviderExt,
    EditableExt,
    EditableSignals,
    EntryExt,
    LabelExt,
    OrientableExt,
    PackType,
    StateFlags,
    StyleContextExt,
    WidgetExt,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk::Orientation::Horizontal;
use pango::EllipsizeMode;
use relm::{Relm, Widget};
use relm_derive::widget;

use self::Msg::*;
use self::ItemMsg::{Color, Text};

#[derive(Msg)]
pub enum Msg {
    BarVisible(bool),
    Copy,
    Cut,
    DeleteNextChar,
    DeleteNextWord,
    DeletePreviousWord,
    End,
    EntryActivate(String),
    EntryChanged(String),
    EntryText(String),
    EntryShown(bool),
    Identifier(String),
    NextChar,
    NextWord,
    Paste,
    PasteSelection,
    PreviousChar,
    PreviousWord,
    ShowIdentifier,
    SmartHome,
}

pub struct Model {
    identifier_label: &'static str,
    identifier_visible: bool,
    relm: Relm<StatusBar>,
    visible: bool,
}

#[widget]
impl Widget for StatusBar {
    fn init_view(&mut self) {
        // Adjust the look of the entry.
        let style_context = self.widgets.command_entry.get_style_context();
        // TODO: remove the next line when relm supports css.
        let style = include_bytes!("../../style/command-input.css");
        let provider = CssProvider::new();
        provider.load_from_data(style).unwrap();
        style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            identifier_label: ":",
            identifier_visible: false,
            relm: relm.clone(),
            visible: true,
        }
    }

    /// Set whether the entry is visible or not.
    fn set_entry_shown(&mut self, visible: bool) {
        // TODO: document this use of lock.
        let _lock = self.model.relm.stream().lock();
        self.widgets.command_entry.set_text("");
        self.model.identifier_visible = visible;
        self.widgets.command_entry.set_visible(visible); // TODO: use a field model and a binding.

        if visible {
            self.widgets.command_entry.grab_focus();
        }
    }

    /// Show the identifier.
    fn show_identifier(&mut self) {
        self.model.identifier_visible = true;
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            BarVisible(visible) => self.model.visible = visible,
            Copy => self.copy(),
            Cut => self.cut(),
            DeleteNextChar => self.delete_next_char(),
            DeleteNextWord => self.delete_next_word(),
            DeletePreviousWord => self.delete_previous_word(),
            End => self.end(),
            EntryActivate(_) | EntryChanged(_) => (), // NOTE: to be listened by the user.
            EntryShown(visible) => self.set_entry_shown(visible),
            EntryText(input) => self.set_input(&input),
            Identifier(identifier) => self.set_identifier(&identifier),
            NextChar => self.next_char(),
            NextWord => self.next_word(),
            Paste => self.paste(),
            PasteSelection => self.paste_selection(),
            PreviousChar => self.previous_char(),
            PreviousWord => self.previous_word(),
            ShowIdentifier => self.show_identifier(),
            SmartHome => self.smart_home(),
        }
    }

    view! {
        #[container]
        gtk::Box {
            property_height_request: 20, // TODO: is this still useful?
            orientation: Horizontal,
            visible: self.model.visible,
            #[name="identifier_label"]
            gtk::Label {
                text: self.model.identifier_label,
                visible: self.model.identifier_visible,
            },
            #[name="command_entry"]
            gtk::Entry {
                activate(entry) => EntryActivate(entry.get_text().to_string()),
                changed(entry) => EntryChanged(entry.get_text().to_string()),
                has_frame: false,
                hexpand: true,
                widget_name: "mg-input-command",
            },
        }
    }
}

impl StatusBar {
    /// Copy the selection to the clipboard.
    fn copy(&self) {
        self.widgets.command_entry.copy_clipboard();
    }

    /// Cut the selection to the clipboard.
    fn cut(&self) {
        self.widgets.command_entry.cut_clipboard();
    }

    /// Delete the character after the cursor.
    fn delete_next_char(&self) {
        if !self.delete_selection() {
            let text = self.get_command();
            if !text.is_empty() {
                let pos = self.widgets.command_entry.get_position();
                let len = text.chars().count();
                if pos < len as i32 {
                    // NOTE: Lock to avoid moving the cursor when updating the text entry.
                    let _lock = self.model.relm.stream().lock();
                    self.widgets.command_entry.delete_text(pos, pos + 1);
                }
            }
        }
        self.emit_entry_changed();
    }

    /// Delete the word after the cursor.
    fn delete_next_word(&self) {
        if !self.delete_selection() {
            let text = self.get_command();
            if !text.is_empty() {
                let pos = self.widgets.command_entry.get_position();
                let end = text.chars().enumerate()
                    .skip(pos as usize)
                    .skip_while(|&(_, c)| !c.is_alphanumeric())
                    .skip_while(|&(_, c)| c.is_alphanumeric())
                    .map(|(index, _)| index)
                    .next()
                    .unwrap_or_else(|| text.chars().count());
                // NOTE: Lock to avoid moving the cursor when updating the text entry.
                let _lock = self.model.relm.stream().lock();
                self.widgets.command_entry.delete_text(pos, end as i32);
            }
        }
        self.emit_entry_changed();
    }

    /// Delete the word before the cursor.
    fn delete_previous_word(&self) {
        if !self.delete_selection() {
            let text = self.get_command();
            if !text.is_empty() {
                let pos = self.widgets.command_entry.get_position();
                let len = text.chars().count();
                let start = text.chars().rev().enumerate()
                    .skip(len - pos as usize)
                    .skip_while(|&(_, c)| !c.is_alphanumeric())
                    .skip_while(|&(_, c)| c.is_alphanumeric())
                    .map(|(index, _)| len - index)
                    .next()
                    .unwrap_or_default();
                // NOTE: Lock to avoid moving the cursor when updating the text entry.
                let _lock = self.model.relm.stream().lock();
                self.widgets.command_entry.delete_text(start as i32, pos);
            }
        }
        self.emit_entry_changed();
    }

    /// Delete the selected text.
    fn delete_selection(&self) -> bool {
        if self.widgets.command_entry.get_selection_bounds().is_some() {
            // NOTE: Lock to avoid moving the cursor when updating the text entry.
            let _lock = self.model.relm.stream().lock();
            self.widgets.command_entry.delete_selection();
            true
        }
        else {
            false
        }
    }

    /// Emit the EntryChanged event with the current text.
    fn emit_entry_changed(&self) {
        self.model.relm.stream().emit(EntryChanged(self.widgets.command_entry.get_text().to_string()));
    }

    /// Go to the end of the command entry.
    fn end(&self) {
        let text = self.get_command();
        self.widgets.command_entry.set_position(text.chars().count() as i32);
    }

    /// Get the text of the command entry.
    fn get_command(&self) -> String {
        self.widgets.command_entry.get_text().to_string()
    }

    /// Go forward one character in the command entry.
    fn next_char(&self) {
        let pos = self.widgets.command_entry.get_position();
        let text = self.get_command();
        if pos < text.chars().count() as i32 {
            self.widgets.command_entry.set_position(pos + 1);
        }
    }

    /// Go forward one word in the command entry.
    fn next_word(&self) {
        let pos = self.widgets.command_entry.get_position();
        let text = self.get_command();
        let position = text.chars().enumerate()
            .skip(pos as usize)
            .skip_while(|&(_, c)| !c.is_alphanumeric())
            .skip_while(|&(_, c)| c.is_alphanumeric())
            .next()
            .map(|(index, _)| index)
            .unwrap_or_else(|| text.chars().count());
        self.widgets.command_entry.set_position(position as i32);
    }

    /// Paste the clipboard to the selection or the cursor position.
    fn paste(&self) {
        self.widgets.command_entry.paste_clipboard();
    }

    /// Paste the selection clipboard to the selection or the cursor position.
    fn paste_selection(&self) {
        self.delete_selection();
        let clipboard = Clipboard::get(&SELECTION_PRIMARY);
        if let Some(text) = clipboard.wait_for_text() {
            let mut position = self.widgets.command_entry.get_position();
            self.widgets.command_entry.insert_text(&text, &mut position);
            self.widgets.command_entry.set_position(position);
        }
    }

    /// Go back one character in the command entry.
    fn previous_char(&self) {
        let pos = self.widgets.command_entry.get_position();
        if pos > 0 {
            self.widgets.command_entry.set_position(pos - 1);
        }
    }

    /// Go back one word in the command entry.
    fn previous_word(&self) {
        let pos = self.widgets.command_entry.get_position();
        let text = self.get_command();
        let len = text.chars().count();
        let position = text.chars().rev().enumerate()
            .skip(len - pos as usize)
            .skip_while(|&(_, c)| !c.is_alphanumeric())
            .skip_while(|&(_, c)| c.is_alphanumeric())
            .next()
            .map(|(index, _)| len - index)
            .unwrap_or_default();
        self.widgets.command_entry.set_position(position as i32);
    }

    /// Seth the prefix identifier shown at the left of the command entry.
    fn set_identifier(&self, identifier: &str) {
        self.widgets.identifier_label.set_text(identifier);
    }

    /// Set the text of the input entry and move the cursor at the end.
    fn set_input(&self, command: &str) {
        // NOTE: Prevent updating the completions when the user selects a completion entry.
        let _lock = self.model.relm.stream().lock();
        self.widgets.command_entry.set_text(command);
        self.widgets.command_entry.set_position(command.chars().count() as i32);
    }

    /// Go to the beginning of the command entry.
    /// If the cursor is already at the beginning, go after the spaces after the command name.
    fn smart_home(&self) {
        let pos = self.widgets.command_entry.get_position();
        if pos == 0 {
            let text = self.get_command();
            let position = text.chars().enumerate()
                .skip_while(|&(_, c)| c.is_whitespace())
                .skip_while(|&(_, c)| !c.is_whitespace())
                .skip_while(|&(_, c)| c.is_whitespace())
                .map(|(index, _)| index)
                .next()
                .unwrap_or_default();
            self.widgets.command_entry.set_position(position as i32);
        }
        else {
            self.widgets.command_entry.set_position(0);
        }
    }
}

#[derive(Msg)]
pub enum ItemMsg {
    /// Set the color of the status bar item.
    Color(Option<RGBA>),
    /// Set the text of the status bar item.
    Text(String),
}

/// A status bar text item.
#[widget]
impl Widget for StatusBarItem {
    fn model() -> () {
        ()
    }

    fn update(&mut self, msg: ItemMsg) {
        match msg {
            Color(color) => self.widgets.label.override_color(StateFlags::NORMAL, color.as_ref()),
            Text(text) => self.widgets.label.set_text(&text), // TODO: use model field and a binding.
        }
    }

    view! {
        #[parent="status-bar-item"]
        #[name="label"]
        gtk::Label {
            ellipsize: EllipsizeMode::End,
            child: {
                pack_type: PackType::End,
                padding: 3,
            },
        }
    }
}
