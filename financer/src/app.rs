use eframe::egui;
use crate::db;
use diesel::sqlite::SqliteConnection;
use diesel::result::{Error, DatabaseErrorKind};
use crate::models::Account;

pub enum AppState {
    Login,
    Register,
    Dashboard,
}

pub struct FinancerApp {
    username: String,
    user_id: Option<i32>, //initially there wont be a value until the user logs in
    password: String,
    email: String,
    message: String,
    screen: AppState,
    conn: SqliteConnection,
    accounts_list: Vec<Account>,
    new_account_name: String,
    new_account_type: String,   
    new_account_balance: f32,
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
            user_id: None,
            accounts_list: Vec::new(),
            new_account_name: String::new(),
            new_account_type: String::new(),
            new_account_balance: 0.0,
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
                        // get user_id
                        self.user_id = db::get_userid_by_username(&mut self.conn, &self.username).ok().map(|u| u.id);
                        // load user accounts
                        if let Some(uid) = self.user_id {
                            self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                        }
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
                if self.username.is_empty() || self.password.is_empty() {
                    self.message = "Username and password cannot be empty.".to_string();
                    return;
                }

                match db::create_user(&mut self.conn, &self.username, &self.password, Some(&self.email)) {
                    Ok(_) => {
                        self.message = "Account created! Now you can login".to_string();
                        self.screen = AppState::Login;
                        self.username.clear();
                        self.password.clear();
                        self.email.clear();
                    }
                    Err(Error::DatabaseError(kind, _)) => {
                        self.message = match kind {
                            DatabaseErrorKind::UniqueViolation => "Username or email already exists.".to_string(),
                            _ => "Database error occurred.".to_string(),
                        }

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
            ui.label(&self.message);
        });
    }

    fn show_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FinanceR Dashboard");
            ui.label(format!("Logged in as: {}", self.username));

            ui.separator();
            ui.heading("Your Accounts:");

            if self.accounts_list.is_empty() {
                ui.label("No accounts found. Please create one.");
            } else {
                for account in &self.accounts_list {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} - {}: ${:.2}", account.name, account.account_type, account.balance));
                    });
                }
            }

            ui.separator();
            ui.heading("Create New Account:");

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_account_name);
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.text_edit_singleline(&mut self.new_account_type);
            });

            ui.horizontal(|ui| {
                ui.label("Balance:");
                ui.add(egui::DragValue::new(&mut self.new_account_balance).speed(1.0));
            });

            if ui.button("Create Account").clicked() {
                if let Some(uid) = self.user_id {
                    match db::create_account(&mut self.conn, &self.new_account_name, &self.new_account_type, self.new_account_balance, uid) {
                        Ok(_) => {
                            self.message = "Account created successfully.".to_string();
                            // Refresh account list
                            self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                            // Clear input fields
                            self.new_account_name.clear();
                            self.new_account_type.clear();
                            self.new_account_balance = 0.0;
                        }
                        Err(e) => {
                            self.message = format!("Failed to create account: {}", e);
                        }
                    }
                } 
            }

            ui.separator();
            ui.label(&self.message);

            if ui.button("Logout").clicked() {
                self.screen = AppState::Login;
                self.username.clear();
                self.password.clear();
                self.user_id = None;
                self.accounts_list.clear();
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
