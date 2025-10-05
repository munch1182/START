use crate::execute_callback;
use parking_lot::Mutex;
use rdev::{Event, EventType, Key};
use std::{
    sync::LazyLock,
    time::{Duration, SystemTime},
};

static KEY_HELPER: LazyLock<Mutex<KeyHelper>> = LazyLock::new(Default::default);

#[derive(Default)]
struct KeyHelper {
    press_count: usize,
    key_sequence: Vec<Key>,                       // 记录按键顺序
    current_keys: std::collections::HashSet<Key>, // 跟踪当前按下的键
    last_key: Option<(Key, SystemTime)>,          // 上一次按下的键
}

pub fn on_key(event: Event) {
    let event_type = event.event_type;
    if !matches!(
        event_type,
        EventType::KeyPress(_) | EventType::KeyRelease(_)
    ) {
        return;
    }

    let should_process = {
        let mut helper = KEY_HELPER.lock();
        helper.update_state(event_type, event.time)
    };

    if should_process {
        process_key_sequence();
    }
}

impl KeyHelper {
    fn update_state(&mut self, event_type: EventType, time: SystemTime) -> bool {
        match event_type {
            EventType::KeyPress(key) => {
                // 只有当键是第一次按下时才增加计数和记录
                if self.current_keys.insert(key) {
                    self.press_count += 1;
                    self.key_sequence.push(key);
                }
                false
            }
            EventType::KeyRelease(key) => {
                self.current_keys.remove(&key);
                self.press_count = self.press_count.saturating_sub(1);

                if let Some((lkey, ltime)) = self.last_key
                    && lkey == key
                    && time.duration_since(ltime).unwrap_or(Duration::ZERO)
                        < Duration::from_millis(500)
                {
                    self.take_key_sequence();
                    self.press_count = 0;
                    execute_callback(&format!("{}+{}", to_str(&key), to_str(&key)));
                    // 双击时，此次即处理
                    false
                } else {
                    self.last_key = Some((key, time));
                    // 当所有键都释放时处理序列
                    self.press_count == 0
                }
            }
            _ => false,
        }
    }

    fn take_key_sequence(&mut self) -> Vec<Key> {
        std::mem::take(&mut self.key_sequence)
    }
}

fn process_key_sequence() {
    let sequence = {
        let mut helper = KEY_HELPER.lock();
        helper.take_key_sequence()
    };

    if !sequence.is_empty() {
        let str = sequence.iter().map(to_str).collect::<Vec<_>>().join("+");
        execute_callback(&str);
    }
}

fn to_str(key: &Key) -> String {
    let key = match key {
        Key::ControlLeft | Key::ControlRight => Key::ControlLeft,
        Key::ShiftLeft | Key::ShiftRight => Key::ShiftLeft,
        key => *key,
    };
    format!("{key:?}")
}
