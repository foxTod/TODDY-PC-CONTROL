use argon2::{
    Argon2,
    PasswordHash,
    PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
use eframe::egui;
use rand_core::OsRng;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(PartialEq)]
enum Screen {
    Setup,
    Login,
    Dashboard,
}

pub struct SetupApp {
    screen: Screen,
    password: String,
    password_repeat: String,
    telegram_id: String,
    telegram_token: String,
    message: String,
    bot_running: Arc<AtomicBool>,
    monitor: Option<crate::monitor::PcMonitor>,
    monitor_error: Option<String>,
}

impl Default for SetupApp {
    fn default() -> Self {
        let config = crate::config::Config::load().ok();

        let screen = if config.as_ref()
            .and_then(|c| c.password_hash.as_ref())
            .is_some()
        {
            Screen::Login
        } else {
            Screen::Setup
        };

        Self {
            screen,
            password: String::new(),
            password_repeat: String::new(),
            telegram_id: config.as_ref()
                .map(|c| c.admin_id.to_string())
                .unwrap_or_default(),
            telegram_token: config.as_ref()
                .map(|c| c.telegram_token.clone())
                .unwrap_or_default(),
            message: String::new(),
            bot_running: Arc::new(AtomicBool::new(false)),
            monitor: None,
            monitor_error: None,
        }
    }
}

impl SetupApp {
    fn header(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(
                egui::RichText::new("TODDY PC CONTROL")
                    .size(30.0)
                    .color(egui::Color32::from_rgb(255, 135, 45))
                    .strong()
            );
            ui.label(
                egui::RichText::new("DESKTOP CONTROL CENTER")
                    .size(11.0)
                    .color(egui::Color32::GRAY)
            );
        });
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
    }

    fn save_settings(&mut self) {
        if self.password.len() < 8 {
            self.message = "\u{041f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c} \u{0434}\u{043e}\u{043b}\u{0436}\u{0435}\u{043d} \u{0441}\u{043e}\u{0434}\u{0435}\u{0440}\u{0436}\u{0430}\u{0442}\u{044c} \u{043c}\u{0438}\u{043d}\u{0438}\u{043c}\u{0443}\u{043c} 8 \u{0441}\u{0438}\u{043c}\u{0432}\u{043e}\u{043b}\u{043e}\u{0432}".into();
            return;
        }

        if self.password != self.password_repeat {
            self.message = "\u{041f}\u{0430}\u{0440}\u{043e}\u{043b}\u{0438} \u{043d}\u{0435} \u{0441}\u{043e}\u{0432}\u{043f}\u{0430}\u{0434}\u{0430}\u{044e}\u{0442}".into();
            return;
        }

        let admin_id = match self.telegram_id.trim().parse::<u64>() {
            Ok(id) if id > 0 => id,
            _ => {
                self.message = "\u{041d}\u{0435}\u{0432}\u{0435}\u{0440}\u{043d}\u{044b}\u{0439} Telegram ID".into();
                return;
            }
        };

        if self.telegram_token.trim().is_empty() {
            self.message = "\u{041d}\u{0435} \u{0443}\u{043a}\u{0430}\u{0437}\u{0430}\u{043d} \u{0442}\u{043e}\u{043a}\u{0435}\u{043d} Telegram-\u{0431}\u{043e}\u{0442}\u{0430}".into();
            return;
        }

        let salt = SaltString::generate(&mut OsRng);

        let password_hash = match Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
        {
            Ok(hash) => hash.to_string(),
            Err(_) => {
                self.message = "\u{041d}\u{0435} \u{0443}\u{0434}\u{0430}\u{043b}\u{043e}\u{0441}\u{044c} \u{0441}\u{043e}\u{0437}\u{0434}\u{0430}\u{0442}\u{044c} \u{0445}\u{0435}\u{0448} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044f}".into();
                return;
            }
        };

        let config = crate::config::Config {
            telegram_token: self.telegram_token.trim().to_string(),
            admin_id,
            password_hash: Some(password_hash),
        };

        match config.save() {
            Ok(()) => {
                self.password.clear();
                self.password_repeat.clear();
                self.screen = Screen::Login;
                self.message = "\u{041d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0430} \u{0437}\u{0430}\u{0432}\u{0435}\u{0440}\u{0448}\u{0435}\u{043d}\u{0430}. \u{0412}\u{043e}\u{0439}\u{0434}\u{0438}\u{0442}\u{0435} \u{0432} \u{043f}\u{0440}\u{0438}\u{043b}\u{043e}\u{0436}\u{0435}\u{043d}\u{0438}\u{0435}.".into();
            }
            Err(error) => {
                self.message = format!("\u{041e}\u{0448}\u{0438}\u{0431}\u{043a}\u{0430} \u{0441}\u{043e}\u{0445}\u{0440}\u{0430}\u{043d}\u{0435}\u{043d}\u{0438}\u{044f}: {error}");
            }
        }
    }

    fn login(&mut self) {
        let config = match crate::config::Config::load() {
            Ok(config) => config,
            Err(_) => {
                self.message = "\u{041d}\u{0435} \u{0443}\u{0434}\u{0430}\u{043b}\u{043e}\u{0441}\u{044c} \u{0437}\u{0430}\u{0433}\u{0440}\u{0443}\u{0437}\u{0438}\u{0442}\u{044c} \u{043d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0438}".into();
                return;
            }
        };

        let hash = match config.password_hash {
            Some(hash) => hash,
            None => {
                self.screen = Screen::Setup;
                self.message = "\u{0421}\u{043d}\u{0430}\u{0447}\u{0430}\u{043b}\u{0430} \u{0441}\u{043e}\u{0437}\u{0434}\u{0430}\u{0439}\u{0442}\u{0435} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}".into();
                return;
            }
        };

        let parsed_hash = match PasswordHash::new(&hash) {
            Ok(hash) => hash,
            Err(_) => {
                self.message = "\u{041e}\u{0448}\u{0438}\u{0431}\u{043a}\u{0430} \u{0444}\u{043e}\u{0440}\u{043c}\u{0430}\u{0442}\u{0430} \u{0445}\u{0435}\u{0448}\u{0430} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044f}".into();
                return;
            }
        };

        if Argon2::default()
            .verify_password(self.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            self.password.clear();
            self.screen = Screen::Dashboard;
            self.message = "\u{0412}\u{0445}\u{043e}\u{0434} \u{0432}\u{044b}\u{043f}\u{043e}\u{043b}\u{043d}\u{0435}\u{043d}".into();
        } else {
            self.password.clear();
            self.message = "\u{041d}\u{0435}\u{0432}\u{0435}\u{0440}\u{043d}\u{044b}\u{0439} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}".into();
        }
    }

    fn start_bot(&mut self) {
        if self.bot_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = Arc::clone(&self.bot_running);

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build();

            match runtime {
                Ok(runtime) => {
                    runtime.block_on(crate::bot::run_bot());
                }
                Err(error) => {
                    eprintln!("Failed to start Telegram bot: {error}");
                }
            }

            running.store(false, Ordering::SeqCst);
        });
    }

    fn status_message(&self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(&self.message)
                    .color(egui::Color32::from_rgb(255, 145, 65))
            );
        });
    }

    fn draw_setup(&mut self, ui: &mut egui::Ui) {
        ui.heading("\u{041f}\u{0435}\u{0440}\u{0432}\u{043e}\u{043d}\u{0430}\u{0447}\u{0430}\u{043b}\u{044c}\u{043d}\u{0430}\u{044f} \u{043d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0430}");
        ui.label("\u{0421}\u{043e}\u{0437}\u{0434}\u{0430}\u{0439}\u{0442}\u{0435} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c} \u{0434}\u{043b}\u{044f} \u{0437}\u{0430}\u{0449}\u{0438}\u{0442}\u{044b} \u{043f}\u{0440}\u{0438}\u{043b}\u{043e}\u{0436}\u{0435}\u{043d}\u{0438}\u{044f}.");
        ui.add_space(18.0);

        ui.label("\u{041f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}");
        ui.add(
            egui::TextEdit::singleline(&mut self.password)
                .password(true)
                .hint_text("\u{041c}\u{0438}\u{043d}\u{0438}\u{043c}\u{0443}\u{043c} 8 \u{0441}\u{0438}\u{043c}\u{0432}\u{043e}\u{043b}\u{043e}\u{0432}")
                .desired_width(f32::INFINITY)
        );

        ui.add_space(10.0);
        ui.label("\u{041f}\u{043e}\u{0432}\u{0442}\u{043e}\u{0440}\u{0438}\u{0442}\u{0435} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}");
        ui.add(
            egui::TextEdit::singleline(&mut self.password_repeat)
                .password(true)
                .desired_width(f32::INFINITY)
        );

        ui.add_space(15.0);
        ui.label("Telegram ID");
        ui.add(
            egui::TextEdit::singleline(&mut self.telegram_id)
                .desired_width(f32::INFINITY)
        );

        ui.add_space(10.0);
        ui.label("\u{0422}\u{043e}\u{043a}\u{0435}\u{043d} Telegram-\u{0431}\u{043e}\u{0442}\u{0430}");
        ui.add(
            egui::TextEdit::singleline(&mut self.telegram_token)
                .password(true)
                .desired_width(f32::INFINITY)
        );

        ui.add_space(22.0);

        if ui.add_sized(
            [ui.available_width(), 44.0],
            egui::Button::new("\u{0421}\u{043e}\u{0445}\u{0440}\u{0430}\u{043d}\u{0438}\u{0442}\u{044c} \u{043d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0438}")
                .fill(egui::Color32::from_rgb(190, 83, 25))
        ).clicked() {
            self.save_settings();
        }

        self.status_message(ui);
    }

    fn draw_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("\u{0410}\u{0432}\u{0442}\u{043e}\u{0440}\u{0438}\u{0437}\u{0430}\u{0446}\u{0438}\u{044f}");
            ui.label("\u{0412}\u{0432}\u{0435}\u{0434}\u{0438}\u{0442}\u{0435} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c} \u{0434}\u{043b}\u{044f} \u{0432}\u{0445}\u{043e}\u{0434}\u{0430}.");
        });

        ui.add_space(25.0);
        ui.label("\u{041f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}");

        ui.add(
            egui::TextEdit::singleline(&mut self.password)
                .password(true)
                .hint_text("\u{0412}\u{0430}\u{0448} \u{043f}\u{0430}\u{0440}\u{043e}\u{043b}\u{044c}")
                .desired_width(f32::INFINITY)
        );

        ui.add_space(18.0);

        if ui.add_sized(
            [ui.available_width(), 44.0],
            egui::Button::new("\u{0412}\u{043e}\u{0439}\u{0442}\u{0438}")
                .fill(egui::Color32::from_rgb(190, 83, 25))
        ).clicked() {
            self.login();
        }

        self.status_message(ui);
    }

    fn draw_monitoring(&mut self, ui: &mut egui::Ui) {
        if self.monitor.is_none() && self.monitor_error.is_none() {
            match crate::monitor::PcMonitor::new() {
                Ok(monitor) => self.monitor = Some(monitor),
                Err(error) => self.monitor_error = Some(error.to_string()),
            }
        }

        if let Some(monitor) = self.monitor.as_mut() {
            monitor.refresh();

            let cpu = monitor.cpu_usage();
            let used = monitor.used_memory() as f64 / 1_073_741_824.0;
            let total = monitor.total_memory() as f64 / 1_073_741_824.0;
            let processes = monitor.process_count();
            let uptime = monitor.uptime_seconds();

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label("PC MONITORING");
                ui.add_space(8.0);

                ui.label(format!("CPU: {:.1}%", cpu));
                ui.add(egui::ProgressBar::new((cpu / 100.0).clamp(0.0, 1.0)));

                ui.add_space(6.0);
                ui.label(format!("RAM: {:.2} GB / {:.2} GB", used, total));
                let ram_fraction = if total > 0.0 {
                    (used / total).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(ram_fraction));

                ui.add_space(6.0);
                ui.label(format!("Processes: {}", processes));

                match uptime {
                    Ok(seconds) => {
                        let days = seconds / 86400;
                        let hours = (seconds % 86400) / 3600;
                        let minutes = (seconds % 3600) / 60;
                        ui.label(format!(
                            "System uptime: {}d {:02}h {:02}m",
                            days, hours, minutes
                        ));
                    }
                    Err(error) => {
                        ui.label(format!("Uptime unavailable: {}", error));
                    }
                }
            });
        } else if let Some(error) = &self.monitor_error {
            ui.label(format!("PC monitoring unavailable: {}", error));
        }
    }

    fn draw_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
        ui.heading("\u{0413}\u{043b}\u{0430}\u{0432}\u{043d}\u{043e}\u{0435} \u{043c}\u{0435}\u{043d}\u{044e}");
        ui.label("\u{0423}\u{043f}\u{0440}\u{0430}\u{0432}\u{043b}\u{0435}\u{043d}\u{0438}\u{0435} \u{0432} TODDY PC CONTROL");
        ui.add_space(15.0);
        self.draw_monitoring(ui);
        ui.add_space(15.0);

        let running = self.bot_running.load(Ordering::SeqCst);

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("TELEGRAM BOT");
            ui.add_space(8.0);

            let status = if running {
                "\u{0421}\u{0442}\u{0430}\u{0442}\u{0443}\u{0441}: \u{0437}\u{0430}\u{043f}\u{0443}\u{0449}\u{0435}\u{043d}"
            } else {
                "\u{0421}\u{0442}\u{0430}\u{0442}\u{0443}\u{0441}: \u{043e}\u{0441}\u{0442}\u{0430}\u{043d}\u{043e}\u{0432}\u{043b}\u{0435}\u{043d}"
            };

            ui.label(
                egui::RichText::new(status)
                    .color(if running {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::GRAY
                    })
            );

            ui.add_space(12.0);

            if ui.add_enabled(
                !running,
                egui::Button::new("\u{0417}\u{0430}\u{043f}\u{0443}\u{0441}\u{0442}\u{0438}\u{0442}\u{044c} Telegram-\u{0431}\u{043e}\u{0442}")
                    .fill(egui::Color32::from_rgb(190, 83, 25))
            ).clicked() {
                self.start_bot();
                self.message = "\u{0417}\u{0430}\u{043f}\u{0443}\u{0441}\u{043a}\u{0430}\u{044e} Telegram-\u{0431}\u{043e}\u{0442}...".into();
            }
        });

        ui.add_space(20.0);

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("\u{041e} \u{043f}\u{0440}\u{043e}\u{0433}\u{0440}\u{0430}\u{043c}\u{043c}\u{0435}");
            ui.add_space(8.0);
            ui.label("TODDY PC CONTROL");
            ui.label("\u{0420}\u{0435}\u{0436}\u{0438}\u{043c}: \u{043b}\u{043e}\u{043a}\u{0430}\u{043b}\u{044c}\u{043d}\u{043e}\u{0435} \u{0443}\u{043f}\u{0440}\u{0430}\u{0432}\u{043b}\u{0435}\u{043d}\u{0438}\u{0435}");
        });

        ui.add_space(25.0);

        if ui.button("\u{0412}\u{044b}\u{0439}\u{0442}\u{0438} \u{0438}\u{0437} \u{0430}\u{043a}\u{043a}\u{0430}\u{0443}\u{043d}\u{0442}\u{0430}").clicked() {
            self.screen = Screen::Login;
            self.password.clear();
            self.message.clear();
            self.monitor = None;
            self.monitor_error = None;
        }

        self.status_message(ui);
    }
}

impl eframe::App for SetupApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(15.0);
            Self::header(ui);

            match self.screen {
                Screen::Setup => self.draw_setup(ui),
                Screen::Login => self.draw_login(ui),
                Screen::Dashboard => self.draw_dashboard(ui),
            }
        });
    }
}

pub fn run_app() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 650.0])
            .with_min_inner_size([420.0, 520.0])
            .with_title("TODDY PC CONTROL"),
        ..Default::default()
    };

    eframe::run_native(
        "TODDY PC CONTROL",
        options,
        Box::new(|_cc| Ok(Box::new(SetupApp::default()))),
    )
}
