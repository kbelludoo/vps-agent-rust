use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretsConfig {
    #[serde(default)]
    pub telegram_token: String,
    #[serde(default)]
    pub telegram_chat_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramUser {
    pub id: i64,
    pub first_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Clone)]
pub struct TelegramNotifier {
    client: Client,
    token: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new() -> Self {
        let (token, chat_id) = Self::load_config();
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(12))
                .build()
                .unwrap_or_default(),
            token,
            chat_id,
        }
    }

    fn load_config() -> (String, String) {
        if let (Ok(t), Ok(c)) = (env::var("TELEGRAM_TOKEN"), env::var("TELEGRAM_CHAT_ID")) {
            if !t.is_empty() && !c.is_empty() {
                return (t, c);
            }
        }

        let candidates = [
            ".secrets.json",
            "../bot-template-go/.secrets.json",
            "C:/Users/kbell/Downloads/bot-template-go/.secrets.json",
            "/home/ubuntu/.secrets.json",
        ];

        for path in &candidates {
            if Path::new(path).exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(cfg) = serde_json::from_str::<SecretsConfig>(&content) {
                        if !cfg.telegram_token.is_empty() && !cfg.telegram_chat_id.is_empty() {
                            return (cfg.telegram_token, cfg.telegram_chat_id);
                        }
                    }
                }
            }
        }

        (String::new(), String::new())
    }

    pub fn is_configured(&self) -> bool {
        !self.token.is_empty() && !self.chat_id.is_empty()
    }

    pub fn authorized_chat_id(&self) -> &str {
        &self.chat_id
    }

    pub async fn send_alert(&self, message: &str) {
        self.send_message_to(&self.chat_id, message).await;
    }

    pub async fn send_message_to(&self, chat_id: &str, message: &str) {
        if !self.is_configured() {
            return;
        }

        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let payload = json!({
            "chat_id": chat_id,
            "text": message,
            "parse_mode": "Markdown",
            "disable_web_page_preview": true
        });

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("📢 [Telegram] Mensagem enviada com sucesso");
                } else {
                    error!("📢 [Telegram] Falha ao enviar: HTTP {}", resp.status());
                }
            }
            Err(e) => {
                error!("📢 [Telegram] Erro na requisição: {}", e);
            }
        }
    }

    /// Faz Long Polling no Telegram para buscar novas mensagens
    pub async fn get_updates(&self, offset: i64) -> Result<Vec<TelegramUpdate>, reqwest::Error> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=5",
            self.token, offset
        );

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json_val: Value = resp.json().await?;
        if let Some(results) = json_val.get("result") {
            if let Ok(updates) = serde_json::from_value::<Vec<TelegramUpdate>>(results.clone()) {
                return Ok(updates);
            }
        }

        Ok(Vec::new())
    }
}

impl Default for TelegramNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_loader() {
        let notifier = TelegramNotifier::new();
        if Path::new("C:/Users/kbell/Downloads/bot-template-go/.secrets.json").exists() {
            assert!(notifier.is_configured());
        }
    }
}
