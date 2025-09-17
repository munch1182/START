use colored::Colorize;
use libcommon::{
    ext::{FileDirCreateExt, PathJoinExt},
    newerr,
    prelude::*,
};
use libloading::{Library, Symbol};
use plugin_d::{PluginInfo, Res};
use std::{
    env::current_dir,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let current_dir = current_dir()?;
    let plugins = collect_plugins(&current_dir)?;

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
        let target = copy_plugin_to_target(&current_dir, name)?;
        println!("{}", "Write plugin info...".green());
        write_info(name, target)?;
        println!("{}", "success".green());
    } else {
        println!("{}", "error".red());
    }

    Ok(())
}

fn write_info(name: &str, target: PathBuf) -> Result<()> {
    let parent = &target.parent().ok_or(newerr!(""))?;

    let lib = unsafe { Library::new(&target) }?;
    let get_info: Symbol<'_, fn(Res) -> PluginInfo> = unsafe { lib.get(b"plugin_info") }?;

    let info = get_info(Res::new_in_dir(name));

    let info_path = parent.join("plugin_info.json");
    serde_json::to_writer(fs::File::create(info_path)?, &info)?;
    Ok(())
}

fn copy_plugin_to_target(current_dir: &Path, plugin_name: &str) -> Result<PathBuf> {
    let dll_name = format!("{}.dll", plugin_name);
    let source_path = current_dir.join_all(&["target", "release", &dll_name]);

    if !source_path.exists() {
        return Err(newerr!("Plugin DLL not found at {source_path:?}"));
    }

    let target_path = current_dir
        .join_all(&["test_scan_dir", plugin_name, &dll_name])
        .create_parent()?;

    fs::copy(source_path, &target_path)?;

    Ok(target_path)
}

fn generate_plugin(plugin_dir: &Path) -> Result<bool> {
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

    for (i, (name, _)) in plugins.iter().enumerate() {
        println!("[{}] {}", i.to_string().yellow(), name.yellow());
    }

    println!();
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
