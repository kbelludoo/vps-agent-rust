# ⚡ VPS Agent Rust — Watchdog & Canary Orchestrator

Ultra-lightweight, zero-GC Telegram daemon and system watchdog for VPS bot cluster orchestration written in native Rust (Stable 2021). Consumes **< 5 MB RAM** (replacing Go daemon).

## 🚀 Capabilities
- **Telegram Command Listener:** Responds to status queries, memory audits, and restart signals.
- **Canary Tracker:** Monitors Go vs Rust parity and latency in real time.
- **Auto-Recovery Watchdog:** Restarts failed systemd units automatically with rate limiting.
- **Scrubber:** Strips tokens, emails, IPs, and credentials from telemetry and error logs.

## 🌐 Live Telemetry
- **GitHub Pages Dashboard:** [https://kbelludoo.github.io/vps-agent-rust/](https://kbelludoo.github.io/vps-agent-rust/)
