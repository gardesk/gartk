use gartk_core::{Key, KeyEvent, Modifiers};
use x11rb::protocol::xproto::{self, KeyButMask};

/// Convert X11 modifier mask to our Modifiers type
pub fn modifiers_from_x11(state: KeyButMask) -> Modifiers {
    Modifiers {
        shift: state.contains(KeyButMask::SHIFT),
        ctrl: state.contains(KeyButMask::CONTROL),
        alt: state.contains(KeyButMask::MOD1), // Alt is usually Mod1
        super_key: state.contains(KeyButMask::MOD4), // Super is usually Mod4
        caps_lock: state.contains(KeyButMask::LOCK),
        num_lock: state.contains(KeyButMask::MOD2), // NumLock is usually Mod2
    }
}

/// Convert X11 keycode to our Key type
/// This is a basic implementation using evdev keycodes
/// For proper keyboard layout support, use xkbcommon
pub fn key_from_keycode(keycode: u8, modifiers: &Modifiers) -> Key {
    // evdev keycodes (keycode = evdev + 8)
    match keycode {
        9 => Key::Escape,
        36 => Key::Return,
        23 => Key::Tab,
        22 => Key::Backspace,
        119 => Key::Delete,
        118 => Key::Insert,
        110 => Key::Home,
        115 => Key::End,
        112 => Key::PageUp,
        117 => Key::PageDown,
        113 => Key::Left,
        114 => Key::Right,
        111 => Key::Up,
        116 => Key::Down,
        67 => Key::F1,
        68 => Key::F2,
        69 => Key::F3,
        70 => Key::F4,
        71 => Key::F5,
        72 => Key::F6,
        73 => Key::F7,
        74 => Key::F8,
        75 => Key::F9,
        76 => Key::F10,
        95 => Key::F11,
        96 => Key::F12,
        65 => Key::Space,

        // Number row
        10 => char_key('1', '!', modifiers),
        11 => char_key('2', '@', modifiers),
        12 => char_key('3', '#', modifiers),
        13 => char_key('4', '$', modifiers),
        14 => char_key('5', '%', modifiers),
        15 => char_key('6', '^', modifiers),
        16 => char_key('7', '&', modifiers),
        17 => char_key('8', '*', modifiers),
        18 => char_key('9', '(', modifiers),
        19 => char_key('0', ')', modifiers),
        20 => char_key('-', '_', modifiers),
        21 => char_key('=', '+', modifiers),

        // QWERTY row
        24 => alpha_key('q', modifiers),
        25 => alpha_key('w', modifiers),
        26 => alpha_key('e', modifiers),
        27 => alpha_key('r', modifiers),
        28 => alpha_key('t', modifiers),
        29 => alpha_key('y', modifiers),
        30 => alpha_key('u', modifiers),
        31 => alpha_key('i', modifiers),
        32 => alpha_key('o', modifiers),
        33 => alpha_key('p', modifiers),
        34 => char_key('[', '{', modifiers),
        35 => char_key(']', '}', modifiers),

        // ASDF row
        38 => alpha_key('a', modifiers),
        39 => alpha_key('s', modifiers),
        40 => alpha_key('d', modifiers),
        41 => alpha_key('f', modifiers),
        42 => alpha_key('g', modifiers),
        43 => alpha_key('h', modifiers),
        44 => alpha_key('j', modifiers),
        45 => alpha_key('k', modifiers),
        46 => alpha_key('l', modifiers),
        47 => char_key(';', ':', modifiers),
        48 => char_key('\'', '"', modifiers),
        51 => char_key('\\', '|', modifiers),

        // ZXCV row
        52 => alpha_key('z', modifiers),
        53 => alpha_key('x', modifiers),
        54 => alpha_key('c', modifiers),
        55 => alpha_key('v', modifiers),
        56 => alpha_key('b', modifiers),
        57 => alpha_key('n', modifiers),
        58 => alpha_key('m', modifiers),
        59 => char_key(',', '<', modifiers),
        60 => char_key('.', '>', modifiers),
        61 => char_key('/', '?', modifiers),

        // Misc
        49 => char_key('`', '~', modifiers),

        _ => Key::Unknown(keycode),
    }
}

fn alpha_key(lower: char, modifiers: &Modifiers) -> Key {
    let upper = lower.to_ascii_uppercase();
    let shifted = modifiers.shift ^ modifiers.caps_lock;
    Key::Char(if shifted { upper } else { lower })
}

fn char_key(normal: char, shifted: char, modifiers: &Modifiers) -> Key {
    Key::Char(if modifiers.shift { shifted } else { normal })
}

/// Create a KeyEvent from X11 key press/release event
pub fn key_event_from_x11(event: &xproto::KeyPressEvent, pressed: bool) -> KeyEvent {
    let modifiers = modifiers_from_x11(event.state);
    let key = key_from_keycode(event.detail, &modifiers);

    KeyEvent {
        key,
        keycode: event.detail,
        modifiers,
        pressed,
    }
}
