use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotProbe {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub url: String,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub online: bool,
    pub game: String,
    pub active_accounts: usize,
    pub total_kills: u64,
    pub total_gold: u64,
    pub total_exp: u64,
    pub cb_states: Vec<String>,
    pub response_time_ms: u128,
}

pub struct BotFleetProber {
    client: Client,
    probes: Vec<BotProbe>,
}

impl BotFleetProber {
    pub fn new() -> Self {
        let probes = vec![
            BotProbe {
                id: "huntera".to_string(),
                name: "⚔️ Huntera Idle".to_string(),
                port: 3000,
                url: "http://127.0.0.1:3000/api/status".to_string(),
                service_name: "bot-huntera.service".to_string(),
            },
            BotProbe {
                id: "baiak".to_string(),
                name: "🔥 Baiak Idle".to_string(),
                port: 3001,
                url: "http://127.0.0.1:3001/api/status".to_string(),
                service_name: "bot-baiak.service".to_string(),
            },
            BotProbe {
                id: "tibiaidle".to_string(),
                name: "🛡️ Tibia Idle".to_string(),
                port: 3002,
                url: "http://127.0.0.1:3002/api/status".to_string(),
                service_name: "bot-tibiaidle.service".to_string(),
            },
            BotProbe {
                id: "poketrue".to_string(),
                name: "⚡ PokeTrue".to_string(),
                port: 3003,
                url: "http://127.0.0.1:3003/api/status".to_string(),
                service_name: "bot-poketrue.service".to_string(),
            },
            BotProbe {
                id: "pokeidle".to_string(),
                name: "⚡ PokeIdle".to_string(),
                port: 3004,
                url: "http://127.0.0.1:3004/api/status".to_string(),
                service_name: "bot-pokeidle.service".to_string(),
            },
            BotProbe {
                id: "lolidle".to_string(),
                name: "🗡️ LoL Idle".to_string(),
                port: 3005,
                url: "http://127.0.0.1:3005/api/status".to_string(),
                service_name: "bot-lolidle.service".to_string(),
            },
            BotProbe {
                id: "luxidle".to_string(),
                name: "🌟 Lux Idle".to_string(),
                port: 3006,
                url: "http://127.0.0.1:3006/api/status".to_string(),
                service_name: "bot-luxidle.service".to_string(),
            },
            BotProbe {
                id: "sigma".to_string(),
                name: "🔮 Sigma Idle".to_string(),
                port: 3007,
                url: "http://127.0.0.1:3007/api/status".to_string(),
                service_name: "bot-sigma.service".to_string(),
            },
            BotProbe {
                id: "leveling".to_string(),
                name: "🏹 Leveling Idle".to_string(),
                port: 3008,
                url: "http://127.0.0.1:3008/api/status".to_string(),
                service_name: "bot-leveling.service".to_string(),
            },
            BotProbe {
                id: "farmbot".to_string(),
                name: "🌾 Farm Ricochet".to_string(),
                port: 3009,
                url: "http://127.0.0.1:3009/api/status".to_string(),
                service_name: "bot-farmbot.service".to_string(),
            },
            BotProbe {
                id: "pokelegacy".to_string(),
                name: "🐲 PokeLegacy".to_string(),
                port: 3010,
                url: "http://127.0.0.1:3010/api/status".to_string(),
                service_name: "bot-pokelegacy.service".to_string(),
            },
        ];

        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(2500))
                .build()
                .unwrap_or_default(),
            probes,
        }
    }

    #[allow(dead_code)]
    pub fn probes(&self) -> &[BotProbe] {
        &self.probes
    }

    pub async fn probe_single(&self, probe: &BotProbe) -> ProbeResult {
        let start = std::time::Instant::now();
        let mut result = ProbeResult {
            id: probe.id.clone(),
            name: probe.name.clone(),
            port: probe.port,
            online: false,
            game: String::new(),
            active_accounts: 0,
            total_kills: 0,
            total_gold: 0,
            total_exp: 0,
            cb_states: Vec::new(),
            response_time_ms: 0,
        };

        match self.client.get(&probe.url).send().await {
            Ok(resp) => {
                result.response_time_ms = start.elapsed().as_millis();
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<Value>().await {
                        result.online = true;
                        result.game = json.get("game").and_then(|g| g.as_str()).unwrap_or("Unknown").to_string();

                        if let Some(accounts) = json.get("accounts").and_then(|a| a.as_array()) {
                            result.active_accounts = accounts.len();
                            for acc in accounts {
                                result.total_kills += acc.get("kills").and_then(|k| k.as_u64()).unwrap_or(0);
                                result.total_gold += acc.get("gold").and_then(|g| g.as_u64()).unwrap_or(0);
                                result.total_exp += acc.get("exp").and_then(|e| e.as_u64()).unwrap_or(0);
                                if let Some(cb) = acc.get("cb_state").and_then(|c| c.as_str()) {
                                    result.cb_states.push(cb.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                result.response_time_ms = start.elapsed().as_millis();
            }
        }

        result
    }

    pub async fn probe_all(&self) -> Vec<ProbeResult> {
        let mut results = Vec::new();
        for p in &self.probes {
            results.push(self.probe_single(p).await);
        }
        results
    }
}

impl Default for BotFleetProber {
    fn default() -> Self {
        Self::new()
    }
}
