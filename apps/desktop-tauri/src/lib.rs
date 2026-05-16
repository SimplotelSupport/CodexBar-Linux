use std::sync::Mutex;
use std::time::Duration;

use codexbar_core::core::{FetchContext, ProviderId, SourceMode, instantiate_provider};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderRow {
    pub id: String,
    pub display_name: String,
    pub source: String,
    pub account_email: Option<String>,
    pub login_method: Option<String>,
    pub primary_label: String,
    pub primary_pct: f64,
    pub primary_reset: Option<String>,
    pub secondary_label: Option<String>,
    pub secondary_pct: Option<f64>,
    pub secondary_reset: Option<String>,
    pub cost_used: Option<String>,
    pub cost_limit: Option<String>,
    pub cost_period: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshot {
    pub fetched_at: String,
    pub providers: Vec<ProviderRow>,
}

#[derive(Default)]
pub struct AppState {
    pub last_snapshot: Mutex<Option<UsageSnapshot>>,
}

const ALPHA_PROVIDERS: &[ProviderId] = &[
    ProviderId::Codex,
    ProviderId::Claude,
    ProviderId::Copilot,
    ProviderId::OpenAIApi,
    ProviderId::OpenRouter,
];

async fn fetch_one(id: ProviderId) -> ProviderRow {
    let provider = instantiate_provider(id);
    let meta = provider.metadata().clone();
    let ctx = FetchContext {
        source_mode: SourceMode::Auto,
        include_credits: true,
        web_timeout: 30,
        verbose: false,
        manual_cookie_header: None,
        api_key: None,
    };

    match provider.fetch_usage(&ctx).await {
        Ok(result) => {
            let primary = &result.usage.primary;
            let secondary = result.usage.secondary.as_ref();
            ProviderRow {
                id: id.cli_name().to_string(),
                display_name: id.display_name().to_string(),
                source: result.source_label.clone(),
                account_email: result.usage.account_email.clone(),
                login_method: result.usage.login_method.clone(),
                primary_label: meta.session_label.to_string(),
                primary_pct: primary.used_percent,
                primary_reset: primary.format_countdown(),
                secondary_label: secondary.map(|_| meta.weekly_label.to_string()),
                secondary_pct: secondary.map(|s| s.used_percent),
                secondary_reset: secondary.and_then(|s| s.format_countdown()),
                cost_used: result.cost.as_ref().map(|c| c.format_used()),
                cost_limit: result.cost.as_ref().and_then(|c| c.format_limit()),
                cost_period: result.cost.as_ref().map(|c| c.period.clone()),
                error: None,
            }
        }
        Err(e) => ProviderRow {
            id: id.cli_name().to_string(),
            display_name: id.display_name().to_string(),
            source: "error".to_string(),
            account_email: None,
            login_method: None,
            primary_label: meta.session_label.to_string(),
            primary_pct: 0.0,
            primary_reset: None,
            secondary_label: None,
            secondary_pct: None,
            secondary_reset: None,
            cost_used: None,
            cost_limit: None,
            cost_period: None,
            error: Some(e.to_string()),
        },
    }
}

async fn fetch_all() -> UsageSnapshot {
    let mut rows = Vec::with_capacity(ALPHA_PROVIDERS.len());
    for id in ALPHA_PROVIDERS {
        rows.push(fetch_one(*id).await);
    }
    UsageSnapshot {
        fetched_at: chrono_now(),
        providers: rows,
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

#[tauri::command]
async fn refresh_usage(app: AppHandle) -> Result<UsageSnapshot, String> {
    let snap = fetch_all().await;
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut guard) = state.last_snapshot.lock() {
            *guard = Some(snap.clone());
        }
    }
    let _ = app.emit("usage-updated", &snap);
    Ok(snap)
}

#[tauri::command]
fn get_cached_snapshot(state: tauri::State<'_, AppState>) -> Option<UsageSnapshot> {
    state.last_snapshot.lock().ok().and_then(|g| g.clone())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit CodexBar", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &show, &quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("embedded tray icon")
        }))
        .tooltip("CodexBar — AI usage monitor")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "refresh" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = refresh_usage(app).await;
                });
            }
            "show" => {
                if let Some(win) = app.get_webview_window("popover") {
                    let _ = win.show();
                    let _ = win.set_focus();
                    let _ = win.unminimize();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("popover") {
                    let is_visible = win.is_visible().unwrap_or(false);
                    if is_visible {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn spawn_poller(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(120));
        loop {
            ticker.tick().await;
            let _ = refresh_usage(app.clone()).await;
        }
    });
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(win) = app.get_webview_window("popover") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState::default())
        .setup(|app| {
            build_tray(&app.handle().clone())?;

            if let Some(win) = app.get_webview_window("popover") {
                let _ = win.hide();
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = refresh_usage(app_handle).await;
            });

            spawn_poller(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            refresh_usage,
            get_cached_snapshot,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
