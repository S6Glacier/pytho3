
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
    fn test_to_string() {
        let psc = PermashortCitation {
            protocol: String::from("https"),
            domain: String::from("vdx.hu"),
            short_url: String::from("s/Df3l"),
        };

        assert_eq!(psc.to_string().as_str(), "vdx.hu s/Df3l");
    }

    #[test]
    fn test_to_uri() {
        let psc = PermashortCitation {
            protocol: String::from("https"),
            domain: String::from("vdx.hu"),
            short_url: String::from("s/Df3l"),
        };

        assert_eq!(psc.to_uri().as_str(), "https://vdx.hu/s/Df3l");
    }
}