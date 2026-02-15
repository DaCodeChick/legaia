//! First-run setup system for asset extraction
//!
//! This module handles the one-time extraction of assets from the PSX disc
//! when the user first launches the game.

use bevy::prelude::*;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Setup state for first-run extraction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SetupState {
    /// Check if setup is needed
    #[default]
    CheckSetup,
    /// Prompt user for disc location
    PromptDiscPath,
    /// Validate disc and show confirmation
    ValidateDisc,
    /// Extract and convert assets
    Extracting,
    /// Setup complete, proceed to main game
    Complete,
}

/// Resource tracking setup progress
#[derive(Resource, Default)]
pub struct SetupProgress {
    /// Path to PSX disc image
    pub disc_path: Option<PathBuf>,
    /// Current extraction step
    pub current_step: String,
    /// Progress (0.0 - 1.0)
    pub progress: f32,
    /// Total files to extract
    pub total_files: usize,
    /// Files extracted so far
    pub extracted_files: usize,
}

/// Configuration file tracking setup completion
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SetupConfig {
    /// Whether initial setup has been completed
    pub setup_complete: bool,
    /// Path where assets are stored
    pub assets_path: PathBuf,
    /// Path to disc image (for re-extraction if needed)
    pub disc_path: Option<PathBuf>,
}

impl SetupConfig {
    /// Load config from file, or create default if missing
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => warn!("Failed to parse config: {}", e),
                },
                Err(e) => warn!("Failed to read config: {}", e),
            }
        }

        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get config file path (platform-specific)
    fn config_path() -> PathBuf {
        // Use platform-specific config directory
        #[cfg(target_os = "windows")]
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));

        #[cfg(not(target_os = "windows"))]
        let base = dirs::config_dir().unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });

        base.join("legaia").join("config.toml")
    }

    /// Get assets directory path
    pub fn assets_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));

        #[cfg(not(target_os = "windows"))]
        let base = dirs::data_dir().unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local").join("share")
        });

        base.join("legaia").join("assets")
    }
}

/// Plugin for first-run setup
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SetupState>()
            .init_resource::<SetupProgress>()
            .add_systems(OnEnter(SetupState::CheckSetup), check_setup)
            .add_systems(
                Update,
                prompt_disc_path.run_if(in_state(SetupState::PromptDiscPath)),
            )
            .add_systems(OnEnter(SetupState::ValidateDisc), validate_disc)
            .add_systems(
                Update,
                extract_assets.run_if(in_state(SetupState::Extracting)),
            )
            .add_systems(OnEnter(SetupState::Complete), finish_setup);
    }
}

/// Check if setup has been completed
fn check_setup(mut next_state: ResMut<NextState<SetupState>>) {
    let config = SetupConfig::load();

    if config.setup_complete && config.assets_path.exists() {
        info!(
            "Setup already complete, assets found at: {:?}",
            config.assets_path
        );
        next_state.set(SetupState::Complete);
    } else {
        info!("First run detected, starting setup...");
        next_state.set(SetupState::PromptDiscPath);
    }
}

/// Prompt user for disc path (placeholder - will be replaced with UI)
fn prompt_disc_path(
    mut progress: ResMut<SetupProgress>,
    mut next_state: ResMut<NextState<SetupState>>,
) {
    // TODO: Replace with actual UI
    // For now, check common locations or environment variable

    if progress.disc_path.is_none() {
        // Check environment variable
        if let Ok(disc_path) = std::env::var("LEGAIA_DISC_PATH") {
            let path = PathBuf::from(disc_path);
            if path.exists() {
                info!("Found disc at: {:?}", path);
                progress.disc_path = Some(path);
                next_state.set(SetupState::ValidateDisc);
            }
        } else {
            // TODO: Show UI to prompt for disc path
            warn!("No disc path provided. Set LEGAIA_DISC_PATH environment variable or implement UI prompt.");
        }
    }
}

/// Validate the disc image
fn validate_disc(progress: Res<SetupProgress>, mut next_state: ResMut<NextState<SetupState>>) {
    if let Some(disc_path) = &progress.disc_path {
        // Try to open the disc
        match psxutils::cdrom::CdRom::open(disc_path) {
            Ok(_cdrom) => {
                info!("Disc validated successfully");
                // TODO: Verify it's the correct game (check SYSTEM.CNF or SCUS_942.54)
                next_state.set(SetupState::Extracting);
            }
            Err(e) => {
                error!("Failed to open disc: {}", e);
                // TODO: Show error UI and go back to prompt
                next_state.set(SetupState::PromptDiscPath);
            }
        }
    } else {
        error!("No disc path set");
        next_state.set(SetupState::PromptDiscPath);
    }
}

/// Extract assets from disc
fn extract_assets(
    mut progress: ResMut<SetupProgress>,
    mut next_state: ResMut<NextState<SetupState>>,
) {
    // TODO: Implement actual extraction
    // For now, just simulate progress

    progress.progress += 0.01;
    progress.current_step = format!("Extracting assets... {:.0}%", progress.progress * 100.0);

    if progress.progress >= 1.0 {
        info!("Asset extraction complete!");

        // Save config
        let config = SetupConfig {
            setup_complete: true,
            assets_path: SetupConfig::assets_dir(),
            disc_path: progress.disc_path.clone(),
        };

        if let Err(e) = config.save() {
            error!("Failed to save config: {}", e);
        }

        next_state.set(SetupState::Complete);
    }
}

/// Finish setup and transition to game
fn finish_setup() {
    info!("Setup complete! Starting game...");
    // TODO: Transition to main menu state
}
