#[must_use]
pub fn parse_bool_env(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(val) => match val.to_lowercase().as_str() {
            "" | "0" | "false" | "f" | "no" | "n" | "off" => false,
            "1" | "true" | "t" | "yes" | "y" | "on" => true,
            _ => panic!("Invalid value for boolean environment variable {name}: {val}"),
        },
        Err(_) => default,
    }
}
