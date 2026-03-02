use configparser::ini::Ini;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize)]
pub struct ProfileInfo {
    name: String,
    path: PathBuf,
    is_relative: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct BrowserProfiles {
    firefox: Vec<ProfileInfo>,
    floorp: Vec<ProfileInfo>,
    zen: Vec<ProfileInfo>,
}

fn parse_profiles(root_dir: Option<PathBuf>) -> Vec<ProfileInfo> {
    let mut results = Vec::new();
    let Some(root) = root_dir else {
        return results;
    };

    let ini_path = root.join("profiles.ini");
    if !ini_path.exists() {
        return results;
    }

    let mut config = Ini::new();
    if config.load(&ini_path).is_err() {
        return results;
    }

    // Profile0 から順に探す
    for i in 0..20 {
        let section = format!("Profile{}", i);

        let name = config.get(&section, "name");
        let path_str = config.get(&section, "path");
        let is_relative = config
            .get(&section, "isrelative")
            .map(|v| v == "1")
            .unwrap_or(false);

        if let (Some(n), Some(p)) = (name, path_str) {
            let full_path = if is_relative {
                root.join(p)
            } else {
                PathBuf::from(p)
            };

            results.push(ProfileInfo {
                name: n,
                path: full_path,
                is_relative,
            });
        }
    }
    results
}

#[tauri::command]
fn get_all_browser_profiles() -> BrowserProfiles {
    let app_data = dirs::data_dir();
    let home = dirs::home_dir();

    // OSごとのベースディレクトリ判定
    let get_root = |win: &str, mac: &str, linux: &str| {
        if cfg!(target_os = "windows") {
            app_data.as_ref().map(|p| p.join(win))
        } else if cfg!(target_os = "macos") {
            app_data.as_ref().map(|p| p.join(mac))
        } else {
            home.as_ref().map(|p| p.join(linux))
        }
    };

    BrowserProfiles {
        firefox: parse_profiles(get_root("Mozilla/Firefox", "Firefox", ".mozilla/firefox")),
        floorp: parse_profiles(get_root("Floorp", "Floorp", ".floorp")),
        zen: parse_profiles(get_root("zen", "ZenBrowser", ".zen")),
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_deno_path() -> Result<String, String> {
    match which::which("deno") {
        Ok(path) => Ok(path.display().to_string()),
        Err(_) => Err("deno not found".into()),
    }
}

#[tauri::command]
fn get_ffmpeg_path() -> Result<String, String> {
    match which::which("ffmpeg") {
        Ok(path) => Ok(path.display().to_string()),
        Err(_) => Err("ffmpeg not found".into()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    let _ = fix_path_env::fix();
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_deno_path,
            get_ffmpeg_path,
            get_all_browser_profiles
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
