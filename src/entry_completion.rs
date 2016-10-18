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

use std::ffi::CStr;

use glib::translate::ToGlibPtr;
use glib::translate::FromGlibPtr;
use gtk::{EntryCompletion, TreeIter};
use gtk_sys::{GtkEntryCompletion, GtkTreeIter, gtk_entry_completion_set_match_func};
use libc::{c_char, c_void};

type MatchFuncCallback = Box<Fn(&EntryCompletion, &str, &TreeIter) -> bool + 'static>;

pub trait EntryCompletionExtManual {
    fn set_match_func<F: Fn(&Self, &str, &TreeIter) -> bool>(&self, func: F);
}

impl EntryCompletionExtManual for EntryCompletion {
    fn set_match_func<F: Fn(&Self, &str, &TreeIter) -> bool + 'static>(&self, func: F) {
        unsafe {
            let callback: Box<MatchFuncCallback> = Box::new(Box::new(func));
            let user_data: *mut c_void = Box::into_raw(callback) as *mut _;
            gtk_entry_completion_set_match_func(self.to_glib_full(), Some(set_match_func_trampoline), user_data, None);
        }
    }
}

unsafe extern "C" fn set_match_func_trampoline(this: *mut GtkEntryCompletion, key: *const c_char, iter: *mut GtkTreeIter, user_data: *mut c_void) -> i32 {
    let cstr = CStr::from_ptr(key);
    let f: &MatchFuncCallback = &*(user_data as *const _);
    f(&FromGlibPtr::from_glib_none(this), cstr.to_str().unwrap(), &FromGlibPtr::from_glib_none(iter)) as i32
}
