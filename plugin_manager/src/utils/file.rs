use libcommon::prelude::Result;
use plugin_d::{PluginInfo, Res};
use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

/// 扫描文件夹下所有能够解析为PluginInfo的json文件
/// 并将json文件路径和解析后的PluginInfo返回
pub(crate) fn scan_plugin(path: impl AsRef<Path>) -> Vec<PluginInfo> {
    let jsons = scan_dir_find(path, |p| p.extension().unwrap_or_default() == "json");
    let mut res = vec![];
    for json in jsons {
        if let Ok(mut info) = PluginInfo::from_json(&json) {
            fix_res_dir(&mut info.res, &json);
            res.push(info);
        }
    }
    res
}

fn fix_res_dir(res: &mut Res, file: &OsStr) {
    // 如果路径已经是绝对路径，不需要修改
    if Path::new(&res.dir).is_absolute() {
        return;
    }

    let parent = Path::new(file).parent().unwrap_or_else(|| Path::new("."));
    if &res.dir == "." || res.dir.is_empty() {
        res.dir = parent.to_str().unwrap_or_default().to_string();
        return;
    }

    // 获取文件所在的目录
    let new_res_dir = parent.join(&res.dir);

    // 尝试规范化路径（解析相对路径等）
    let canonical_path = match new_res_dir.canonicalize() {
        Ok(path) => path,
        Err(_) => new_res_dir,
    };

    // 更新资源目录
    if let Some(path_str) = canonical_path.to_str() {
        res.dir = path_str.to_string();
    }
}

fn scan_dir_find<F>(path: impl AsRef<Path>, find: F) -> Vec<OsString>
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
