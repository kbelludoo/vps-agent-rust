use regex::Regex;
use serde_json::Value;

#[allow(dead_code)]
pub struct DataScrubber {
    nick_regex: Regex,
    email_regex: Regex,
    ip_regex: Regex,
}

#[allow(dead_code)]
impl DataScrubber {
    pub fn new() -> Self {
        Self {
            nick_regex: Regex::new(r"(?i)(l?kbelludoo\w*|lohmoreira|l?lrn\b|l?maumau\w*|klord\w*|secondpally\w*|mojo\b)").unwrap(),
            email_regex: Regex::new(r"(?i)[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            ip_regex: Regex::new(r"\b(129\.153|141\.148|193\.122|147\.15|163\.176)\.\d{1,3}\.\d{1,3}\b").unwrap(),
        }
    }

    pub fn scrub_string(&self, input: &str) -> String {
        if input.starts_with("http://") || input.starts_with("https://") {
            return input.to_string();
        }
        let s = self.email_regex.replace_all(input, "[EMAIL_PROTEGIDO]");
        let s = self.ip_regex.replace_all(&s, "127.0.0.1");
        let s = self.nick_regex.replace_all(&s, "anonimo");
        s.to_string()
    }

    pub fn deep_scrub(&self, value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.scrub_string(s)),
            Value::Array(arr) => {
                let scrubbed: Vec<Value> = arr.iter().map(|v| self.deep_scrub(v)).collect();
                Value::Array(scrubbed)
            }
            Value::Object(map) => {
                let mut scrubbed_map = serde_json::Map::new();
                for (k, v) in map {
                    let clean_key = if !k.starts_with("http") {
                        self.nick_regex.replace_all(k, "anonimo").to_string()
                    } else {
                        k.clone()
                    };
                    scrubbed_map.insert(clean_key, self.deep_scrub(v));
                }
                Value::Object(scrubbed_map)
            }
            _ => value.clone(),
        }
    }
}

impl Default for DataScrubber {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_scrub() {
        let scrubber = DataScrubber::new();
        let payload = json!({
            "user": "kbelludoo",
            "email": "test@domain.com",
            "server": "147.15.109.14",
            "status": "online"
        });

        let clean = scrubber.deep_scrub(&payload);
        assert_eq!(clean["user"], "anonimo");
        assert_eq!(clean["email"], "[EMAIL_PROTEGIDO]");
        assert_eq!(clean["server"], "127.0.0.1");
        assert_eq!(clean["status"], "online");
    }
}
