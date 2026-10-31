mod canary;
mod command_handler;
mod probe;
mod scrub;
mod system_metrics;
mod telegram;
mod watchdog;

use canary::CanaryTracker;
use command_handler::TelegramCommandHandler;
use probe::BotFleetProber;
use system_metrics::collect_system_metrics;
use telegram::TelegramNotifier;
use watchdog::WatchdogController;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    println!("================================================================================");
    println!("       ⚡ VPS-AGENT-RUST: WATCHDOG & CANARY FLEET ORCHESTRATOR ⚡              ");
    println!("================================================================================");

    let notifier = Arc::new(TelegramNotifier::new());
    let prober = Arc::new(BotFleetProber::new());
    let watchdog = Arc::new(WatchdogController::new(60, 3));
    let canary = Arc::new(CanaryTracker::new(8090, 3001));

    if notifier.is_configured() {
        info!("📢 Notificador do Telegram configurado e ativo.");
        notifier.send_alert("🚀 *VPS-Agent-Rust Iniciado*\nWatchdog, Orquestrador e Listener Interativo de Comandos ativos na VPS.").await;

        // Inicia a escuta ativa de mensagens e perguntas do Telegram em background
        let cmd_handler = Arc::new(TelegramCommandHandler::new(notifier.clone(), prober.clone()));
        tokio::spawn(async move {
            cmd_handler.start_polling_loop().await;
        });
    } else {
        warn!("⚠️ Telegram não configurado. Defina TELEGRAM_TOKEN e TELEGRAM_CHAT_ID para alertas.");
    }

    let mut tick = 0u64;

    loop {
        sleep(Duration::from_secs(10)).await;
        tick += 1;

        // 1. Probar a frota dos bots
        let results = prober.probe_all().await;
        let mut online_count = 0;
        let total_bots = results.len();

        for r in &results {
            if r.online {
                online_count += 1;
            } else {
                // Tenta recuperação automática via Watchdog
                let service = format!("bot-{}.service", r.id);
                if let Some(recovery_msg) = watchdog.check_and_recover(&service, false).await {
                    notifier.send_alert(&recovery_msg).await;
                }
            }
        }

        // 2. Coletar métricas da VPS
        let metrics = collect_system_metrics();

        // 3. Ticker no terminal a cada 30 segundos (a cada 3 ticks de 10s)
        if tick.is_multiple_of(3) {
            info!(
                "[Fleet Ticker #{}] Bots Online: {}/{} | RAM VPS: {}MB ({}%) | Load: {}",
                tick, online_count, total_bots, metrics.ram_used_mb, metrics.ram_percent, metrics.load
            );
        }

        // 4. Relatório Canário a cada 5 minutos (30 ticks)
        if tick.is_multiple_of(30) {
            let canary_report = canary.evaluate().await;
            if canary_report.rust_online || canary_report.go_online {
                let md = canary.format_markdown(&canary_report);
                info!("🐤 [Canary] {}", md.replace('\n', " | "));
                notifier.send_alert(&md).await;
            }
        }

        // 5. Alerta de sobrecarga de memória na VPS (> 90%)
        if metrics.ram_percent > 90 {
            warn!("🚨 [VPS Alerta] Consumo de memória crítico: {}%", metrics.ram_percent);
            if tick.is_multiple_of(60) {
                let alert = format!("🚨 *ALERTA VPS CRÍTICO*\nRAM acima de 90%: `{}MB / {}MB`\nLoad: `{}`", metrics.ram_used_mb, metrics.ram_total_mb, metrics.load);
                notifier.send_alert(&alert).await;
            }
        }
    }
}
