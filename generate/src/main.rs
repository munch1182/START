use colored::Colorize;
use libcommon::{
    curr_dir,
    ext::{FileDirCreateExt, PathJoinExt},
    newerr,
    prelude::Result,
};
use libloading::{Library, Symbol};
use plugin_d::{NAME_GET_INO, PluginInfo};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::RwLock,
};

static IS_DEBUG: RwLock<bool> = RwLock::new(false);

fn main() {
    let plugin_dir = curr_dir!("plugins").unwrap();
    let from_dir = curr_dir!().unwrap();
    let to_dir = curr_dir!("test_dir_scan").unwrap();

    let args = std::env::args().collect::<Vec<_>>();
    *IS_DEBUG.write().unwrap() =
        args.contains(&"--debug".to_string()) || args.contains(&"-d".to_string());

    if let Err(e) = run(plugin_dir, from_dir, to_dir) {
        eprintln!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}

fn run(plugin_dirs: PathBuf, from_dir: PathBuf, to_dir: PathBuf) -> Result<()> {
    let plugins = collect_plugins(&plugin_dirs)?;

    if plugins.is_empty() {
        println!("{}", "No plugins found".yellow());
        return Ok(());
    }

    display_plugins(&plugins);

    let index = read_plugin_index(plugins.len())?;
    let (name, dir) = &plugins[index];

    println!("{}", format!("Generating {}...", name).green());

    if generate_plugin(dir)? {
        println!("{}", "Copy plugin to target...".green());
        let info = copy_plugin_to_target(&from_dir, &to_dir, name)?;
        println!("{}", "Write plugin info...".green());
        write_info(info)?;
        println!("{}", "success".green());
    } else {
        println!("{}", "error".red());
    }

    Ok(())
}

fn write_info(info: (PluginInfo, PathBuf)) -> Result<()> {
    let (info, parent) = info;
    let info_path = parent.join("plugin_info.json");
    if is_debug() {
        println!("Write plugin info to {info_path:?}");
        println!("Plugin info: {info:?}");

        let index_html = parent.join("index.html");
        fs::write(
            index_html,
            r#"
<!DOCTYPE html>
<html lang="en">
<body>
    <span>hello world</span>
</body>
</html>
        "#,
        )?;
    }
    serde_json::to_writer(fs::File::create(info_path)?, &info)?;
    Ok(())
}

fn copy_plugin_to_target(
    from_dir: &Path,
    to_dir: &Path,
    plugin_name: &str,
) -> Result<(PluginInfo, PathBuf)> {
    let dll_name = format!("{}.dll", plugin_name);
    let source_path = from_dir.join_all(&["target", "release", &dll_name]);

    if !source_path.exists() {
        return Err(newerr!("Plugin DLL not found at {source_path:?}"));
    }

    let lib = unsafe { Library::new(&source_path) }?;
    let get_info: Symbol<'_, fn() -> PluginInfo> = unsafe { lib.get(NAME_GET_INO) }?;
    let info = get_info();

    let dll_name = &info.res.file;
    let target_path = to_dir.join_all(&[plugin_name, dll_name]).create_parent()?;

    if is_debug() {
        println!("Copy {source_path:?} to {target_path:?}");
    }

    fs::copy(source_path, &target_path)?;

    Ok((
        info,
        target_path
            .parent()
            .ok_or(newerr!("no parent"))?
            .to_path_buf(),
    ))
}

fn generate_plugin(plugin_dir: &Path) -> Result<bool> {
    if is_debug() {
        println!("Build plugin at {plugin_dir:?}");
    }
    let status = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(plugin_dir)
        .status()?;

    Ok(status.success())
}

fn read_plugin_index(max_index: usize) -> Result<usize> {
    loop {
        print!("{}", "Input index: ".green());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        if let Ok(index) = trimmed.parse::<usize>()
            && index < max_index
        {
            return Ok(index);
        }

        println!("{}", "Invalid index, please try again".red());
    }
}

fn display_plugins(plugins: &[(String, PathBuf)]) {
    println!("{}", format!("Plugins found: {}", plugins.len()).yellow());
    println!();

    for (i, (name, path)) in plugins.iter().enumerate() {
        println!(
            "[{}] {} {}",
            i.to_string().yellow(),
            name.yellow(),
            if is_debug() { path.to_str() } else { None }.unwrap_or_default()
        );
    }

    println!();
}

fn is_debug() -> bool {
    if let Ok(is_debug) = IS_DEBUG.read() {
        *is_debug
    } else {
        false
    }
}

fn collect_plugins(current_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut plugins = Vec::new();

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }
        let Some(file_name) = path.file_name() else {
            continue;
        };
        let name = file_name.to_string_lossy();
        if name.starts_with("plugin_") && name != "plugin_d" && name != "plugin_manager" {
            plugins.push((name.to_string(), path));
        }
    }

    Ok(plugins)
}
