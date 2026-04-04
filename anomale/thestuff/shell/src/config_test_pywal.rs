#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_app_config_pywal_auto_detection() -> anyhow::Result<()> {
        // Create a temp directory for configs
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join("anomale");
        fs::create_dir_all(&config_dir)?;

        // Override XDG_CONFIG_HOME to point to temp dir
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        // Create apps.conf with pywal colors but NO pywal=true
        let apps_config_path = config_dir.join("apps.conf");
        let mut file = fs::File::create(&apps_config_path)?;
        writeln!(file, "background_color=pywal_color0")?;
        writeln!(file, "border_color=pywal_color1")?;
        writeln!(file, "background_opacity=0.8")?;

        // Load config
        let config = AppConfig::load()?;

        // Verify pywal was auto-detected
        assert!(config.pywal, "Pywal should be auto-detected when pywal_color is used");
        
        // Verify opacity was parsed
        assert_eq!(config.background_opacity, 0.8, "Background opacity should be 0.8");

        Ok(())
    }
}
