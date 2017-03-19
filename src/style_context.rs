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

use gdk::RGBA;
use glib::translate::{ToGlibPtr, ToGlibPtrMut};
use gtk::{StateFlags, StyleContext};
use gtk_sys::{GtkStateFlags, GtkStyleContext, gtk_style_context_get_background_color, gtk_style_context_get_color};

pub trait StyleContextExtManual {
    fn get_background_color(&self, state: StateFlags) -> RGBA;
    fn get_color(&self, state: StateFlags) -> RGBA;
}

impl StyleContextExtManual for StyleContext {
    fn get_background_color(&self, state: StateFlags) -> RGBA {
        let context: *mut GtkStyleContext = self.to_glib_full();
        let state = GtkStateFlags::from_bits(state.bits()).unwrap();
        let mut color = RGBA {
            alpha: 0.0,
            blue: 0.0,
            green: 0.0,
            red: 0.0,
        };
        unsafe { gtk_style_context_get_background_color(context, state, color.to_glib_none_mut().0) };
        color
    }

    fn get_color(&self, state: StateFlags) -> RGBA {
        let context: *mut GtkStyleContext = self.to_glib_full();
        let state = GtkStateFlags::from_bits(state.bits()).unwrap();
        let mut color = RGBA {
            alpha: 0.0,
            blue: 0.0,
            green: 0.0,
            red: 0.0,
        };
        unsafe { gtk_style_context_get_color(context, state, color.to_glib_none_mut().0) };
        color
    }
}
