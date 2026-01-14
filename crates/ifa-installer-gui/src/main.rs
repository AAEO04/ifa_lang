use clap::Parser;
use eframe::egui;
use ifa_installer_core::{
    args::InstallerArgs, check, config::InstallConfig, install, profiles::Profile, uninstall
};
use std::thread;
use std::sync::mpsc::{channel, Receiver};

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
        let dir = args.dir.unwrap_or_else(|| dirs::home_dir().unwrap().join(".ifa"));
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
        profile: args.profile,
        install_dir: args.dir.unwrap_or_else(|| dirs::home_dir().unwrap().join(".ifa")),
        ..Default::default()
    };

    println!("Installing profile: {:?}", config.profile);
    let components = config.profile.components();
    
    match install::install(&config, &components) {
        Ok(_) => println!("Installation complete!"),
        Err(e) => eprintln!("Installation failed: {}", e),
    }
}

#[derive(PartialEq)]
enum AppState {
    Welcome,
    ProfileSelect,
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
            let components = config.profile.components();
            
            match install::install(&config, &components) {
                Ok(_) => { let _ = tx.send("Done!".to_string()); },
                Err(e) => { let _ = tx.send(format!("Error: {}", e)); }
            }
        });
    }

    fn render_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new(title).size(28.0).strong().color(egui::Color32::WHITE));
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(20.0);
        });
    }

    fn render_profile_card(&mut self, ui: &mut egui::Ui, profile: Profile, title: &str, desc: &str) {
        let is_selected = self.config.profile == profile;
        let bg_color = if is_selected { egui::Color32::from_rgb(30, 40, 30) } else { egui::Color32::from_gray(30) };
        let border_color = if is_selected { IFA_GREEN } else { egui::Color32::TRANSPARENT };

        let card = egui::Frame::group(ui.style())
            .fill(bg_color)
            .stroke(egui::Stroke::new(2.0, border_color))
            .inner_margin(15.0)
            .rounding(5.0);

        card.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).size(18.0).strong());
                    ui.label(desc);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                   if ui.radio(is_selected, "").clicked() {
                       self.config.profile = profile;
                   }
                });
            });
            // Use a stable, unique ID for this card's click interaction
            let card_id = ui.make_persistent_id(format!("card_interact_{:?}", profile));
            if ui.interact(ui.min_rect(), card_id, egui::Sense::click()).clicked() {
                self.config.profile = profile;
            }
        });
        ui.add_space(10.0);
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
                    if self.install_progress > 0.9 { self.install_progress = 0.9; }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Welcome => {
                    self.render_header(ui, "Welcome to Ifá-Lang");
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("The Yoruba Programming Language Toolchain").size(16.0));
                        ui.add_space(40.0);
                        
                        // System Check Badge
                        ui.group(|ui| {
                             ui.horizontal(|ui| {
                                 ui.label("System Detected:");
                                 ui.label(egui::RichText::new(format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)).strong().color(IFA_GREEN));
                                 ui.label("✓");
                             });
                        });
                        
                        ui.add_space(40.0);
                        if ui.add(egui::Button::new(egui::RichText::new("Get Started").size(18.0)).min_size([150.0, 40.0].into())).clicked() {
                            self.state = AppState::ProfileSelect;
                        }
                    });
                }
                AppState::ProfileSelect => {
                    self.render_header(ui, "Select Installation Profile");
                    
                    self.render_profile_card(ui, Profile::Fusion, "Fusion (Recommended)", "Fullstack Hybrid Support, ML Stacks, and Python Bridge.");
                    self.render_profile_card(ui, Profile::Standard, "Standard", "Compiler, Oja Package Manager, and Standard Library.");
                    self.render_profile_card(ui, Profile::Dev, "Developer", "Complete toolchain including LSP, Linter, and debugging tools.");
                    self.render_profile_card(ui, Profile::Minimal, "Minimal", "Compiler and Package Manager only.");

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                         if ui.button("Back").clicked() { self.state = AppState::Welcome; }
                         ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                             if ui.button("Next").clicked() { self.state = AppState::Config; }
                         });
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
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.config.install_dir = path;
                            }
                        }
                    });

                    ui.add_space(20.0);
                    ui.checkbox(&mut self.config.add_to_path, "Add to PATH environment variable");
                    ui.checkbox(&mut self.config.create_shortcut, "Create Desktop Shortcut");
                    ui.checkbox(&mut self.config.update_shell, "Update Shell Profile (.bashrc/.zshrc)");

                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("INSTALL").size(20.0).strong()).fill(IFA_GREEN).min_size([200.0, 50.0].into())).clicked() {
                            self.start_install();
                        }
                    });
                }
                AppState::Installing => {
                    self.render_header(ui, "Installing Ifá-Lang...");
                    
                    ui.add(egui::ProgressBar::new(self.install_progress).show_percentage());
                    ui.add_space(20.0);
                    
                    ui.label("Installation Log:");
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.install_log)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
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
