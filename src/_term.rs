
/// Drain factory creating drains logging to the terminal
pub struct TermDrainFactory;

impl DrainFactory for TermDrainFactory {
    fn from_config(&self, config: &config::Output) -> Result<Option<Drain>, String> {
        use std::io;

        let type_ = config.get("type").ok_or("output type missing")?;

        if type_ != "term" {
            return Ok(None)
        }
        let format_str = config.get("format").ok_or("format_missing")?;

        let format_mode = match format_str.as_str() {
            "compact" => slog_term::FormatMode::Compact,
            "full" => slog_term::FormatMode::Full,
            _ => return Err(format!("unknown terminal format: {}", format_str))
        };

        // TODO: this should probably use an actual boolean type, but the config is currently
        // parsed as just strings
        let use_stdout = match config.get("use_stdout").map_or("false", |v| v.as_str()) {
            "true" => true,
            "false" => false,
            _ => return Err("use_stdout must be true or false".to_owned())
        };

        let use_color = match config.get("color").map_or("auto", |v| v.as_str()) {
            "true" => true,
            "false" => false,
            "auto" => if use_stdout {
                isatty::stdout_isatty()
            } else {
                isatty::stderr_isatty()
            },
            _ => return Err("color must be true, false, or auto".to_owned())
        };

        let color_decorator = if use_color {
            slog_term::ColorDecorator::new_colored()
        } else {
            slog_term::ColorDecorator::new_plain()
        };

        let timestamp_fn: Box<slog_term::TimestampFn> = match config.get("timestamp").map_or("local", |v| v.as_str()) {
            "local" => Box::new(slog_term::timestamp_local),
            "utc" => Box::new(slog_term::timestamp_utc),
            unknown => return Err(format!("unknown timestamp type: {}", unknown))
        };

        let format = slog_term::Format::new(format_mode, color_decorator, timestamp_fn);
        let io: Box<io::Write + Send + Sync> = if use_stdout {
            Box::new(io::stdout())
        } else {
            Box::new(io::stderr())
        };

        let drain = slog_stream::stream(io, format);

        Ok(Some(Box::new(BoxErrorDrain(drain))))
    }
}
