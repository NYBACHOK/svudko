use std::sync::LazyLock;

pub const LOCAL_HOSTNAME_PREFIX: &str = ".local";

pub static HOSTNAME: LazyLock<Hostname> = LazyLock::new(|| {
    Hostname::new(
        gethostname::gethostname()
            .to_string_lossy()
            .replace(char::REPLACEMENT_CHARACTER, ""),
    )
});

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hostname(String);

impl From<String> for Hostname {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.0
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<String> for Hostname {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Hostname {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let value = if value.ends_with(".local") {
            value.replace(".local", "")
        } else {
            value
        };

        Self(value)
    }

    pub fn to_local_dns_name(&self) -> String {
        format!("{}{LOCAL_HOSTNAME_PREFIX}", self.0)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
