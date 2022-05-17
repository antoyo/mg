/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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

use glib::object::{IsA, Object, ObjectExt};
use gdk::RGBA;
use gtk::{
    traits::{
        StyleContextExt,
        WidgetExt,
    },
    Settings,
    StateFlags,
    Widget,
};
use mg_settings::{self, EnumFromStr, EnumMetaData, SettingCompletion, SpecialCommand};

use app::Mg;

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Get the color of the text.
    pub fn get_foreground_color(&self) -> RGBA {
        let style_context = self.widgets.window.style_context();
        style_context.color(StateFlags::NORMAL)
    }

    /// Reset the background and foreground colors of the status bar.
    pub fn reset_colors(&self) {
        let style_context = self.widgets.status_bar.style_context();
        for class in style_context.list_classes() {
            style_context.remove_class(&class);
        }
    }

    /// Use the dark variant of the theme if available.
    pub fn set_dark_theme(&mut self, use_dark: bool) {
        let settings = Settings::default().unwrap();
        let _ = settings.set_property("gtk-application-prefer-dark-theme", &use_dark);
        self.model.foreground_color = self.get_foreground_color();
    }
}

/// Color the status bar in blue.
pub fn color_blue<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    widget.style_context().add_class("blue_background");
}

/// Color the status bar in orange.
pub fn color_orange<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    widget.style_context().add_class("orange_background");
}

/// Color the status bar in red.
pub fn color_red<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    widget.style_context().add_class("red_background");
}
