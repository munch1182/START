mod pmimpl;
use libcommon::hash;
pub use pmimpl::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PluginId(String);

impl std::fmt::Debug for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PluginId {
    /// 将s原值作为插件id
    pub fn new_by(s: impl ToString) -> Self {
        Self(s.to_string().to_lowercase())
    }

    /// 将s的16进制hash值作为插件id
    pub fn new_from(s: impl ToString) -> Self {
        let id = format!("{:x}", hash!(s.to_string()));
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use libcommon::{
        log::log_setup,
        prelude::{Result, info},
    };

    #[test]
    fn test() -> Result<()> {
        log_setup();
        let id1 = PluginId::new_from("adb");
        let id2 = PluginId::new_by("15a16988a79de7ed");
        assert_eq!(&id1, &id2);
        assert_eq!(id1, id2);

        let mut map = HashMap::with_capacity(1);
        map.insert(id1.clone(), 1);

        let id3 = PluginId::new_from("adb");
        assert!(map.contains_key(&id3));
        info!("id1 {id1:?}, &id2: {:?}, id3: {id3:?}", &id2);
        Ok(())
    }
}
