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
 * TODO: set the size of the status bar according to the size of the font.
 * TODO: supports shortcuts like <C-a> and <C-w> in the command entry.
 * TODO: smart home (with Ctrl-A) in the command text entry.
 * TODO: support non-"always" in special commands.
 * TODO: different event for activate event of special commands.
 * TODO: use the gtk::Statusbar widget?
 * TODO: support shortcuts with number like "50G".
 * TODO: Associate a color with modes.
 */

//! Minimal UI library based on GTK+.

#![warn(missing_docs)]

extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
#[macro_use]
extern crate log;
extern crate mg_settings;
extern crate pango_sys;

#[macro_use]
mod gtk_timeout;
#[macro_use]
mod signal;
mod app;
pub mod completion;
mod gobject;
mod gtk_widgets;
mod key_converter;
mod scrolled_window;
mod style_context;

use std::char;
use std::result;

pub use app::{Application, ApplicationBuilder, SimpleApplicationBuilder};
pub use app::dialog::{DialogBuilder, DialogResult};
pub use app::settings::NoSettings;
pub use app::status_bar::StatusBarItem;

#[macro_export]
macro_rules! hash {
    ($($key:expr => $value:expr),* $(,)*) => {{
        let mut hashmap = ::std::collections::HashMap::new();
        $(hashmap.insert($key.into(), $value.into());)*
        hashmap
    }};
}

#[macro_export]
macro_rules! special_commands {
    ($enum_name:ident { $( $command:ident ( $identifier:expr , always ),)* } ) => {
        pub enum $enum_name {
            $( $command(String), )*
        }

        impl $crate::SpecialCommand for $enum_name {
            fn identifier_to_command(identifier: char, input: &str) -> ::std::result::Result<Self, String> {
                match identifier {
                    $( $identifier => Ok($enum_name::$command(input.to_string())), )*
                    _ => Err(format!("unknown identifier {}", identifier)),
                }
            }

            fn is_always(identifier: char) -> bool {
                match identifier {
                    $( $identifier )|* => true,
                    _ => false,
                }
            }

            fn is_identifier(character: char) -> bool {
                match character {
                    $( $identifier )|* => true,
                    _ => false,
                }
            }
        }
    };
}

/// Trait for converting an identifier like "/" to a special command.
pub trait SpecialCommand
    where Self: Sized
{
    /// Convert an identifier like "/" to a special command.
    fn identifier_to_command(identifier: char, input: &str) -> result::Result<Self, String>;

    /// Check if the identifier is declared `always`.
    /// The always option means that the command is activated every time a character is typed (like
    /// incremental search).
    fn is_always(identifier: char) -> bool;

    /// Check if a character is a special command identifier.
    fn is_identifier(character: char) -> bool;
}

// TODO: remove when special commands are merge with commands.
#[doc(hidden)]
pub struct NoSpecialCommands {
}

impl SpecialCommand for NoSpecialCommands {
    fn identifier_to_command(_identifier: char, _input: &str) -> result::Result<Self, String> {
        Err(String::new())
    }

    fn is_always(_identifier: char) -> bool {
        false
    }

    fn is_identifier(_character: char) -> bool {
        false
    }
}
