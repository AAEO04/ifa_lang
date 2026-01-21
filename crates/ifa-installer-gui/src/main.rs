use clap::Parser;
use eframe::egui;
use ifa_installer_core::{
    args::InstallerArgs, check, config::InstallConfig, install, profiles, uninstall,
};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

const IFA_GREEN: egui::Color32 = egui::Color32::from_rgb(46, 125, 50);
const IFA_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);

fn main() -> Result<(), eframe::Error> {
    let args = InstallerArgs::parse();

    if args.headless {
        run_headless(args);
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Ifá-Lang Installer"),
        ..Default::default()
    };

    eframe::run_native(
        "Ifá-Lang Installer",
        options,
        Box::new(|cc| {
            configure_styles(&cc.egui_ctx);
            Box::new(InstallerApp::new(cc))
        }),
    )
}

fn configure_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = IFA_BG;
    visuals.panel_fill = IFA_BG;
    visuals.hyperlink_color = IFA_GREEN;

    // Customizing widgets
    visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_gray(60);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(30);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(40);
    visuals.widgets.active.bg_fill = egui::Color32::from_gray(50);

    visuals.selection.bg_fill = IFA_GREEN;

    ctx.set_visuals(visuals);
}

fn run_headless(args: InstallerArgs) {
    println!("Running in Headless Mode...");

    if args.uninstall {
        let dir = args
            .dir
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".ifa"));
        match uninstall::uninstall(&dir) {
            Ok(_) => println!("Uninstalled successfully."),
            Err(e) => eprintln!("Uninstall failed: {}", e),
        }
        return;
    }

    println!("Checking system...");
    let sys = check::check_system();
    println!("Detected: {} {}", sys.os, sys.arch);

    let config = InstallConfig {
        install_dir: args
            .dir
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".ifa")),
        ..Default::default()
    };

    println!("Installing Ifá-Lang (complete bundle)...");
    let components = profiles::all_components();

    match install::install(&config, &components) {
        Ok(_) => println!("Installation complete!"),
        Err(e) => eprintln!("Installation failed: {}", e),
    }
}

#[derive(PartialEq)]
enum AppState {
    Welcome,
    Config,
    Installing,
    Finished,
}

struct InstallerApp {
    state: AppState,
    config: InstallConfig,
    install_log: String,
    log_receiver: Option<Receiver<String>>,
    install_progress: f32,
}

impl InstallerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::Welcome,
            config: InstallConfig::default(),
            install_log: String::new(),
            log_receiver: None,
            install_progress: 0.0,
        }
    }

    fn start_install(&mut self) {
        self.state = AppState::Installing;
        let (tx, rx) = channel();
        self.log_receiver = Some(rx);

        let config = self.config.clone();

        thread::spawn(move || {
            let _ = tx.send("Init...".to_string());
            thread::sleep(std::time::Duration::from_millis(500)); // Sim delay

            let _ = tx.send("Checking requirements...".to_string());
            let components = profiles::all_components();

            match install::install(&config, &components) {
                Ok(_) => {
                    let _ = tx.send("Done!".to_string());
                }
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e));
                }
            }
        });
    }

    fn render_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(
                egui::RichText::new(title)
                    .size(28.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(20.0);
        });
    }

    fn render_components_list(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Components to Install:").strong());
            ui.add_space(10.0);
            for component in profiles::all_components() {
                ui.horizontal(|ui| {
                    ui.label("✓");
                    ui.label(
                        egui::RichText::new(&component.name)
                            .strong()
                            .color(IFA_GREEN),
                    );
                    ui.label(format!("- {}", component.description));
                });
            }
        });
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.log_receiver {
            while let Ok(msg) = rx.try_recv() {
                self.install_log.push_str(&format!("{}\n", msg));
                if msg == "Done!" {
                    self.install_progress = 1.0;
                    self.state = AppState::Finished;
                } else {
                    self.install_progress += 0.1; // Fake progress increment
                    if self.install_progress > 0.9 {
                        self.install_progress = 0.9;
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Welcome => {
                    self.render_header(ui, "Welcome to Ifá-Lang");
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("The Yoruba Programming Language Toolchain")
                                .size(16.0),
                        );
                        ui.add_space(20.0);

                        // System Check Badge
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("System Detected:");
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {}",
                                        std::env::consts::OS,
                                        std::env::consts::ARCH
                                    ))
                                    .strong()
                                    .color(IFA_GREEN),
                                );
                                ui.label("✓");
                            });
                        });

                        ui.add_space(20.0);
                    });

                    // Show what will be installed
                    self.render_components_list(ui);

                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Configure Installation").size(18.0),
                                )
                                .min_size([200.0, 40.0].into()),
                            )
                            .clicked()
                        {
                            self.state = AppState::Config;
                        }
                    });
                }
                AppState::Config => {
                    self.render_header(ui, "Configuration");

                    ui.group(|ui| {
                        ui.label("Installation Directory:");
                        let mut path_str = self.config.install_dir.to_string_lossy().to_string();
                        if ui.text_edit_singleline(&mut path_str).changed() {
                            self.config.install_dir = std::path::PathBuf::from(path_str);
                        }
                        // Note: Real file picker would usually go via rfd here if button clicked
                        if ui.button("Browse...").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder()
                        {
                            self.config.install_dir = path;
                        }
                    });

                    ui.add_space(20.0);
                    ui.checkbox(
                        &mut self.config.add_to_path,
                        "Add to PATH environment variable",
                    );
                    ui.checkbox(&mut self.config.create_shortcut, "Create Desktop Shortcut");
                    ui.checkbox(
                        &mut self.config.update_shell,
                        "Update Shell Profile (.bashrc/.zshrc)",
                    );

                    ui.add_space(40.0);
                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            self.state = AppState::Welcome;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("INSTALL").size(20.0).strong(),
                                    )
                                    .fill(IFA_GREEN)
                                    .min_size([200.0, 50.0].into()),
                                )
                                .clicked()
                            {
                                self.start_install();
                            }
                        });
                    });
                }
                AppState::Installing => {
                    self.render_header(ui, "Installing Ifá-Lang...");

                    ui.add(egui::ProgressBar::new(self.install_progress).show_percentage());
                    ui.add_space(20.0);

                    ui.label("Installation Log:");
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.install_log)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(10)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                }
                AppState::Finished => {
                    self.render_header(ui, "Installation Complete!");
                    ui.vertical_centered(|ui| {
                        // Logo would go here
                        ui.add_space(20.0);
                        ui.label("Ifá-Lang has been successfully installed.");
                        ui.label("Please restart your terminal to use the 'ifa' command.");
                        ui.add_space(40.0);
                        if ui.button("Finish").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }
            }
        });
    }
}
