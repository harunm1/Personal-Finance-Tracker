use eframe::egui;

pub struct FinancerApp {
    username: String,
    password: String,
    message: String,
}

impl Default for FinancerApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            message: String::new(),
        }
    }
}

impl eframe::App for FinancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Personal Finance Tracker - Login");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            ui.label("Password:");
            ui.text_edit_singleline(&mut self.password);

            if ui.button("Login").clicked() {
                // Placeholder for authentication logic
                if self.username == "user" && self.password == "pass" {
                    self.message = "Login successful!".to_string();
                } else {
                    self.message = "Invalid credentials.".to_string();
                }
            }

            ui.label(&self.message);
        });
    }
}