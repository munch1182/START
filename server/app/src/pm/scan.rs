use libcommon::{
    ext::FileFinderExt,
    newerr,
    prelude::{Result, info},
};
use plugin_d::PluginInfo;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn scan_info(path: impl AsRef<Path>) -> Result<Vec<(PathBuf, PluginInfo)>> {
    Ok(scan_json(path)?.iter().flat_map(read).collect())
}

fn read(f: &PathBuf) -> Result<(PathBuf, PluginInfo)> {
    let dir = f.parent().ok_or(newerr!("mut had parent"))?.to_path_buf();
    let info: PluginInfo = serde_json::from_reader(fs::File::open(f)?)?;
    info!("plugin info: {info:?} from {f:?}");
    Ok((dir, info))
}

/// 扫描该目录下(非.开头的目录)的所有json文件
fn scan_json(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    let find = |f: &Path| {
        if f.is_file() {
            f.extension().and_then(|x| x.to_str()) == Some("json") // 文件以.json结尾
        } else if f.is_dir() {
            f.file_name()
                .map(|x| !x.to_string_lossy().starts_with('.')) // 文件夹不以.开头
                .unwrap_or(false)
        } else {
            false
        }
    };
    Ok(path.find(false, true, find))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{curr_dir, ext::FileDirCreateExt, log_setup, prelude::debug, timer};
    use std::env::current_dir;

    #[test]
    fn test_scan() -> Result<()> {
        let dir = curr_dir!("test_scan_plugin")?;
        let target = [
            dir.join("plugin1.json"),
            dir.join("test1").join("plugin2.json").create_parent()?,
            dir.join(".cargo").join("plugin3.json").create_parent()?,
        ];
        for json in &target {
            let info = PluginInfo::new("testjson", "0.0.1", "testjson.dll", "testjson.html");
            fs::write(json, serde_json::to_string_pretty(&info)?)?;
        }

        let jsons = scan_json(&dir)?;
        assert_eq!(jsons.len(), target.len() - 1);

        let res = scan_info(&dir)?;
        assert_eq!(res.len(), target.len() - 1);

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[ignore = "need generate"]
    #[timer]
    #[test]
    fn test_generate() -> Result<()> {
        log_setup();
        let dir = current_dir()?;
        let dir = dir.parent().and_then(Path::parent).ok_or(newerr!(""))?;
        let dir = dir.join("test_scan_dir");

        let jsons = scan_json(&dir)?;
        info!("scan jsons: {jsons:?} from {dir:?}");
        for ele in jsons {
            let read = read(&ele);
            debug!("read {ele:?}: {:?}", read.is_ok());
        }

        Ok(())
    }
}
