use crate::WindowConfig;

pub enum UserEvent {
    Create(WindowConfig),
    Exit,
}

impl std::fmt::Display for UserEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UserEvent::Create(wc) => format!("UserEvent::Create({})", wc.title),
            UserEvent::Exit => String::from("UserEvent::Exit"),
        };
        write!(f, "{str}")
    }
}

impl std::fmt::Debug for UserEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
