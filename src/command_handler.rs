use std::sync::Arc;
use tracing::info;

use crate::probe::BotFleetProber;
use crate::system_metrics::collect_system_metrics;
use crate::telegram::TelegramNotifier;

pub struct TelegramCommandHandler {
    notifier: Arc<TelegramNotifier>,
    prober: Arc<BotFleetProber>,
}

impl TelegramCommandHandler {
    pub fn new(notifier: Arc<TelegramNotifier>, prober: Arc<BotFleetProber>) -> Self {
        Self { notifier, prober }
    }

    /// Processa uma mensagem recebida e retorna a resposta formatada
    pub async fn handle_message(&self, chat_id: &str, text: &str) {
        // Validação de segurança: apenas responde ao chat_id autorizado
        if chat_id != self.notifier.authorized_chat_id() {
            info!("🔒 [Telegram] Mensagem ignorada de chat_id não autorizado: {}", chat_id);
            return;
        }

        let trimmed = text.trim();
        let lower = trimmed.to_lowercase();

        // 1. Perguntas sobre Memória / RAM
        if lower.contains("memoria") || lower.contains("memória") || lower.contains("ram") || lower == "/ram" {
            let m = collect_system_metrics();
            let response = format!(
                "🧠 *Status de Memória da VPS (OCI Ashburn)*\n\n\
                • *Em Uso:* `{} MB` ({}%)\n\
                • *Livre / Disponível:* `{} MB`\n\
                • *Total Instalado:* `{} MB`\n\
                • *Swap em Uso:* `{} MB / {} MB`\n\
                • *Load Average:* `{}`\n\
                • *Status:* 🟢 Normal (Zero-GC)",
                m.ram_used_mb, m.ram_percent, m.ram_available_mb, m.ram_total_mb,
                m.swap_used_mb, m.swap_total_mb, m.load
            );
            self.notifier.send_message_to(chat_id, &response).await;
            return;
        }

        // 2. Status Geral da Frota / VPS
        if lower == "/status" || lower == "status" || lower.contains("como ta") || lower.contains("como tá") {
            let m = collect_system_metrics();
            let results = self.prober.probe_all().await;
            let online = results.iter().filter(|r| r.online).count();
            let total = results.len();

            let mut total_gold: u64 = 0;
            let mut total_kills: u64 = 0;
            for r in &results {
                total_gold += r.total_gold;
                total_kills += r.total_kills;
            }

            let response = format!(
                "⚡ *Status Geral da VPS & Frota*\n\n\
                🤖 *Bots Operando:* `{}/{} Online`\n\
                🧠 *RAM VPS:* `{} MB / {} MB` ({}%)\n\
                ⏱️ *Uptime:* `{}`\n\
                📊 *Load Average:* `{}`\n\
                💰 *Ouro Total Consolidado:* `{} gp`\n\
                ⚔️ *Kills Totais:* `{}`\n\n\
                Use `/bots` para ver lista ou `/ouro` para detalhamento.",
                online, total, m.ram_used_mb, m.ram_total_mb, m.ram_percent,
                m.uptime, m.load, format_number(total_gold), format_number(total_kills)
            );
            self.notifier.send_message_to(chat_id, &response).await;
            return;
        }

        // 3. Listagem detalhada dos bots
        if lower == "/bots" || lower == "bots" || lower.contains("lista") {
            let results = self.prober.probe_all().await;
            let mut lines = Vec::new();
            lines.push("🤖 *Frota de Bots na VPS:*\n".to_string());

            for r in &results {
                let status_icon = if r.online { "🟢" } else { "🔴" };
                lines.push(format!(
                    "{} *{}* (Porta {})\n   └ Status: `{}` | Ouro: `{} gp` | Kills: `{}`",
                    status_icon, r.name, r.port,
                    if r.online { "ONLINE" } else { "OFFLINE" },
                    format_number(r.total_gold),
                    format_number(r.total_kills)
                ));
            }

            let response = lines.join("\n");
            self.notifier.send_message_to(chat_id, &response).await;
            return;
        }

        // 4. Detalhamento de Ouro
        if lower == "/ouro" || lower == "ouro" || lower.contains("gold") {
            let results = self.prober.probe_all().await;
            let mut lines = Vec::new();
            lines.push("💰 *Ranking de Ouro por Bot:*\n".to_string());
            let mut total: u64 = 0;

            for r in &results {
                if r.total_gold > 0 {
                    total += r.total_gold;
                    lines.push(format!("• {}: `{} gp`", r.name, format_number(r.total_gold)));
                }
            }
            lines.push(format!("\n💎 *Total Geral:* `{} gp`", format_number(total)));

            let response = lines.join("\n");
            self.notifier.send_message_to(chat_id, &response).await;
            return;
        }

        // 5. Ajuda / Comandos disponíveis
        if lower == "/help" || lower == "ajuda" || lower == "/start" {
            let help_msg = "🤖 *Central de Comando da VPS (Telegram Bot)*\n\n\
                Você pode me fazer perguntas em texto ou usar comandos:\n\n\
                • `memória` ou `/ram` — Consumo de RAM, swap e load da VPS\n\
                • `status` ou `/status` — Visão geral da máquina e frota\n\
                • `bots` ou `/bots` — Lista de todos os bots e portas\n\
                • `ouro` ou `/ouro` — Ouro acumulado em cada jogo\n\
                • `/huntera` — Informações específicas do Huntera\n\
                • `/ajuda` — Mostra este menu";
            self.notifier.send_message_to(chat_id, help_msg).await;
            return;
        }

        // 6. Huntera específico
        if lower.contains("huntera") {
            let results = self.prober.probe_all().await;
            if let Some(h) = results.iter().find(|r| r.id == "huntera") {
                let resp = format!(
                    "⚔️ *Huntera Idle — Status*\n\n\
                    • Status: `{}`\n\
                    • Porta: `{}`\n\
                    • Contas Ativas: `{}`\n\
                    • Ouro Farmado: `{} gp`\n\
                    • Kills: `{}`\n\
                    • Latência API: `{} ms`",
                    if h.online { "ONLINE (Rust Zero-GC)" } else { "OFFLINE" },
                    h.port, h.active_accounts,
                    format_number(h.total_gold),
                    format_number(h.total_kills),
                    h.response_time_ms
                );
                self.notifier.send_message_to(chat_id, &resp).await;
                return;
            }
        }

        // Resposta padrão inteligente caso não case com comando exato
        let fallback = format!(
            "🤖 Recebi sua mensagem: _{}_\n\n\
            Para consultar os recursos da VPS, tente:\n\
            • `quanto de memória em uso?` ou `/ram`\n\
            • `/status` para resumo geral.",
            trimmed
        );
        self.notifier.send_message_to(chat_id, &fallback).await;
    }

    /// Loop em background que escuta novas mensagens (Long Polling)
    pub async fn start_polling_loop(self: Arc<Self>) {
        info!("🤖 [Telegram Bot] Polling de comandos iniciado!");
        let mut offset = 0i64;

        loop {
            match self.notifier.get_updates(offset).await {
                Ok(updates) => {
                    for u in updates {
                        offset = u.update_id + 1;
                        if let Some(msg) = u.message {
                            if let Some(txt) = msg.text {
                                let chat_id_str = msg.chat.id.to_string();
                                self.handle_message(&chat_id_str, &txt).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ [Telegram Polling] Erro de conexão: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}

fn format_number(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    if n < 1_000_000 {
        return format!("{:.1}k", n as f64 / 1_000.0);
    }
    format!("{:.2}kk", n as f64 / 1_000_000.0)
}
