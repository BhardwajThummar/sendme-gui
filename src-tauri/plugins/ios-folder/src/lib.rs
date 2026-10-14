//! Minimal iOS folder access for SendMe.
//!
//! iOS apps can only touch a folder outside their sandbox through the system
//! document picker, and only while holding a security-scoped URL. This plugin
//! lets the user pick a folder, keeps access to it across launches with a
//! bookmark, and copies files in and out of it (the Swift side does the
//! copying so the scoped access stays open for exactly as long as needed).

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_ios_folder);

/// A folder the user picked. `bookmark` is opaque and must be passed back to
/// the other calls; it stays valid across app launches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickedFolder {
    pub bookmark: String,
    pub name: String,
}

#[derive(Deserialize)]
struct PickResponse {
    folder: Option<PickedFolder>,
}

#[derive(Deserialize)]
struct PathResponse {
    path: String,
}

#[derive(Deserialize)]
struct ExistingResponse {
    existing: Vec<String>,
}

#[derive(Serialize)]
struct CopyOutArgs<'a> {
    bookmark: &'a str,
    dest: &'a str,
}

#[derive(Serialize)]
struct ListArgs<'a> {
    bookmark: &'a str,
    paths: &'a [String],
}

#[derive(Serialize)]
struct CopyInArgs<'a> {
    bookmark: &'a str,
    src: &'a str,
    overwrite: bool,
}

pub struct IosFolder<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> IosFolder<R> {
    /// Shows the system folder picker. `None` if the user cancelled.
    pub async fn pick(&self) -> Result<Option<PickedFolder>, String> {
        let r: PickResponse = self
            .0
            .run_mobile_plugin_async("pickFolder", ())
            .await
            .map_err(|e| e.to_string())?;
        Ok(r.folder)
    }

    /// Copies the picked folder into `dest` (as `dest/<folder name>`) and
    /// returns the path of the copy.
    pub async fn copy_out(&self, bookmark: &str, dest: &str) -> Result<String, String> {
        let r: PathResponse = self
            .0
            .run_mobile_plugin_async("copyOut", CopyOutArgs { bookmark, dest })
            .await
            .map_err(|e| e.to_string())?;
        Ok(r.path)
    }

    /// Which of the relative `paths` already exist inside the picked folder.
    pub async fn existing(&self, bookmark: &str, paths: &[String]) -> Result<Vec<String>, String> {
        let r: ExistingResponse = self
            .0
            .run_mobile_plugin_async("existing", ListArgs { bookmark, paths })
            .await
            .map_err(|e| e.to_string())?;
        Ok(r.existing)
    }

    /// Copies every file under `src` into the picked folder, keeping the
    /// relative layout. A taken name is overwritten or numbered (`a (1).txt`).
    pub async fn copy_in(&self, bookmark: &str, src: &str, overwrite: bool) -> Result<(), String> {
        let _: serde_json::Value = self
            .0
            .run_mobile_plugin_async(
                "copyIn",
                CopyInArgs {
                    bookmark,
                    src,
                    overwrite,
                },
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub trait IosFolderExt<R: Runtime> {
    fn ios_folder(&self) -> &IosFolder<R>;
}

impl<R: Runtime, T: Manager<R>> IosFolderExt<R> for T {
    fn ios_folder(&self) -> &IosFolder<R> {
        self.state::<IosFolder<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ios-folder")
        .setup(|app: &AppHandle<R>, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_ios_folder)?;
                app.manage(IosFolder(handle));
            }
            let _ = (app, api);
            Ok(())
        })
        .build()
}
