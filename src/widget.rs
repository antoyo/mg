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

/// Implement the IsA<Widget> trait for `$widget` where `$inner_widget` is a `Widget`.
macro_rules! is_widget {
    ($widget:ident, $inner_widget:ident) => {
        impl<'a> ::glib::translate::ToGlibPtr<'a, *const ::gtk_sys::GtkWidget> for $widget {
            type Storage = <::glib::object::ObjectRef as
                ::glib::translate::ToGlibPtr<'a, *mut ::glib::object::GObject>>::Storage;

            #[inline]
            fn to_glib_none(&'a self) -> ::glib::translate::Stash<'a, *const ::gtk_sys::GtkWidget, Self> {
                let stash: ::glib::translate::Stash<'a, *const _, _> = self.$inner_widget.to_glib_none();
                ::glib::translate::Stash(stash.0 as *const _, stash.1)
            }

            #[inline]
            fn to_glib_full(&self) -> *const ::gtk_sys::GtkWidget {
                let widget: *const _ = self.$inner_widget.to_glib_full();
                widget as *const _
            }
        }

        impl<'a> ::glib::translate::ToGlibPtr<'a, *mut ::gtk_sys::GtkWidget> for $widget {
            type Storage = <::glib::object::ObjectRef as
                ::glib::translate::ToGlibPtr<'a, *mut ::glib::object::GObject>>::Storage;

            #[inline]
            fn to_glib_none(&'a self) -> ::glib::translate::Stash<'a, *mut ::gtk_sys::GtkWidget, Self> {
                let stash: ::glib::translate::Stash<'a, *mut _, _> = self.$inner_widget.to_glib_none();
                ::glib::translate::Stash(stash.0 as *mut _, stash.1)
            }

            #[inline]
            fn to_glib_full(&self) -> *mut ::gtk_sys::GtkWidget {
                let widget: *mut _ = self.$inner_widget.to_glib_full();
                widget as *mut _
            }
        }

        impl Into<::glib::object::ObjectRef> for $widget {
            fn into(self) -> ::glib::object::ObjectRef {
                self.$inner_widget.into()
            }
        }

        impl ::glib::types::StaticType for $widget {
            fn static_type() -> ::glib::types::Type {
                unsafe { ::glib::translate::from_glib(::gtk_sys::gtk_box_get_type()) }
            }
        }

        impl ::glib::wrapper::UnsafeFrom<::glib::object::ObjectRef> for $widget {
            unsafe fn from(_t: ::glib::object::ObjectRef) -> Self {
                unimplemented!()
            }
        }

        impl ::glib::wrapper::Wrapper for $widget {
            type GlibType = ::gtk_sys::GtkWidget;
        }

        impl ::gtk::IsA<::gtk::Widget> for $widget {}
    };
}
