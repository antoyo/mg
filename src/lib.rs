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

/*
 * FIXME: delete previous word does not remove enough characters when it contains multi-byte characters.
 * FIXME: deleting previous word does not update completions.
 * TODO: remove blocking dialogs?
 * FIXME: typing numbers in insert mode adds them to the shortcut label.
 * FIXME: should not be able to select completion items for input dialog without completer.
 * FIXME: can block in input mode.
 * TODO: use MainLoop::new() instead of gtk::main()?
 * FIXME: the commands in the config file are not executed.
 * TODO: take advantage of IntoOption and IntoPair.
 * TODO: automatically connect the AppClose message to Quit if it exists.
 * TODO: smart selection (select all on first time, select all except the prefix on the second).
 * TODO: shortcut to move the cursor at the other end of the selection.
 * TODO: thread example (using a oneshot).
 * TODO: generate async gtk methods as returning futures.
 * TODO: change all unneeded &mut self to &self.
 * TODO: set the size of the status bar according to the size of the font.
 * TODO: different event for activate event of special commands.
 * TODO: use the gtk::Statusbar widget?
 * TODO: Associate a color with modes.
 */

//! Minimal UI library based on GTK+.

#![feature(proc_macro, unboxed_closures)]
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

#[macro_use]
extern crate error_chain;
extern crate futures_glib;
extern crate gdk;
extern crate glib;
extern crate gtk;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mg_settings;
extern crate pango;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

mod app;
pub mod completion;
mod key_converter;

/// Map mode prefix (i.e. "i") to the mode name (i.e. "insert").
pub type Modes = &'static [(&'static str, &'static str)];
/// Map variable names to a function returning the value of this variable.

pub use app::{Mg, parse_config};
pub use app::Msg::{
    Alert,
    AppClose,
    CloseWin,
    Completers,
    CompletionViewChange,
    CustomCommand,
    CustomDialog,
    DarkTheme,
    DeleteCompletionItem,
    Error,
    Info,
    Message,
    ModeChanged,
    Question,
    SetMode,
    SetSetting,
    SettingChanged,
    Title,
    Variables,
    Warning,
};
pub use app::dialog::{
    BlockingInputDialog,
    DialogBuilder,
    DialogResult,
    InputDialog,
    Responder,
    blocking_dialog,
    blocking_input,
    blocking_question,
    blocking_yes_no_question,
    input,
    question,
    yes_no_question,
};
pub use app::settings::{DefaultConfig, NoSettings};
pub use app::status_bar::{StatusBar, StatusBarItem};
pub use app::status_bar::ItemMsg::Text;

#[macro_export]
macro_rules! hash {
    ($($ident:expr => $value:expr,)*) => {{
        let mut hash_map: ::mg::completion::Completers = ::std::collections::HashMap::new();
        $(let _ = hash_map.insert($ident, $value);)*
        hash_map
    }};
}

#[macro_export]
macro_rules! char_slice {
    ($( $val:expr ),*) => {{
        static SLICE: &'static [char] = &[$($val),*];
        SLICE
    }};
}

#[macro_export]
macro_rules! static_slice {
    ($( $val:expr ),* ; $typ:ty) => {{
        static SLICE: &'static [$typ] = &[$($val),*];
        SLICE
    }};
}
