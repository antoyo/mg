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

use gdk::{EventKey, CONTROL_MASK, MOD1_MASK};
use gdk::enums::key::{self, A,
    _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, B, C, D, E, F, G, H, I, J, K, KP_0, KP_1, KP_2, KP_3,
    KP_4, KP_5, KP_6, KP_7, KP_8, KP_9, L, M, N, O, P, Q, R, Return, S, T, U, V, W, X, Y, Z, a, b,
    c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z};
use mg_settings::key::Key::{self, Alt, Backspace, Char, Control, Down, Enter, Escape, F1, F2, F3, F4, F5, F6, F7, F8,
    F9, F10, F11, F12, Left, Right, Shift, Space, Tab, Up};

/// Convert a GDK key to an MG Key.
#[allow(non_upper_case_globals)]
pub fn gdk_key_to_key(key: &EventKey) -> Option<Key> {
    let alt_pressed = key.get_state() & MOD1_MASK == MOD1_MASK;
    let control_pressed = key.get_state() & CONTROL_MASK == CONTROL_MASK;
    let key =
        match key.get_keyval() {
            A => Char('A'),
            _0 | KP_0 => Char('0'),
            _1 | KP_1 => Char('1'),
            _2 | KP_2 => Char('2'),
            _3 | KP_3 => Char('3'),
            _4 | KP_4 => Char('4'),
            _5 | KP_5 => Char('5'),
            _6 | KP_6 => Char('6'),
            _7 | KP_7 => Char('7'),
            _8 | KP_8 => Char('8'),
            _9 | KP_9 => Char('9'),
            B => Char('B'),
            C => Char('C'),
            D => Char('D'),
            E => Char('E'),
            F => Char('F'),
            G => Char('G'),
            H => Char('H'),
            I => Char('I'),
            J => Char('J'),
            K => Char('K'),
            L => Char('L'),
            M => Char('M'),
            N => Char('N'),
            O => Char('O'),
            P => Char('P'),
            Q => Char('Q'),
            R => Char('R'),
            S => Char('S'),
            T => Char('T'),
            U => Char('U'),
            V => Char('V'),
            W => Char('W'),
            X => Char('X'),
            Y => Char('Y'),
            Z => Char('Z'),
            a => Char('a'),
            b => Char('b'),
            c => Char('c'),
            d => Char('d'),
            e => Char('e'),
            f => Char('f'),
            g => Char('g'),
            h => Char('h'),
            i => Char('i'),
            j => Char('j'),
            k => Char('k'),
            l => Char('l'),
            m => Char('m'),
            n => Char('n'),
            o => Char('o'),
            p => Char('p'),
            q => Char('q'),
            r => Char('r'),
            s => Char('s'),
            t => Char('t'),
            u => Char('u'),
            v => Char('v'),
            w => Char('w'),
            x => Char('x'),
            y => Char('y'),
            z => Char('z'),
            key::agrave => Char('à'),
            key::Agrave => Char('À'),
            key::asciicircum => Char('^'),
            key::asciitilde => Char('~'),
            key::ampersand => Char('&'),
            key::at => Char('@'),
            key::backslash => Char('\\'),
            key::BackSpace => Backspace,
            key::bar => Char('|'),
            key::braceleft => Char('{'),
            key::braceright => Char('}'),
            key::bracketleft => Char('['),
            key::bracketright => Char(']'),
            key::ccedilla => Char('ç'),
            key::Ccedilla => Char('Ç'),
            key::comma => Char(','),
            key::dollar => Char('$'),
            key::Down => Down,
            key::eacute => Char('é'),
            key::Eacute => Char('É'),
            key::egrave => Char('è'),
            key::Egrave => Char('È'),
            key::equal => Char('='),
            key::Escape => Escape,
            key::exclam => Char('!'),
            key::F1 => F1,
            key::F2 => F2,
            key::F3 => F3,
            key::F4 => F4,
            key::F5 => F5,
            key::F6 => F6,
            key::F7 => F7,
            key::F8 => F8,
            key::F9 => F9,
            key::F10 => F10,
            key::F11 => F11,
            key::F12 => F12,
            key::ISO_Left_Tab => Shift(Box::new(Tab)),
            key::Left => Left,
            key::leftanglebracket => Char('<'),
            key::minus => Char('-'),
            key::multiply => Char('*'),
            key::numbersign => Char('#'),
            key::parenleft => Char('('),
            key::parenright => Char(')'),
            key::percent => Char('%'),
            key::period => Char('.'),
            key::plus => Char('+'),
            Return => Enter,
            key::Right => Right,
            key::rightanglebracket => Char('>'),
            key::question => Char('?'),
            key::quotedbl => Char('"'),
            key::semicolon => Char(';'),
            key::slash => Char('/'),
            key::space => Space,
            key::Tab => Tab,
            key::underscore => Char('_'),
            key::Up => Up,
            _ => return None,
        };
    let key =
        if control_pressed {
            if alt_pressed {
                Control(Box::new(Alt(Box::new(key))))
            }
            else {
                Control(Box::new(key))
            }
        }
        else if alt_pressed {
            Alt(Box::new(key))
        }
        else {
            key
        };
    Some(key)
}
