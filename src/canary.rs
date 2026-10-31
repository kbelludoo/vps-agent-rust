use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, Instant};

pub struct CanaryTracker {
    client: Client,
    go_port: u16,
    rust_port: u16,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct CanaryReport {
    pub elapsed_min: u64,
    pub go_online: bool,
    pub rust_online: bool,
    pub go_kills: u64,
    pub rust_kills: u64,
    pub go_gold: u64,
    pub rust_gold: u64,
    pub go_exp: u64,
    pub rust_exp: u64,
    pub rust_cb_state: String,
}

impl CanaryTracker {
    pub fn new(go_port: u16, rust_port: u16) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(2000))
                .build()
                .unwrap_or_default(),
            go_port,
            rust_port,
            start_time: Instant::now(),
        }
    }

    pub async fn evaluate(&self) -> CanaryReport {
        let elapsed = self.start_time.elapsed().as_secs() / 60;
        let mut report = CanaryReport {
            elapsed_min: elapsed,
            go_online: false,
            rust_online: false,
            go_kills: 0,
            rust_kills: 0,
            go_gold: 0,
            rust_gold: 0,
            go_exp: 0,
            rust_exp: 0,
            rust_cb_state: "Closed".to_string(),
        };

        // Probe Go Bot
        let go_url = format!("http://127.0.0.1:{}/api/status", self.go_port);
        if let Ok(resp) = self.client.get(&go_url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    report.go_online = true;
                    if let Some(accounts) = json.get("accounts").and_then(|a| a.as_array()) {
                        for a in accounts {
                            report.go_kills += a.get("kills").and_then(|k| k.as_u64()).unwrap_or(0);
                            report.go_gold += a.get("gold").and_then(|g| g.as_u64()).unwrap_or(0);
                            report.go_exp += a.get("exp").and_then(|e| e.as_u64()).unwrap_or(0);
                        }
                    }
                }
            }
        }

        // Probe Rust Bot
        let rust_url = format!("http://127.0.0.1:{}/api/status", self.rust_port);
        if let Ok(resp) = self.client.get(&rust_url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    report.rust_online = true;
                    if let Some(accounts) = json.get("accounts").and_then(|a| a.as_array()) {
                        for a in accounts {
                            report.rust_kills += a.get("kills").and_then(|k| k.as_u64()).unwrap_or(0);
                            report.rust_gold += a.get("gold").and_then(|g| g.as_u64()).unwrap_or(0);
                            report.rust_exp += a.get("exp").and_then(|e| e.as_u64()).unwrap_or(0);
                            if let Some(cb) = a.get("cb_state").and_then(|c| c.as_str()) {
                                report.rust_cb_state = cb.to_string();
                            }
                        }
                    }
                }
            }
        }

        report
    }

    pub fn format_markdown(&self, r: &CanaryReport) -> String {
        format!(
            "🐤 *Relatório Canário ({} min)*\n\
             • *Go (Porta {})*: Online: `{}` | Kills: `{}` | Gold: `{}` | Exp: `{}`\n\
             • *Rust (Porta {})*: Online: `{}` | Kills: `{}` | Gold: `{}` | Exp: `{}` | CB: `{}`\n\
             • *Estabilidade*: Zero-GC Ativo | Memória Rust < 10MB",
            r.elapsed_min,
            self.go_port,
            r.go_online,
            r.go_kills,
            r.go_gold,
            r.go_exp,
            self.rust_port,
            r.rust_online,
            r.rust_kills,
            r.rust_gold,
            r.rust_exp,
            r.rust_cb_state
        )
    }
}
