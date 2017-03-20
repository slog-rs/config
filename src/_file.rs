
/// Drain factory creating drains logging to a file
///
/// See `tests` and `examples` for reference.
pub struct FileDrainFactory;

impl DrainFactory for FileDrainFactory {
    fn from_config(&self, config : &config::Output) -> Result<Option<Drain>, String> {
        let type_ = config.get("type").ok_or("output type missing")?;


        if type_ != "file" {
            return Ok(None)
        }
        let path = config.get("path").ok_or("file path missing")?;
        let format_str = config.get("format").ok_or("format missing")?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path).unwrap();

        let drain = match format_str.as_str() {
            "json" => BoxErrorDrain(
                Mutex::new(
                slog_json::Json::new(file).build()
                )),
            _ => return Err(format!("unkown file format: {}", format_str)),
        };

        Ok(Some(Box::new(drain)))
    }
}
