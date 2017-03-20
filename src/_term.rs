
/// Drain factory creating drains logging to the terminal
pub struct TermDrainFactory;

impl DrainFactory for TermDrainFactory {
    fn from_config(&self, config: &config::Output) -> Result<Option<Drain>, String> {
        let type_ = config.get("type").ok_or("output type missing")?;

        if type_ != "term" {
            return Ok(None)
        }

        let decorator = slog_term::TermDecorator::new();

        // TODO: this should probably use an actual boolean type, but the config is currently
        // parsed as just strings
        let decorator = match config.get("use_stdout").map_or("false", |v| v.as_str()) {
            "true" => decorator.stdout(),
            "false" => decorator.stderr(),
            _ => return Err("use_stdout must be true or false".to_owned())
        };

        let decorator = match config.get("color").map_or("auto", |v| v.as_str()) {
            "true" => decorator.force_color(),
            "false" => decorator.force_plain(),
            "auto" => decorator,
            _ => return Err("color must be true, false, or auto".to_owned())
        };

        let utc  = match config.get("timestamp").map_or("utc", |v| v.as_str()) {
            "local" => false,
            "utc" => true,
            unknown => return Err(format!("unknown timestamp type: {}", unknown))
        };
        let format_str = config.get("format").ok_or("format_missing")?;
        match format_str.as_str() {
            "compact" => {
                let format = slog_term::CompactFormat::new(decorator.build());
                let format = if utc { format.use_utc_timestamp() } else {
                    format.use_local_timestamp() };
                Ok(Some(Box::new(BoxErrorDrain(Mutex::new(format.build())))))
            },
            "full" =>
            {
                let format = slog_term::FullFormat::new(decorator.build());
                let format = if utc { format.use_utc_timestamp() } else {
                    format.use_local_timestamp() };
                Ok(Some(Box::new(BoxErrorDrain(Mutex::new(format.build())))))
            },
            _ => return Err(format!("unknown terminal format: {}", format_str))
        }
    }
}
