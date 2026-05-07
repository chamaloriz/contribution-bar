use std::time::{Duration, Instant};
use tray_icon::{TrayIcon, TrayIconBuilder, menu::Menu};
use winit::{
    application::ApplicationHandler,
    event::StartCause,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

mod gitea;
mod github;
mod image;
use image::{generate_icon, load_icon};
use serde::{Deserialize, Serialize};

const REFRESH_DELAY_SECS: u64 = 3600;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct AppConfig {
    version: i32,
    provider: String,
    github_username: String,
    gitea_username: String,
    gitea_server_url: String,
    gitea_token: Option<String>,
}

impl ::std::default::Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 2,
            provider: "gitea".to_string(),
            github_username: "chamaloriz".to_string(),
            gitea_username: "chamaloriz".to_string(),
            gitea_server_url: "https://gitea.selfhosted.be".to_string(),
            gitea_token: None,
        }
    }
}

#[derive(Debug)]
enum UserEvent {}

struct Application {
    tray_icon: Option<TrayIcon>,
    next_switch: Instant,
    config: AppConfig,
}

impl Application {
    fn new() -> Self {
        Self {
            tray_icon: None,
            next_switch: Instant::now() + Duration::from_secs(REFRESH_DELAY_SECS),
            config: confy::load("contribution-bar", None).expect("issue in config loading"),
        }
    }

    fn generate_icon(&mut self) -> tray_icon::Icon {
        let cfg = &self.config;
        let result = match cfg.provider.as_str() {
            "gitea" => gitea::get_contributions(
                &cfg.gitea_server_url,
                &cfg.gitea_username,
                cfg.gitea_token.as_deref(),
            )
            .map_err(|e| format!("{e}")),
            _ => github::get_contributions(&cfg.github_username).map_err(|e| format!("{e}")),
        };

        match result {
            Ok(contributions) => {
                self.next_switch = Instant::now() + Duration::from_secs(REFRESH_DELAY_SECS);
                generate_icon(contributions, &cfg.provider)
            }
            Err(_error) => {
                self.next_switch = Instant::now() + Duration::from_secs(30);
                load_icon("warning")
            }
        }
    }

    fn set_tray_icon(&mut self) {
        let icon = self.generate_icon();

        if let Some(tray_icon) = &mut self.tray_icon {
            tray_icon.set_icon(Some(icon)).unwrap();
        }
    }

    fn new_tray_icon(&mut self) -> TrayIcon {
        let icon = self.generate_icon();
        let tray_menu = Menu::new();
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Commit History")
            .with_icon(icon)
            .build()
            .unwrap()
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_switch));
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init => {
                #[cfg(not(target_os = "linux"))]
                {
                    self.tray_icon = Some(self.new_tray_icon());
                }
                #[cfg(target_os = "macos")]
                {
                    use objc2_core_foundation::CFRunLoop;
                    let rl = CFRunLoop::main().unwrap();
                    CFRunLoop::wake_up(&rl);
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_switch));
            }
            StartCause::ResumeTimeReached { .. } => {
                self.set_tray_icon();
                event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_switch));
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: UserEvent) {}
}

fn main() {
    #[cfg(not(debug_assertions))]
    {
        std::env::set_current_dir(
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .expect("Failed to get executable directory"),
        )
        .expect("Failed to set working directory");
    }

    let mut builder = EventLoop::<UserEvent>::with_user_event();

    #[cfg(target_os = "windows")]
    let event_loop = builder.build().unwrap();

    #[cfg(target_os = "macos")]
    let event_loop = builder
        .with_activation_policy(ActivationPolicy::Accessory)
        .build()
        .unwrap();

    #[cfg(target_os = "linux")]
    let event_loop = builder.build().unwrap();

    let mut app = Application::new();

    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        gtk::init().unwrap();
        let mut app = Application::new();
        let _tray_icon = Application::new_tray_icon(&mut app);
        gtk::main();
    });

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {err:?}");
    }
}
