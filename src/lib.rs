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
 * TODO: <C-e>, <A-f>, <C-d>, <A-d>.
 *
 * TODO: change all unneeded &mut self to &self.
 * TODO: set the size of the status bar according to the size of the font.
 * TODO: different event for activate event of special commands.
 * TODO: use the gtk::Statusbar widget?
 * TODO: support shortcuts with number like "50G".
 * TODO: Associate a color with modes.
 */

//! Minimal UI library based on GTK+.

#![warn(missing_docs)]
#![feature(proc_macro)]

extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate libc;
#[macro_use]
extern crate log;
extern crate mg_settings;
extern crate pango;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

#[macro_use]
mod gtk_timeout;
mod app;
pub mod completion;
mod key_converter;

/// Map mode prefix (i.e. "i") to the mode name (i.e. "insert").
pub type Modes = &'static [(&'static str, &'static str)];
/// Map variable names to a function returning the value of this variable.
pub type Variables = &'static [(&'static str, fn() -> String)]; // TODO: probably needs to use a Box<Fn() -> String>.

pub use app::{Mg, parse_config};
pub use app::Msg::{CustomCommand, ModeChanged, SettingChanged};
//pub use app::dialog::{DialogBuilder, DialogResult};
pub use app::settings::NoSettings;
pub use app::status_bar::{StatusBar, StatusBarItem};
