
pub struct PermashortCitation {
    protocol: String,
    domain: String,
    short_url: String,
}

impl PermashortCitation {
    #[must_use]
    pub fn new(protocol: String, domain: String, short_url: String) -> Self {
        Self {
            protocol,
            domain,
            short_url,
        }
    }

    #[must_use]
    pub fn to_uri(&self) -> String {
        format!("{}://{}/{}", self.protocol, self.domain, self.short_url)
    }
}

impl ToString for PermashortCitation {
    #[must_use]
    fn to_string(&self) -> String {
        format!("{} {}", self.domain, self.short_url)
    }
}

#[cfg(test)]
mod test {
    use super::PermashortCitation;

    #[test]