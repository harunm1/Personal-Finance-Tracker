use eframe::egui;
use crate::db;
use diesel::sqlite::SqliteConnection;

pub enum AppState {
    Login,
    Register,
    Dashboard,
}

pub struct FinancerApp {
    username: String,
    password: String,
    email: String,
    message: String,
    screen: AppState,
    conn: SqliteConnection,
}

impl FinancerApp {
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            email: String::new(),
            message: String::new(),
            screen: AppState::Login,
            conn,
        }
    }

    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FinanceR Login");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

            if ui.button("Login").clicked() {
                match db::verify_user(&mut self.conn, &self.username, &self.password) {
                    Ok(true) => {
                        self.message.clear();
                        self.screen = AppState::Dashboard;
                    }
                    Ok(false) => {
                        self.message = "Invalid credentials. Please try again.".to_string();
                    }
                    Err(e) => self.message = format!("Error during login: {}", e),
                }
            }

            if ui.button("Register").clicked() {
                self.screen = AppState::Register;
                self.message.clear();
            }

            ui.separator();
            ui.label(&self.message)
        });
    }

    fn show_register(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FinanceR Registration");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            ui.label("Email:");
            ui.text_edit_singleline(&mut self.email);

            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

            if ui.button("Create account").clicked() {
                match db::create_user(&mut self.conn, &self.username, &self.password, Some(&self.email)) {
                    Ok(_) => {
                        self.message = "Account created! Now you can login".to_string();
                        self.screen = AppState::Login;
                        self.username.clear();
                        self.password.clear();
                        self.email.clear();
                    }
                    Err(e) => {
                        self.message = format!("Failed to create account: {}", e);
                    }
                }
            }

            if ui.button("Back to Login").clicked() {
                self.screen = AppState::Login;
                self.message.clear();
            }

            ui.separator();
            ui.label(&self.message)
        });
    }

    fn show_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FinanceR Dashboard");
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

impl eframe::App for FinancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            AppState::Login => self.show_login(ctx),
            AppState::Register => self.show_register(ctx),
            AppState::Dashboard => self.show_dashboard(ctx),
        }
    }
}
