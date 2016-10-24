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

use glib::translate::{ToGlib, ToGlibPtr};
use gtk::ScrolledWindow;

pub trait ScrolledWindowExtManual {
    fn set_max_content_height(&self, height: i32);
    fn set_propagate_natural_height(&self, propagate: bool);
}

impl ScrolledWindowExtManual for ScrolledWindow {
    fn set_max_content_height(&self, height: i32) {
        unsafe {
            ffi::gtk_scrolled_window_set_max_content_height(self.to_glib_none().0, height);
        }
    }

    fn set_propagate_natural_height(&self, propagate: bool) {
        unsafe {
            ffi::gtk_scrolled_window_set_propagate_natural_height(self.to_glib_none().0, propagate.to_glib());
        }
    }
}

mod ffi {
    use glib_sys::gboolean;
    use gtk_sys::GtkScrolledWindow;
    use libc::c_int;

    extern "C" {
        pub fn gtk_scrolled_window_set_max_content_height(scrolled_window: *mut GtkScrolledWindow, height: c_int);
        pub fn gtk_scrolled_window_set_propagate_natural_height(scrolled_window: *mut GtkScrolledWindow, propagate: gboolean);
    }
}
