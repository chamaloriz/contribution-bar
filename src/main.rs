use objc2_core_foundation::CFRunLoop;
use std::time::{Duration, Instant};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::{
    application::ApplicationHandler,
    event::StartCause,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
};

mod github;
mod image;
use github::get_contributions;
use image::{generate_icon, load_icon};
use serde::{Deserialize, Serialize};

const REFRESH_DELAY_SECS: u64 = 3600;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    version: i32,
    github_username: String,
}

impl ::std::default::Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            github_username: "chamaloriz".to_string(),
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
        let username = &self.config.github_username;
        match get_contributions(username) {
            Ok(contributions) => {
                self.next_switch = Instant::now() + Duration::from_secs(REFRESH_DELAY_SECS);
                generate_icon(contributions)
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

        TrayIconBuilder::new()
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
                self.tray_icon = Some(self.new_tray_icon());
                let rl = CFRunLoop::main().unwrap();
                CFRunLoop::wake_up(&rl);
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
    let event_loop = builder
        .with_activation_policy(ActivationPolicy::Accessory)
        .build()
        .unwrap();

    let mut app = Application::new();

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {err:?}");
    }
}
