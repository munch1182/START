use libcommon::prelude::Result;
use std::{ffi::OsString, path::Path};

pub(crate) fn scan_dir_find<F>(path: impl AsRef<Path>, find: F) -> Vec<OsString>
where
    F: Fn(&Path) -> bool,
{
    _scan_dir_dindl_impl(path.as_ref(), find)
}

fn _scan_dir_dindl_impl<F>(path: &Path, find: F) -> Vec<OsString>
where
    F: Fn(&Path) -> bool,
{
    if !path.exists() || !path.is_dir() {
        return vec![];
    }
    let mut dlls = vec![];
    let _ = _get_dll_from_dir_impl(path, &mut dlls, &find);
    dlls
}

fn _get_dll_from_dir_impl<F>(path: &Path, res: &mut Vec<OsString>, find: &F) -> Result<()>
where
    F: Fn(&Path) -> bool,
{
    if path.is_file() {
        if find(path) {
            res.push(path.as_os_str().to_os_string());
        }
    } else if path.is_dir() {
        for entry in path.read_dir()? {
            let path = entry?.path();
            let _ = _get_dll_from_dir_impl(&path, res, find);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{curr_dir, log::log_setup};
    use std::fs;

    #[test]
    fn test() -> Result<()> {
        log_setup();
        let dir = curr_dir!("test_tmp_dir")?;
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;

        let finder = |p: &Path| p.extension().unwrap_or_default() == "dll";

        let res = scan_dir_find(&dir, finder);
        assert!(res.is_empty());
        fs::write(&dir.join("a.dll"), b"1")?;
        fs::write(dir.join("b.dll"), b"2")?;
        let res = scan_dir_find(&dir, finder);
        assert_eq!(res.len(), 2);

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
