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

use std::ffi::CString;
use std::ptr::null_mut;

use glib::{IsA, Object, ObjectExt};
use gobject_sys::{GObject, g_object_set};
use libc::c_void;
use pango_sys::PangoEllipsizeMode;

pub trait ObjectExtManual {
    fn set_data(&self, key: &str, data: i32);
    fn set_ellipsize_data(&self, key: &str, data: PangoEllipsizeMode);
}

impl<O: ObjectExt + IsA<Object>> ObjectExtManual for O {
    fn set_data(&self, key: &str, data: i32) {
        let object: *mut GObject = self.to_glib_full();
        let key = CString::new(key).unwrap();
        unsafe { g_object_set(object as *mut _, key.as_ptr(), data, c_null()) };
    }

    fn set_ellipsize_data(&self, key: &str, data: PangoEllipsizeMode) {
        let object: *mut GObject = self.to_glib_full();
        let key = CString::new(key).unwrap();
        unsafe { g_object_set(object as *mut _, key.as_ptr(), data, c_null()) };
    }
}

fn c_null() -> *mut c_void {
    null_mut()
}
