use zed_extension_api as zed;

struct ClaudeConfigExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for ClaudeConfigExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command, String> {
        // Check for user-configured binary path in LSP settings
        let settings = zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map_err(|e| format!("failed to get LSP settings: {e}"))?;

        if let Some(binary) = settings.binary.as_ref() {
            if let Some(path) = binary.path.as_ref() {
                return Ok(zed::Command {
                    command: path.clone(),
                    args: Default::default(),
                    env: Default::default(),
                });
            }
        }

        // Use cached path if available
        if let Some(path) = &self.cached_binary_path {
            return Ok(zed::Command {
                command: path.clone(),
                args: Default::default(),
                env: Default::default(),
            });
        }

        // Download from GitHub releases
        let release = zed::latest_github_release(
            "simonw/zed-claude-config",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("failed to fetch latest release: {e}"))?;

        let (platform, arch) = zed::current_platform();

        let os = match platform {
            zed::Os::Mac => "apple-darwin",
            zed::Os::Linux => "unknown-linux-gnu",
            zed::Os::Windows => return Err("Windows is not supported".to_string()),
        };

        let arch = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => return Err("x86 (32-bit) is not supported".to_string()),
        };

        let asset_name = format!("claude-settings-lsp-{arch}-{os}.tar.gz");

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {asset_name}"))?;

        let version_dir = format!("claude-settings-lsp-{}", release.version);
        let binary_path = format!("{version_dir}/claude-settings-lsp");

        if !std::fs::metadata(&binary_path).is_ok() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("failed to make binary executable: {e}"))?;

            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
        }

        self.cached_binary_path = Some(binary_path.clone());

        Ok(zed::Command {
            command: binary_path,
            args: Default::default(),
            env: Default::default(),
        })
    }
}

zed::register_extension!(ClaudeConfigExtension);
