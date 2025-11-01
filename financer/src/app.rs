use eframe::egui;

pub enum AppState {
    Login,
    Dashboard,
}

pub struct FinancerApp {
    username: String,
    password: String,
    message: String,
    screen: AppState,
}

impl Default for FinancerApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            message: String::new(),
            screen: AppState::Login,
        }
    }
}

impl eframe::App for FinancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            AppState::Login => self.show_login(ctx),
            AppState::Dashboard => self.show_dashboard(ctx),
        }
    }
}

impl FinancerApp {
    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FinanceR Login");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

            if ui.button("Login").clicked() {
                if self.username == "user" && self.password == "pass" {
                    self.message.clear();
                    self.screen = AppState::Dashboard; //switch to dashboard view on successful login
                } else {
                    self.message = "Invalid credentials. Please try again.".to_string();
                }
            }

            ui.separator();
            ui.label(&self.message)
        });
    }

    fn show_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to your FinanceR Dashboard");
            ui.label(format!("Logged in as: {}", self.username));

            if ui.button("Logout").clicked() {
                self.screen = AppState::Login;
                self.username.clear();
                self.password.clear();
                self.message.clear();
            }
        });
    }
}