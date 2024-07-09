use std::fs::{self, OpenOptions};
// use std::io::{prelude::*};

use zed_extension_api::{self as zed, LanguageServerId, Result};

fn log(msg: String) {
    println!("{}", msg);
    // let mut logfile = OpenOptions::new().write(true).append(true).open("/home/uwun/projects.local/zed-wakatime/ext.log").unwrap();
    // writeln!(logfile, "{}", msg).unwrap();
}

struct WakatimeExtension {
    cached_binary_path: Option<String>,
}

impl WakatimeExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        log(format!("searching for wakatime-lsp"));
        if let Some(path) = worktree.which("wakatime-lsp") {
            // !("found wakatime-lsp at {:?}", path);
            // write the above line to the logfile at ~/projects.local/zed-wakatime/ext.log
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        // Waiting on https://github.com/mrnossiom/wakatime-lsp/issues/2
        unimplemented!(
            "Use installation instructions on https://github.com/mrnossiom/wakatime-lsp"
        );

        let release = zed::latest_github_release(
            "mrnossiom/wakatime-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "wakatime-lsp-{arch}-{os}.tar.gz",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "pc-windows-msvc",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("wakatime-lsp-{}", release.version);
        let binary_path: String = todo!(); // format!("{version_dir}/wakatime-lsp-server");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for WakatimeExtension {
    fn new() -> Self {
        log(format!("new"));
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        log(format!("srvcmd @ server_id: {:?}", language_server_id));
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![],
            env: vec![
                ("SCLS_CONFIG_SUBDIRECTORY".to_owned(), "zed".to_owned()),
                ("RUST_LOG".to_owned(), "debug".to_owned()),
            ],
            // env: vec![],
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        log(format!("wkspconf @ server_id: {:?}", server_id));
        Ok(None)
        // Ok(Some(
        //     LspSettings::for_worktree("wakatime-lsp", worktree)
        //         .ok()
        //         .and_then(|settings| settings.settings.clone())
        //         .unwrap(),
        // ))
    }
}

zed::register_extension!(WakatimeExtension);
