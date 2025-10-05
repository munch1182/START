use key::{Event, EventType, Key, on_key};
use std::time::SystemTime;
use tao::{
    event::{ElementState, RawKeyEvent},
    keyboard::KeyCode,
};

/// 将tao的WindowEvent::KeyboardInput转换为rdev的Event
pub(crate) fn call_on_key_from_window(event: &RawKeyEvent) {
    let key = convert_virtual_keycode(event.physical_key);

    let time = SystemTime::now();
    let event_type = match event.state {
        ElementState::Pressed => EventType::KeyPress(key),
        ElementState::Released => EventType::KeyRelease(key),
        _ => return,
    };

    let event = Event {
        time,
        name: None,
        event_type,
    };
    on_key(event);
}

/// 将tao的VirtualKeyCode转换为rdev的Key
fn convert_virtual_keycode(vk: KeyCode) -> Key {
    match vk {
        KeyCode::KeyA => Key::KeyA,
        KeyCode::KeyB => Key::KeyB,
        KeyCode::KeyC => Key::KeyC,
        KeyCode::KeyD => Key::KeyD,
        KeyCode::KeyE => Key::KeyE,
        KeyCode::KeyF => Key::KeyF,
        KeyCode::KeyG => Key::KeyG,
        KeyCode::KeyH => Key::KeyH,
        KeyCode::KeyI => Key::KeyI,
        KeyCode::KeyJ => Key::KeyJ,
        KeyCode::KeyK => Key::KeyK,
        KeyCode::KeyL => Key::KeyL,
        KeyCode::KeyM => Key::KeyM,
        KeyCode::KeyN => Key::KeyN,
        KeyCode::KeyO => Key::KeyO,
        KeyCode::KeyP => Key::KeyP,
        KeyCode::KeyQ => Key::KeyQ,
        KeyCode::KeyR => Key::KeyR,
        KeyCode::KeyS => Key::KeyS,
        KeyCode::KeyT => Key::KeyT,
        KeyCode::KeyU => Key::KeyU,
        KeyCode::KeyV => Key::KeyV,
        KeyCode::KeyW => Key::KeyW,
        KeyCode::KeyX => Key::KeyX,
        KeyCode::KeyY => Key::KeyY,
        KeyCode::KeyZ => Key::KeyZ,

        // 数字键
        KeyCode::Numpad0 => Key::Num0,
        KeyCode::Numpad1 => Key::Num1,
        KeyCode::Numpad2 => Key::Num2,
        KeyCode::Numpad3 => Key::Num3,
        KeyCode::Numpad4 => Key::Num4,
        KeyCode::Numpad5 => Key::Num5,
        KeyCode::Numpad6 => Key::Num6,
        KeyCode::Numpad7 => Key::Num7,
        KeyCode::Numpad8 => Key::Num8,
        KeyCode::Numpad9 => Key::Num9,

        // 功能键
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,

        // 修饰键
        KeyCode::ShiftLeft => Key::ShiftLeft,
        KeyCode::ShiftRight => Key::ShiftRight,
        KeyCode::ControlLeft => Key::ControlLeft,
        KeyCode::ControlRight => Key::ControlRight,
        KeyCode::AltLeft => Key::Alt,
        KeyCode::AltRight => Key::AltGr,

        // 导航键
        KeyCode::ArrowLeft => Key::LeftArrow,
        KeyCode::ArrowRight => Key::RightArrow,
        KeyCode::ArrowUp => Key::UpArrow,
        KeyCode::ArrowDown => Key::DownArrow,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,

        // 编辑键
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        // KeyCode::Resume => Key::Return,
        KeyCode::Escape => Key::Escape,
        KeyCode::Space => Key::Space,
        KeyCode::Tab => Key::Tab,

        // 符号键
        KeyCode::Backquote => Key::BackQuote,
        KeyCode::Minus => Key::Minus,
        KeyCode::Equal => Key::Equal,
        KeyCode::BracketLeft => Key::LeftBracket,
        KeyCode::BracketRight => Key::RightBracket,
        KeyCode::Backslash => Key::BackSlash,
        KeyCode::Semicolon => Key::SemiColon,
        KeyCode::Quote => Key::Quote,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Dot,
        KeyCode::Slash => Key::Slash,

        // 小键盘
        // KeyCode::Numpad0 => Key::Kp0,
        // KeyCode::Numpad1 => Key::Kp1,
        // KeyCode::Numpad2 => Key::Kp2,
        // KeyCode::Numpad3 => Key::Kp3,
        // KeyCode::Numpad4 => Key::Kp4,
        // KeyCode::Numpad5 => Key::Kp5,
        // KeyCode::Numpad6 => Key::Kp6,
        // KeyCode::Numpad7 => Key::Kp7,
        // KeyCode::Numpad8 => Key::Kp8,
        // KeyCode::Numpad9 => Key::Kp9,
        // KeyCode::NumpadAdd => Key::KpPlus,
        // KeyCode::NumpadSubtract => Key::KpMinus,
        // KeyCode::NumpadMultiply => Key::KpMultiply,
        // KeyCode::NumpadDivide => Key::KpDivide,
        // // KeyCode::NumpadDecimal => Key::KpDecimal,
        // KeyCode::NumpadEnter => Key::KpReturn,

        // 其他键
        // KeyCode::CapsLock => Key::CapsLock,
        // KeyCode::Scroll => Key::ScrollLock,
        // KeyCode::NumLock => Key::NumLock,
        // KeyCode::Print => Key::PrintScreen,
        // KeyCode::Pause => Key::Pause,

        // 如果遇到未映射的键，使用Unknown
        _ => Key::Unknown(0),
    }
}
