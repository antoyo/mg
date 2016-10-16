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

use gdk::enums::key::{self, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, Return, S, T, U, V, W, X, Y, Z, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z};
use mg_settings::key::Key::{self, Char, Control, Down, Enter, Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, Left, Minus, Plus, Right, Space, Tab, Up};

/// Convert a GDK key to an MG Key.
#[allow(non_upper_case_globals)]
pub fn gdk_key_to_key(key: key::Key, control_pressed: bool) -> Option<Key> {
    let key =
        match key {
            A => Char('A'),
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
            key::Down => Down,
            key::Escape => Escape,
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
            key::Left => Left,
            key::minus => Minus,
            Return => Enter,
            key::plus => Plus,
            key::Right => Right,
            key::space => Space,
            key::Tab => Tab,
            key::Up => Up,
            _ => return None,
        };
    if control_pressed {
        Some(Control(Box::new(key)))
    }
    else {
        Some(key)
    }
}
