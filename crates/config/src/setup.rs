//! First-run setup wizard

use anyhow::Result;
use std::io::{self, Write};

use super::settings::Config;

/// Check if first-run setup is needed
pub fn needs_setup(config: &Config) -> bool {
    !config.has_api_keys()
}

/// Display security guidance
pub fn display_security_guidance() {
    println!();
    println!("╭─────────────────── API Key Security ───────────────────╮");
    println!("│                                                        │");
    println!("│  Your API keys grant access to paid services.          │");
    println!("│  Keep them secure:                                     │");
    println!("│                                                        │");
    println!("│  ✓ Use environment variables (recommended)             │");
    println!("│    export NEWS_API_KEY=\"your-key\"                      │");
    println!("│                                                        │");
    println!("│  ✓ Or use a .env file (add to .gitignore)             │");
    println!("│                                                        │");
    println!("│  ✗ Avoid committing keys to version control            │");
    println!("│  ✗ Don't share config files containing keys            │");
    println!("│                                                        │");
    println!("╰────────────────────────────────────────────────────────╯");
    println!();
}

/// Run interactive setup wizard
pub fn run_setup_wizard(config: &mut Config) -> Result<()> {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("           Welcome to The Daily Update!");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Check for existing env vars
    let missing = config.missing_api_keys();

    if missing.is_empty() {
        println!("✓ All API keys found in environment variables!");
    } else {
        println!("Missing API keys: {}", missing.join(", "));
        println!();
        println!("You can set them as environment variables:");
        for key in &missing {
            println!("  export {}=\"your-key-here\"", key);
        }
        println!();

        // Ask if user wants to enter them now
        print!("Would you like to enter them now? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            prompt_for_api_keys(config, &missing)?;
        }
    }

    // Prompt for default location
    println!();
    print!(
        "Enter your default location for weather [{}]: ",
        config.general.default_location
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() {
        config.general.default_location = input.to_string();
    }

    // Ask about vim mode
    print!("Enable vim-style keybindings? (y/n) [n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    config.general.vim_mode = input.trim().to_lowercase() == "y";

    // Save config
    config.save()?;

    display_security_guidance();

    println!("Configuration saved to: {:?}", Config::config_path()?);
    println!();

    Ok(())
}

fn prompt_for_api_keys(config: &mut Config, missing: &[&str]) -> Result<()> {
    println!();
    println!("Note: Keys entered here will be saved to config file.");
    println!("For better security, use environment variables instead.");
    println!();

    for key in missing {
        print!("Enter {}: ", key);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim().to_string();

        if !value.is_empty() {
            match *key {
                "NEWS_API_KEY" => config.apis.news_api_key = Some(value),
                "WEATHER_API_KEY" => config.apis.weather_api_key = Some(value),
                "TIINGO_API_KEY" => config.apis.tiingo_api_key = Some(value),
                _ => {}
            }
        }
    }

    Ok(())
}

/// Display startup banner
pub fn display_banner() {
    println!();
    println!("╔═════════════════════════════════════════╗");
    println!("║       The Daily Update v0.1.0         ║");
    println!("╚═════════════════════════════════════════╝");
}
