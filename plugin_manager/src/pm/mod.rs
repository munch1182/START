use std::ffi::OsString;

use crate::utils::file::scan_dir_find;

#[derive(Clone)]
pub(crate) struct PM {
    scan_dir: OsString,
}

impl PM {
    pub fn new(scan_dir: impl Into<OsString>) -> Self {
        let scan_dir = scan_dir.into();
        Self { scan_dir }
    }

    pub fn update_dir(&mut self, new_dir: impl Into<OsString>) {
        self.scan_dir = new_dir.into();
        self.scan();
    }

    pub fn scan(&self) {
        let files = scan_dir_find(&self.scan_dir, |f| {
            f.extension().unwrap_or_default() == "dll"
        });
        if files.is_empty() {
            return;
        }
    }
}
