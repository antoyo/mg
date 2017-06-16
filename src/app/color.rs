/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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
use gtk::{IsA, Object, Widget, WidgetExt, STATE_FLAG_NORMAL};

/// Color the status bar in blue.
pub fn color_blue<W: IsA<Object> + IsA<Widget>>(widget: &W) {
    widget.override_background_color(STATE_FLAG_NORMAL, &RGBA::blue());
    white_foreground(widget);
}

/// Color the status bar in orange.
pub fn color_orange<W: IsA<Object> + IsA<Widget>>(widget: &W) {
    widget.override_background_color(STATE_FLAG_NORMAL, &RGBA {
        red: 0.9,
        green: 0.55,
        blue: 0.0,
        alpha: 1.0 ,
    });
    white_foreground(widget);
}

/// Color the status bar in red.
pub fn color_red<W: IsA<Object> + IsA<Widget>>(widget: &W) {
    widget.override_background_color(STATE_FLAG_NORMAL, &RGBA::red());
    white_foreground(widget);
}

/// Set the foreground (text) color to white.
fn white_foreground<W: IsA<Object> + IsA<Widget>>(widget: &W) {
    widget.override_color(STATE_FLAG_NORMAL, &RGBA::white());
}
