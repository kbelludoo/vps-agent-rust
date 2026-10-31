use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{error, info, warn};

pub struct WatchdogController {
    cooldowns: Mutex<HashMap<String, Instant>>,
    failure_counts: Mutex<HashMap<String, u32>>,
    cooldown_duration: Duration,
    max_consecutive_failures: u32,
}

impl WatchdogController {
    pub fn new(cooldown_sec: u64, max_failures: u32) -> Self {
        Self {
            cooldowns: Mutex::new(HashMap::new()),
            failure_counts: Mutex::new(HashMap::new()),
            cooldown_duration: Duration::from_secs(cooldown_sec),
            max_consecutive_failures: max_failures,
        }
    }

    fn should_restart(&self, service_name: &str, is_online: bool) -> bool {
        let mut fails = self.failure_counts.lock().unwrap();
        let mut cds = self.cooldowns.lock().unwrap();

        if is_online {
            fails.insert(service_name.to_string(), 0);
            return false;
        }

        let count = fails.entry(service_name.to_string()).or_insert(0);
        *count += 1;

        if *count < self.max_consecutive_failures {
            warn!("⚠️ [Watchdog] {} falhou {}/{} vezes. Aguardando limite para reiniciar...", service_name, count, self.max_consecutive_failures);
            return false;
        }

        if let Some(last_restart) = cds.get(service_name) {
            if last_restart.elapsed() < self.cooldown_duration {
                let rem = (self.cooldown_duration - last_restart.elapsed()).as_secs();
                warn!("⏳ [Watchdog] Cooldown ativo para {} ({}s restantes)", service_name, rem);
                return false;
            }
        }

        cds.insert(service_name.to_string(), Instant::now());
        *count = 0;
        true
    }

    pub async fn check_and_recover(&self, service_name: &str, is_online: bool) -> Option<String> {
        if !self.should_restart(service_name, is_online) {
            return None;
        }

        // Executa reinício do serviço systemd
        info!("🔄 [Watchdog] Disparando reinício do serviço: sudo systemctl restart {}", service_name);

        match Command::new("sudo")
            .args(["systemctl", "restart", service_name])
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    let msg = format!("♻️ *Watchdog Auto-Recovery*: Serviço `{}` reiniciado com sucesso na VPS.", service_name);
                    info!("{}", msg);
                    Some(msg)
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    error!("❌ [Watchdog] Falha ao reiniciar {}: {}", service_name, err);
                    None
                }
            }
            Err(e) => {
                error!("❌ [Watchdog] Erro ao invocar systemctl: {}", e);
                None
            }
        }
    }
}
