use libcommon::prelude::{Result, info};
use std::{
    cell::RefCell,
    collections::HashMap,
    time::{Duration, Instant},
};
pub use tao::{event::ElementState, keyboard::KeyCode};

pub type KeyEle = (KeyCode, ElementState);
type KeyHandle = Box<dyn Fn(&str)>;

#[derive(Default)]
pub struct KeyHelper {
    combo_handle: RefCell<HashMap<String, KeyHandle>>,
    combo_detector: RefCell<KeyComboDetector>,
    last_key: RefCell<Option<KeyEle>>,
}

#[derive(Debug, Clone)]
struct KeyEvent {
    key: KeyCode,
    state: ElementState,
    time: Instant,
}

struct KeyComboDetector {
    pressed_keys: HashMap<String, Instant>,
    event_history: Vec<KeyEvent>,
    double_click_keys: Vec<KeyCode>,
    double_click_timeout: Duration,
    double_click_key: (Option<KeyEvent>, Option<KeyEvent>),
}

impl Default for KeyComboDetector {
    fn default() -> Self {
        Self {
            pressed_keys: Default::default(),
            event_history: Default::default(),
            double_click_keys: vec![KeyCode::ControlLeft],
            double_click_timeout: Duration::from_millis(300),
            double_click_key: Default::default(),
        }
    }
}

impl Drop for KeyHelper {
    fn drop(&mut self) {
        self.combo_handle.borrow_mut().clear();
    }
}

pub trait KeyListenerExt: Sized {
    fn check(self, key: String) -> bool;
    fn register_key_listener(self, key: String, lis: impl Fn(&str) + 'static) -> Result<Self>;
    fn unregister_key_listener(self, key: &str) -> Result<Self>;
}

impl KeyHelper {
    pub(crate) fn on_key(&self, key: KeyCode, state: ElementState) {
        {
            let mut last_key = self.last_key.borrow_mut();
            let new_key = Some((key, state));
            if *last_key == new_key {
                // 将重复的按键过滤掉
                return;
            }
            *last_key = new_key;
        }
        {
            if let Some(combo) = self.combo_detector.borrow_mut().on_key(key, state) {
                info!("key combo detected: {combo}");
                for ele in self.combo_handle.borrow().values() {
                    ele(&combo);
                }
            }
        }
    }
}

impl KeyListenerExt for &KeyHelper {
    fn check(self, key: String) -> bool {
        self.combo_handle.borrow().contains_key(&key)
    }

    fn register_key_listener(self, key: String, lis: impl Fn(&str) + 'static) -> Result<Self> {
        self.combo_handle.borrow_mut().insert(key, Box::new(lis));
        Ok(self)
    }

    fn unregister_key_listener(self, key: &str) -> Result<Self> {
        self.combo_handle.borrow_mut().remove(key);
        Ok(self)
    }
}

impl KeyComboDetector {
    pub(crate) fn on_key(&mut self, key: KeyCode, state: ElementState) -> Option<String> {
        let time = Instant::now();
        let event = KeyEvent { key, state, time };
        match state {
            ElementState::Pressed => {
                self.pressed_keys.insert(key.to_string(), time);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&key.to_string());
                if self.double_click_keys.contains(&key) {
                    if let Some(k0) = &self.double_click_key.0
                        && time.duration_since(k0.time) <= self.double_click_timeout
                    {
                        self.double_click_key.1 = Some(event.clone());
                    } else {
                        self.double_click_key = (Some(event.clone()), None);
                    }
                } else {
                    self.double_click_key = (None, None)
                }
            }
            _ => {}
        }
        self.event_history.push(event);
        self.detect_combinations()
    }

    fn detect_combinations(&mut self) -> Option<String> {
        if !self.pressed_keys.is_empty() {
            return None;
        }
        let his = match &self.double_click_key {
            (Some(key1), Some(key2)) => {
                format!("{}+{}", key1.key_str(), key2.key_str())
            }
            _ => self
                .event_history
                .iter()
                .filter(|e| e.state == ElementState::Pressed)
                .map(|x| x.key_str())
                .collect::<Vec<_>>()
                .join("+"),
        };
        self.event_history.clear();
        Some(his)
    }
}

impl KeyEvent {
    pub fn key_str(&self) -> String {
        match self.key {
            KeyCode::AltLeft | KeyCode::AltRight => "Alt".to_string(),
            KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl".to_string(),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift".to_string(),
            _ => self.key.to_string().replace("Key", "").replace("Digit", ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn test_key() {
        let mut d = KeyComboDetector::default();
        let key = vec![
            d.on_key(KeyCode::ControlLeft, ElementState::Pressed),
            d.on_key(KeyCode::KeyA, ElementState::Pressed),
            d.on_key(KeyCode::KeyA, ElementState::Released),
            d.on_key(KeyCode::ControlLeft, ElementState::Released),
        ];
        let key = key
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        println!("key: {key:?}");
        assert!(key.len() == 1);

        let key = vec![
            d.on_key(KeyCode::ControlLeft, ElementState::Pressed),
            d.on_key(KeyCode::KeyA, ElementState::Pressed),
            d.on_key(KeyCode::KeyC, ElementState::Pressed),
            d.on_key(KeyCode::KeyC, ElementState::Released),
            d.on_key(KeyCode::KeyA, ElementState::Released),
            d.on_key(KeyCode::ControlLeft, ElementState::Released),
        ];
        let key = key
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        println!("key: {key:?}");
        assert!(
            key.first()
                .map(|x| x.split("+").count() == 3)
                .unwrap_or(false)
        );

        thread::sleep(Duration::from_secs(1));

        let key = vec![
            d.on_key(KeyCode::ControlLeft, ElementState::Pressed),
            d.on_key(KeyCode::ControlLeft, ElementState::Released),
            d.on_key(KeyCode::ControlLeft, ElementState::Pressed),
            d.on_key(KeyCode::ControlLeft, ElementState::Released),
        ];
        let key = key
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        println!("key: {key:?}");
        assert!(key.len() == 2);
    }
}
