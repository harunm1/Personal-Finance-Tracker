use eframe::egui;
use eframe::egui::Ui;
use crate::db;
use diesel::sqlite::SqliteConnection;
use diesel::result::{Error, DatabaseErrorKind};
use crate::models::{Account, Transaction};
use crate::models::{Budget, Period};
use egui_plot::{Plot, Line, PlotPoints, Text as PlotText};
use egui::epaint::Shape;
use std::collections::HashMap;
use std::f32::consts::TAU;
use chrono::{NaiveDateTime,NaiveDate,Datelike};
use eframe::egui::Color32;
use csv::Writer;

const DEFAULT_CATEGORIES: &[&str] = &[
    "Food & Dining",
    "Groceries",
    "Transportation",
    "Shopping",
    "Entertainment",
    "Bills & Utilities",
    "Rent/Mortgage",
    "Healthcare",
    "Income",
    "Transfer",
    "Other",
];

pub enum AppState {
    Login,
    Register,
    Dashboard,
    Budgeting,
    Transactions,
    Transfers,
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
    budgets: Vec<Budget>,
    selected_budget_period: Period,
    budget_progress: HashMap<i32, (i64, i32)>,
    editor_category: String,
    editor_limit_cents: i32,
    editor_period: Period,
    editor_target_is_expense: bool,
    current_editing: Option<i32>,
    period_offset: i32,
    editor_open: bool,
    // Transaction fields
    transactions_list: Vec<Transaction>,
    tx_account_id: i32,
    tx_amount: f32,
    tx_category: String,
    tx_custom_category: String,
    tx_date: String,
    tx_is_expense: bool,
    user_categories: Vec<String>,
    show_category_input: bool,
    // Transaction editor fields
    tx_editing_id: Option<i32>,
    tx_editor_open: bool,
    tx_editor_account_id: i32,
    tx_editor_amount: f32,
    tx_editor_category: String,
    tx_editor_date: String,
    tx_editor_is_expense: bool,
    // Transaction filter
    tx_filter_account_id: Option<i32>,
    tx_filter_category: Option<String>,
    tx_filter_start_date: String,
    tx_filter_end_date: String,
    // Transfer fields
    transfer_from_account_id: i32,
    transfer_to_account_id: i32,
    transfer_amount: f32,
    transfer_date: String,
    transfer_filter_start_date: String,
    transfer_filter_end_date: String,
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
            budgets: Vec::new(),
            selected_budget_period: Period::Monthly,
            budget_progress: HashMap::new(),
            editor_category: String::new(),
            editor_limit_cents: 0,
            editor_period: Period::Monthly,
            editor_target_is_expense: true,
            current_editing: None,
            period_offset: 0,
            editor_open: false,
            // Transaction initialization
            transactions_list: Vec::new(),
            tx_account_id: 0,
            tx_amount: 0.0,
            tx_category: DEFAULT_CATEGORIES[0].to_string(),
            tx_custom_category: String::new(),
            tx_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            tx_is_expense: true,
            user_categories: Vec::new(),
            show_category_input: false,
            // Transaction editor initialization
            tx_editing_id: None,
            tx_editor_open: false,
            tx_editor_account_id: 0,
            tx_editor_amount: 0.0,
            tx_editor_category: DEFAULT_CATEGORIES[0].to_string(),
            tx_editor_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            tx_editor_is_expense: true,
            // Transaction filter initialization
            tx_filter_account_id: None,
            tx_filter_category: None,
            tx_filter_start_date: chrono::Local::now().date_naive().with_day(1).unwrap().format("%Y-%m-%d").to_string(),
            tx_filter_end_date: chrono::Local::now().date_naive().format("%Y-%m-%d").to_string(),
            // Transfer initialization
            transfer_from_account_id: 0,
            transfer_to_account_id: 0,
            transfer_amount: 0.0,
            transfer_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            transfer_filter_start_date: chrono::Local::now().date_naive().with_day(1).unwrap().format("%Y-%m-%d").to_string(),
            transfer_filter_end_date: chrono::Local::now().date_naive().format("%Y-%m-%d").to_string(),
        }
    }

    // Budgeting display helpers
    fn show_expense_pie_chart(&mut self, ui: &mut egui::Ui) {
        // Get the current period range
        let (start, end) = Self::get_period_range(self.selected_budget_period, self.period_offset);

        // Collect negative transactions (expenses) for the current period
        let mut category_totals: Vec<(String, f32)> = {
            let mut map = std::collections::HashMap::<String, f32>::new();
            for tx in &self.transactions_list {
                // Parse date and filter by period
                if let Ok(date) = chrono::NaiveDateTime::parse_from_str(&tx.date, "%Y-%m-%d %H:%M:%S") {
                    if date >= start && date < end && tx.amount < 0.0 {
                        *map.entry(tx.category.clone()).or_insert(0.0) += tx.amount.abs();
                    }
                }
            }
            map.into_iter().collect()
        };

        if category_totals.is_empty() {
            ui.label("No expense data available for this period.");
            return;
        }

        // Sort alphabetically for stable order
        category_totals.sort_by(|a, b| a.0.cmp(&b.0));

        // Total
        let total: f32 = category_totals.iter().map(|(_, v)| *v).sum();

        ui.label("Expense Breakdown");

        // Horizontal layout: pie chart on left, legend on right
        ui.horizontal(|ui| {
            // Pie chart
            ui.vertical(|ui| {
                // Add dynamic vertical space above the pie chart
                let available_height = ui.available_height();
                ui.add_space(available_height * 0.05);

                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(160.0, 160.0),
                    egui::Sense::focusable_noninteractive(),
                );
                let painter = ui.painter();
                let center = rect.center();
                let radius = 70.0;

                // Stable colors
                let colors = [
                    egui::Color32::from_rgb(255, 99, 132),
                    egui::Color32::from_rgb(54, 162, 235),
                    egui::Color32::from_rgb(255, 206, 86),
                    egui::Color32::from_rgb(75, 192, 192),
                    egui::Color32::from_rgb(153, 102, 255),
                    egui::Color32::from_rgb(255, 159, 64),
                    egui::Color32::from_rgb(199, 199, 199),
                ];

                let mut start_angle = 0.0_f32;
                for (i, (_, amount)) in category_totals.iter().enumerate() {
                    let proportion = *amount / total;
                    let sweep = std::f32::consts::TAU * proportion;
                    let end_angle = start_angle + sweep;

                    let segments = 60;
                    let mut points = Vec::with_capacity(segments + 2);
                    points.push(center);
                    for s in 0..=segments {
                        let t = start_angle + (s as f32 / segments as f32) * sweep;
                        points.push(center + egui::vec2(t.cos(), t.sin()) * radius);
                    }
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        colors[i % colors.len()],
                        egui::Stroke::NONE,
                    ));

                    // Percentage label
                    let mid_angle = (start_angle + end_angle) / 2.0;
                    let label_pos =
                        center + egui::vec2(mid_angle.cos(), mid_angle.sin()) * (radius * 0.55);
                    painter.text(
                        label_pos,
                        egui::Align2::CENTER_CENTER,
                        format!("{:.0}%", proportion * 100.0),
                        egui::FontId::proportional(12.0),
                        egui::Color32::BLACK,
                    );

                    start_angle = end_angle;
                }
            });

            // Legend
            ui.vertical(|ui| {
                ui.label("Legend:");
                for (i, (category, amount)) in category_totals.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(14.0, 14.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, [
                            egui::Color32::from_rgb(255, 99, 132),
                            egui::Color32::from_rgb(54, 162, 235),
                            egui::Color32::from_rgb(255, 206, 86),
                            egui::Color32::from_rgb(75, 192, 192),
                            egui::Color32::from_rgb(153, 102, 255),
                            egui::Color32::from_rgb(255, 159, 64),
                            egui::Color32::from_rgb(199, 199, 199),
                        ][i % 7]);
                        ui.label(format!("{} — ${:.2}", category, amount));
                    });
                    ui.add_space(4.0);
                }
            });
        });

        ui.separator();
    }

    fn show_income_line_chart(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0); // Horizontal space to push content right

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Fine-tune this value for more/less right shift
                    ui.label("Income Progression");
                });

                // Aggregate income by month/year
                let mut income_by_month: Vec<((i32, u32), f32)> = Vec::new();

                for tx in &self.transactions_list {
                    if tx.amount > 0.0 {
                        if let Ok(date) = NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d %H:%M:%S") {
                            let key = (date.year(), date.month());
                            if let Some(entry) = income_by_month.iter_mut().find(|e| e.0 == key) {
                                entry.1 += tx.amount;
                            } else {
                                income_by_month.push((key, tx.amount));
                            }
                        }
                    }
                }

                if income_by_month.is_empty() {
                    ui.label("No income data available.");
                    return;
                }

                income_by_month.sort_by(|a, b| a.0.cmp(&b.0));

                let mut points: Vec<[f64; 2]> = Vec::new();
                let mut x_labels: Vec<PlotText> = Vec::new();

                for (idx, ((year, month), amount)) in income_by_month.iter().enumerate() {
                    points.push([idx as f64, *amount as f64]);
                    let label = NaiveDate::from_ymd_opt(*year, *month, 1)
                        .unwrap()
                        .format("%b %Y")
                        .to_string();
                    x_labels.push(PlotText::new([idx as f64, 0.0].into(), label));
                }
                let desired_size = egui::vec2(350.0, 180.0);
                ui.allocate_ui(desired_size, |ui| {
                    Plot::new("income_line_plot")
                        .allow_scroll(false)
                        .allow_zoom(false)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points.clone()).color(Color32::LIGHT_GREEN));
                            for label in &x_labels {
                                plot_ui.text(label.clone());
                            }
                        });
                });
            });
        });
    }

    // Transaction helpers
    fn load_user_transactions(&mut self) {
        if let Some(uid) = self.user_id {
            self.transactions_list = db::get_user_transactions(&mut self.conn, uid).unwrap_or_default();
        } else {
            self.transactions_list.clear();
        }
    }

    fn load_user_categories(&mut self) {
        if let Some(uid) = self.user_id {
            self.user_categories = db::get_user_categories(&mut self.conn, uid).unwrap_or_default();
            // Limit to 50 categories
            if self.user_categories.len() > 50 {
                self.user_categories.truncate(50);
            }
        } else {
            self.user_categories.clear();
        }
    }

    fn get_all_categories(&self) -> Vec<String> {
        let mut all = DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        all.extend(self.user_categories.iter().cloned());
        all.sort();
        all.dedup();
        if all.len() > 50 {
            all.truncate(50);
        }
        all
    }

    // Helper function to show date selector with dropdowns
    fn show_date_selector(ui: &mut egui::Ui, date_string: &mut String, id_prefix: &str) {
        use chrono::Datelike;
        
        // Parse current date or use today as default
        let current_date = chrono::NaiveDate::parse_from_str(date_string, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        
        let mut year = current_date.year();
        let mut month = current_date.month();
        let mut day = current_date.day();
        
        let mut changed = false;
        
        // Year dropdown
        egui::ComboBox::from_id_source(format!("{}_year", id_prefix))
            .selected_text(format!("{}", year))
            .width(70.0)
            .show_ui(ui, |ui| {
                for y in (2020..=2030).rev() {
                    if ui.selectable_value(&mut year, y, format!("{}", y)).clicked() {
                        changed = true;
                    }
                }
            });
        
        ui.label("-");
        
        // Month dropdown
        let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                          "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        egui::ComboBox::from_id_source(format!("{}_month", id_prefix))
            .selected_text(month_names[(month - 1) as usize])
            .width(50.0)
            .show_ui(ui, |ui| {
                for m in 1..=12 {
                    if ui.selectable_value(&mut month, m, month_names[(m - 1) as usize]).clicked() {
                        changed = true;
                    }
                }
            });
        
        ui.label("-");
        
        // Day dropdown
        let max_day = match month {
            2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        
        // Clamp day if needed
        if day > max_day {
            day = max_day;
            changed = true;
        }
        
        egui::ComboBox::from_id_source(format!("{}_day", id_prefix))
            .selected_text(format!("{:02}", day))
            .width(45.0)
            .show_ui(ui, |ui| {
                for d in 1..=max_day {
                    if ui.selectable_value(&mut day, d, format!("{:02}", d)).clicked() {
                        changed = true;
                    }
                }
            });
        
        // Update the date string if changed
        if changed {
            if let Some(new_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                *date_string = new_date.format("%Y-%m-%d").to_string();
            }
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
                let mut clicked_account_id: Option<i32> = None;
                
                for account in &self.accounts_list {
                    ui.horizontal(|ui| {
                        if ui.button(format!("{} - {}: ${:.2}", account.name, account.account_type, account.balance)).clicked() {
                            clicked_account_id = Some(account.id);
                        }
                    });
                }
                
                // Handle click outside the loop to avoid borrow issues
                if let Some(account_id) = clicked_account_id {
                    self.tx_filter_account_id = Some(account_id);
                    self.screen = AppState::Transactions;
                    self.load_user_transactions();
                    self.load_user_categories();
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

            if ui.button("Budgets").clicked() {
                self.screen = AppState::Budgeting;
                self.load_user_budgets();
                self.load_user_categories();
                self.compute_budget_progress(0);
            }

            if ui.button("Transactions").clicked() {
                self.tx_filter_account_id = None; // Clear account filter when navigating via button
                self.screen = AppState::Transactions;
                self.load_user_transactions();
                self.load_user_categories();
            }

            if ui.button("Transfers").clicked() {
                self.screen = AppState::Transfers;
                self.load_user_transactions();
            }
        });
    }

    // load budgets for current user
    fn load_user_budgets(&mut self) {
        if let Some(uid) = self.user_id {
            match db::get_user_budgets(&mut self.conn, uid) {
                Ok(bs) => self.budgets = bs,
                Err(_) => self.budgets.clear(),
            }
        } else {
            self.budgets.clear();
        }
    }


    fn get_period_range(period: Period, offset: i32) -> (NaiveDateTime, NaiveDateTime) {
        // minimal example — uses chrono and assumes Local::today -> convert to NaiveDateTime at midnight.
        // Replace with your preferred week-start logic.
        use chrono::{Datelike, Duration, Local, NaiveDate};
        let today = Local::now().date_naive();
        match period {
            Period::Daily => {
                let day = today + Duration::days(offset as i64);
                let start = day.and_hms_opt(0, 0, 0).unwrap();
                let end = (day + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap();
                (start, end)
            }
            Period::Weekly => {
                // start at Monday of the current week
                let weekday = today.weekday().num_days_from_monday() as i64;
                let week_start = today - Duration::days(weekday) + Duration::weeks(offset as i64);
                let start = week_start.and_hms_opt(0, 0, 0).unwrap();
                let end = (week_start + Duration::weeks(1)).and_hms_opt(0, 0, 0).unwrap();
                (start, end)
            }
            Period::Monthly => {
                let mut year = today.year();
                let mut month = today.month() as i32;
                // shift months by offset
                let total_month = month - 1 + offset;
                year += total_month.div_euclid(12);
                month = (total_month.rem_euclid(12) + 1) as u32 as i32;
                let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
                let next_month = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
                };
                (start_date.and_hms_opt(0, 0, 0).unwrap(), next_month.and_hms_opt(0, 0, 0).unwrap())
            }
            Period::Yearly => {
                let year = today.year() + offset;
                let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let end = NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                (start, end)
            }
        }
    }

    // compute progress for all loaded budgets and store into self.budget_progress
    fn compute_budget_progress(&mut self, offset: i32) {
        self.budget_progress.clear();
        if let Some(uid) = self.user_id {
            for b in &self.budgets {
                // Use each budget's own period, not the view filter period
                let budget_period = Period::from_str(&b.period);
                let (start, end) = Self::get_period_range(budget_period, offset);
                // assumes db::get_spend_for_category_period returns sum in cents (i64)
                match db::get_spend_for_category_period(&mut self.conn, uid, &b.category, start, end) {
                    Ok(spent_cents) => {
                        self.budget_progress.insert(b.id.unwrap_or(0), (spent_cents, b.limit_cents));
                    }
                    Err(_) => {
                        self.budget_progress.insert(b.id.unwrap_or(0), (0, b.limit_cents));
                    }
                }
            }
        }
    }

    fn show_budgets(&mut self, ctx: &egui::Context) {
        self.load_user_transactions();
        use egui::Color32;
        egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Budgets");

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = AppState::Dashboard;
            }

            ui.separator();

            // Time navigation controls
            if ui.button("Prev").clicked() {
                self.period_offset -= 1;
                self.compute_budget_progress(self.period_offset);
            }
            
            // Show current viewing status
            let offset_text = match self.period_offset {
                0 => "Current Period".to_string(),
                1 => "1 period ahead".to_string(),
                -1 => "1 period ago".to_string(),
                n if n > 0 => format!("{} periods ahead", n),
                n => format!("{} periods ago", n.abs()),
            };
            ui.label(egui::RichText::new(&offset_text).strong());
            
            if ui.button("Next").clicked() {
                self.period_offset += 1;
                self.compute_budget_progress(self.period_offset);
            }
            
            if self.period_offset != 0 {
                if ui.button("Reset to Current").clicked() {
                    self.period_offset = 0;
                    self.compute_budget_progress(self.period_offset);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Create Budget").clicked() {
                    // open editor in create mode
                    self.current_editing = None;
                    self.editor_category.clear();
                    self.editor_limit_cents = 0;
                    self.editor_period = Period::Monthly;
                    self.editor_target_is_expense = true;
                    self.editor_open = true;
                }
            });
        });

        ui.separator();

        // show list
        if self.budgets.is_empty() {
            ui.label("No budgets. Create one with the Create Budget button.");
        } else {
            for b in &self.budgets {
                let (raw_spent, limit) = self
                    .budget_progress
                    .get(&b.id.unwrap_or(0))
                    .cloned()
                    .unwrap_or((0, b.limit_cents));

                // Calculate date range for this budget to display
                let budget_period = crate::models::Period::from_str(&b.period);
                let (start_date, end_date) = Self::get_period_range(budget_period, self.period_offset);
                let date_range_str = format!(
                    "{} - {}",
                    start_date.format("%b %d, %Y"),
                    end_date.format("%b %d, %Y")
                );

                // Interpret spent based on target type
                let target_type = crate::models::TargetType::from_str(&b.target_type);

                let spent_for_bar: f32 = match target_type {
                    crate::models::TargetType::Expense => {
                        // expenses are stored as negative -> use absolute value
                        (raw_spent.abs()) as f32
                    }
                    crate::models::TargetType::Income => {
                        // income is positive; if somehow negative, treat as 0
                        raw_spent.max(0) as f32
                    }
                };

                let limit_f = limit.max(0) as f32;
                let ratio = if limit_f > 0.0 {
                    spent_for_bar / limit_f
                } else {
                    0.0
                };

                let progress = ratio.clamp(0.0, 5.0);

                let color = if ratio < 0.8 {
                    Color32::from_rgb(100, 200, 100)
                } else if ratio <= 1.0 {
                    Color32::from_rgb(250, 200, 50)
                } else {
                    Color32::from_rgb(220, 50, 50)
                };

                ui.horizontal(|ui| {
                    ui.label(format!("{} ({:?})", b.category, b.period));
                    
                    if ui.button("Edit").clicked() {
                        self.current_editing = b.id;
                        self.editor_category = b.category.clone();
                        self.editor_limit_cents = b.limit_cents;
                        self.editor_period = crate::models::Period::from_str(&b.period);
                        self.editor_target_is_expense = crate::models::TargetType::from_str(&b.target_type) == crate::models::TargetType::Expense;
                        self.editor_open = true;
                    }
                });
                
                ui.label(egui::RichText::new(&date_range_str).small().italics());
                
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 20.0],
                        egui::ProgressBar::new(progress).fill(color).show_percentage()
                    );
                    ui.label(format!(
                        "spent ${:.2} / ${:.2}",
                        spent_for_bar / 100.0,
                        limit_f / 100.0
                    ));
                });
                ui.separator();
            }
        }

        ui.separator();
        ui.heading("Quick Create Budget");
        
        // Category dropdown with custom option
        ui.horizontal(|ui| {
            ui.label("Category:");
            let all_categories = self.get_all_categories();
            
            egui::ComboBox::from_id_source("budget_category")
                .selected_text(&self.editor_category)
                .show_ui(ui, |ui| {
                    for cat in &all_categories {
                        ui.selectable_value(&mut self.editor_category, cat.clone(), cat);
                    }
                    ui.separator();
                    if ui.selectable_label(self.show_category_input, "+ Add New Category").clicked() {
                        self.show_category_input = true;
                    }
                });
        });

        // Custom category input
        if self.show_category_input {
            ui.horizontal(|ui| {
                ui.label("New Category:");
                ui.text_edit_singleline(&mut self.tx_custom_category);
                if ui.button("Add").clicked() {
                    if !self.tx_custom_category.is_empty() && self.user_categories.len() < 50 {
                        self.editor_category = self.tx_custom_category.clone();
                        self.tx_custom_category.clear();
                        self.show_category_input = false;
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.tx_custom_category.clear();
                    self.show_category_input = false;
                }
            });
        }
        
        ui.horizontal(|ui| {
            ui.label("Limit ($):");
            let limit_dollars = self.editor_limit_cents as f32 / 100.0;
            let mut temp_limit = limit_dollars;
            if ui.add(egui::DragValue::new(&mut temp_limit).speed(1.0).prefix("$")).changed() {
                self.editor_limit_cents = (temp_limit * 100.0) as i32;
            }
            if ui.button("Create").clicked() {
                if let Some(uid) = self.user_id {
                    let nb = crate::models::NewBudget {
                        user_id: uid,
                        category: self.editor_category.clone(),
                        limit_cents: self.editor_limit_cents,
                        period: self.editor_period.to_str().to_string(),
                        target_type: if self.editor_target_is_expense {
                            crate::models::TargetType::Expense.to_str().to_string()
                        } else {
                            crate::models::TargetType::Income.to_str().to_string()
                        },
                    };
                    if let Ok(_) = db::create_budget(&mut self.conn, nb) {
                        self.load_user_budgets();
                        self.compute_budget_progress(self.period_offset);
                        self.editor_category.clear();
                        self.editor_limit_cents = 0;
                    } else {
                        self.message = "Failed to create budget.".to_string();
                    }
                }
            }
        });

        ui.separator();
        ui.heading("Charts");

        // Use ui.columns outside of a horizontal layout
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                self.show_expense_pie_chart(ui);
            });

            cols[1].vertical(|ui| {
                self.show_income_line_chart(ui);
            });
        });
    });

    // Render editor window if requested
    if self.editor_open {
        // current_editing == Some(id) -> editing that id; None -> create
        self.show_budget_editor(ctx, self.current_editing);
    }
}

    fn show_budget_editor(&mut self, ctx: &egui::Context, editing: Option<i32>) {
        // populate editor fields from selected budget (only if fields look "empty" to avoid overwriting while typing)
        if let Some(id) = editing {
            if let Some(b) = self.budgets.iter().find(|b| b.id.unwrap_or(-1) == id) {
                if self.editor_category.is_empty() {
                    self.editor_category = b.category.clone();
                }
                if self.editor_limit_cents == 0 {
                    self.editor_limit_cents = b.limit_cents;
                }
                self.editor_period = crate::models::Period::from_str(&b.period);
                self.editor_target_is_expense = crate::models::TargetType::from_str(&b.target_type) == crate::models::TargetType::Expense;
            }
        }

        egui::Window::new("Budget Editor").resizable(false).show(ctx, |ui| {
            // Category dropdown with custom option
            ui.horizontal(|ui| {
                ui.label("Category:");
                let all_categories = self.get_all_categories();
                
                egui::ComboBox::from_id_source("budget_editor_category")
                    .selected_text(&self.editor_category)
                    .show_ui(ui, |ui| {
                        for cat in &all_categories {
                            ui.selectable_value(&mut self.editor_category, cat.clone(), cat);
                        }
                        ui.separator();
                        if ui.selectable_label(self.show_category_input, "+ Add New Category").clicked() {
                            self.show_category_input = true;
                        }
                    });
            });

            // Custom category input
            if self.show_category_input {
                ui.horizontal(|ui| {
                    ui.label("New Category:");
                    ui.text_edit_singleline(&mut self.tx_custom_category);
                    if ui.button("Add").clicked() {
                        if !self.tx_custom_category.is_empty() && self.user_categories.len() < 50 {
                            self.editor_category = self.tx_custom_category.clone();
                            self.tx_custom_category.clear();
                            self.show_category_input = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.tx_custom_category.clear();
                        self.show_category_input = false;
                    }
                });
            }

            // Limit in dollars
            ui.horizontal(|ui| {
                ui.label("Limit ($):");
                let mut limit_dollars = self.editor_limit_cents as f32 / 100.0;
                if ui.add(egui::DragValue::new(&mut limit_dollars).speed(1.0).prefix("$")).changed() {
                    self.editor_limit_cents = (limit_dollars * 100.0) as i32;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Period:");
                ui.radio_value(&mut self.editor_period, Period::Daily, "Daily");
                ui.radio_value(&mut self.editor_period, Period::Weekly, "Weekly");
                ui.radio_value(&mut self.editor_period, Period::Monthly, "Monthly");
                ui.radio_value(&mut self.editor_period, Period::Yearly, "Yearly");
            });

            ui.checkbox(&mut self.editor_target_is_expense, "Expense budget");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(uid) = self.user_id {
                let nb = crate::models::NewBudget {
                    user_id: uid,
                    category: self.editor_category.clone(),
                    limit_cents: self.editor_limit_cents,
                    period: self.editor_period.to_str().to_string(),
                    target_type: if self.editor_target_is_expense {
                        crate::models::TargetType::Expense.to_str().to_string()
                    } else {
                        crate::models::TargetType::Income.to_str().to_string()
                    },
                };                        let res = match editing {
                            Some(id) => db::update_budget(&mut self.conn, id, nb).map(|_| ()),
                            None => db::create_budget(&mut self.conn, nb).map(|_| ()),
                        };

                        if res.is_ok() {
                            self.load_user_budgets();
                            self.compute_budget_progress(0);
                            // clear quick-editor
                            self.editor_category.clear();
                            self.editor_limit_cents = 0;
                            self.editor_open = false; // add this
                            self.current_editing = None; // add this
                        } else {
                            self.message = format!("Failed to save budget: {:?}", res.err());
                        }
                    } else {
                        self.message = "Not logged in.".to_string();
                    }
                }

                if editing.is_some() {
                    if ui.button("Delete").clicked() {
                        if let Some(id) = editing {
                            if db::delete_budget(&mut self.conn, id).is_ok() {
                                self.load_user_budgets();
                                self.compute_budget_progress(0);
                            } else {
                                self.message = "Failed to delete budget.".to_string();
                            }
                        }
                    }
                }

                if ui.button("Close").clicked() {
                    self.editor_open = false;
                    self.current_editing = None;
                }
            });
        });
    }

    fn show_transactions(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Transactions");

            ui.horizontal(|ui| {
                if ui.button("Back to Dashboard").clicked() {
                    self.screen = AppState::Dashboard;
                }
            });

            ui.separator();
            ui.heading("Add New Transaction");

            // Account selector
            ui.horizontal(|ui| {
                ui.label("Account:");
                egui::ComboBox::from_id_source("tx_account_selector")
                    .selected_text(
                        self.accounts_list
                            .iter()
                            .find(|a| a.id == self.tx_account_id)
                            .map(|a| a.name.as_str())
                            .unwrap_or("Select Account")
                    )
                    .show_ui(ui, |ui| {
                        for account in &self.accounts_list {
                            ui.selectable_value(&mut self.tx_account_id, account.id, &account.name);
                        }
                    });
            });

            // Amount and type
            ui.horizontal(|ui| {
                ui.label("Amount:");
                ui.add(egui::DragValue::new(&mut self.tx_amount).speed(1.0).prefix("$"));
                ui.checkbox(&mut self.tx_is_expense, "Expense");
            });

            // Category selector
            ui.horizontal(|ui| {
                ui.label("Category:");
                let all_categories = self.get_all_categories();
                
                egui::ComboBox::from_id_source("tx_category_selector")
                    .selected_text(&self.tx_category)
                    .show_ui(ui, |ui| {
                        for cat in &all_categories {
                            ui.selectable_value(&mut self.tx_category, cat.clone(), cat);
                        }
                        ui.separator();
                        if ui.selectable_label(self.show_category_input, "+ Add New Category").clicked() {
                            self.show_category_input = true;
                        }
                    });
            });

            // Custom category input
            if self.show_category_input {
                ui.horizontal(|ui| {
                    ui.label("New Category:");
                    ui.text_edit_singleline(&mut self.tx_custom_category);
                    if ui.button("Add").clicked() {
                        if !self.tx_custom_category.is_empty() && self.user_categories.len() < 50 {
                            self.tx_category = self.tx_custom_category.clone();
                            self.tx_custom_category.clear();
                            self.show_category_input = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.tx_custom_category.clear();
                        self.show_category_input = false;
                    }
                });
            }

            // Date
            ui.horizontal(|ui| {
                ui.label("Date:");
                Self::show_date_selector(ui, &mut self.tx_date, "tx_date");
            });

            // Submit button
            if ui.button("Add Transaction").clicked() {
                if let Some(_uid) = self.user_id {
                    if self.tx_account_id > 0 && self.tx_amount > 0.0 {
                        // Find the selected account
                        let account_opt = self.accounts_list.iter().find(|a| a.id == self.tx_account_id);
                        if let Some(account) = account_opt {
                            let amount = if self.tx_is_expense { -self.tx_amount } else { self.tx_amount };
                            // Check for sufficient funds if expense
                            if self.tx_is_expense && account.balance < self.tx_amount {
                                self.message = "Insufficient funds for this expense.".to_string();
                            } else {
                                let date_time = format!("{} 00:00:00", self.tx_date);

                                match db::create_transaction(
                                    &mut self.conn,
                                    self.tx_account_id,
                                    0, // contact_id - not used
                                    amount,
                                    self.tx_category.clone(),
                                    date_time,
                                ) {
                                    Ok(_) => {
                                        self.message = "Transaction added successfully!".to_string();
                                        self.load_user_transactions();
                                        self.load_user_categories();
                                        self.load_user_budgets();
                                        self.compute_budget_progress(self.period_offset);
                                        // Reload accounts to show updated balance
                                        if let Some(uid) = self.user_id {
                                            self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                                        }
                                        // Reset form
                                        self.tx_amount = 0.0;
                                        self.tx_is_expense = true;
                                    }
                                    Err(e) => {
                                        self.message = format!("Failed to add transaction: {:?}", e);
                                    }
                                }
                            }
                        } else {
                            self.message = "Selected account not found.".to_string();
                        }
                    } else {
                        self.message = "Please select an account and enter a positive amount.".to_string();
                    }
                }
            }

            ui.separator();
            ui.label(&self.message);

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Export CSV").clicked() {
                    let file_path = format!(
                        "transactions_{}_to_{}.csv",
                        self.tx_filter_start_date.replace("-", ""),
                        self.tx_filter_end_date.replace("-", "")
                    );
                    // Collect filtered transactions (same filter as below)
                    let filtered_transactions: Vec<&Transaction> = self.transactions_list
                        .iter()
                        .filter(|tx| {
                            let account_match = if let Some(filter_id) = self.tx_filter_account_id {
                                tx.user_account_id == filter_id
                            } else {
                                true
                            };
                            let category_match = if let Some(ref filter_cat) = self.tx_filter_category {
                                &tx.category == filter_cat
                            } else {
                                true
                            };
                            let date_match = {
                                let tx_date = &tx.date[..10];
                                let start_match = if !self.tx_filter_start_date.is_empty() {
                                    tx_date >= self.tx_filter_start_date.as_str()
                                } else {
                                    true
                                };
                                let end_match = if !self.tx_filter_end_date.is_empty() {
                                    tx_date <= self.tx_filter_end_date.as_str()
                                } else {
                                    true
                                };
                                start_match && end_match
                            };
                            account_match && category_match && date_match
                        })
                        .collect();

                    let mut wtr = Writer::from_path(&file_path);
                    match wtr {
                        Ok(mut writer) => {
                            let _ = writer.write_record(&[
                                "id", "account_id", "contact_id", "amount", "category", "date", "amount_cents", "balance_after"
                            ]);
                            for tx in &filtered_transactions {
                                let _ = writer.write_record(&[
                                    tx.id.to_string(),
                                    tx.user_account_id.to_string(),
                                    tx.contact_id.to_string(),
                                    tx.amount.to_string(),
                                    tx.category.clone(),
                                    tx.date.clone(),
                                    tx.amount_cents.to_string(),
                                    tx.balance_after.to_string(),
                                ]);
                            }
                            let _ = writer.flush();
                            self.message = format!("Exported {} transactions to {}", filtered_transactions.len(), file_path);
                        }
                        Err(e) => {
                            self.message = format!("Failed to export CSV: {}", e);
                        }
                    }
                }
            });            
            
            // Filter section
            ui.horizontal(|ui| {
                ui.heading("Transaction History");
            });
            
            ui.horizontal(|ui| {
                ui.label("Filter by Account:");
                egui::ComboBox::from_id_source("tx_filter_account")
                    .selected_text(
                        if let Some(filter_id) = self.tx_filter_account_id {
                            self.accounts_list
                                .iter()
                                .find(|a| a.id == filter_id)
                                .map(|a| a.name.as_str())
                                .unwrap_or("All Accounts")
                        } else {
                            "All Accounts"
                        }
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.tx_filter_account_id, None, "All Accounts");
                        ui.separator();
                        for account in &self.accounts_list {
                            ui.selectable_value(&mut self.tx_filter_account_id, Some(account.id), &account.name);
                        }
                    });
                
                ui.separator();
                ui.label("Filter by Category:");
                let all_categories = self.get_all_categories();
                egui::ComboBox::from_id_source("tx_filter_category")
                    .selected_text(
                        if let Some(ref filter_cat) = self.tx_filter_category {
                            filter_cat.as_str()
                        } else {
                            "All Categories"
                        }
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.tx_filter_category, None, "All Categories");
                        ui.separator();
                        for cat in &all_categories {
                            ui.selectable_value(&mut self.tx_filter_category, Some(cat.clone()), cat);
                        }
                    });
            });
            
            // Date filter row
            ui.horizontal(|ui| {
                ui.label("Filter by Date Range:");
                
                // From date selector
                ui.label("From:");
                Self::show_date_selector(ui, &mut self.tx_filter_start_date, "tx_filter_start");
                
                ui.separator();
                
                // To date selector
                ui.label("To:");
                Self::show_date_selector(ui, &mut self.tx_filter_end_date, "tx_filter_end");
                
                if ui.button("This Month").clicked() {
                    let today = chrono::Local::now().date_naive();
                    self.tx_filter_start_date = today.with_day(1).unwrap().format("%Y-%m-%d").to_string();
                    self.tx_filter_end_date = today.format("%Y-%m-%d").to_string();
                }
                
                if ui.button("All Time").clicked() {
                    self.tx_filter_start_date = "2000-01-01".to_string();
                    self.tx_filter_end_date = "2099-12-31".to_string();
                }
            });

            // Transaction list with filtering
            let mut tx_to_edit: Option<Transaction> = None;
            let mut tx_to_delete: Option<i32> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                let filtered_transactions: Vec<&Transaction> = self.transactions_list
                    .iter()
                    .filter(|tx| {
                        let account_match = if let Some(filter_id) = self.tx_filter_account_id {
                            tx.user_account_id == filter_id
                        } else {
                            true
                        };
                        
                        let category_match = if let Some(ref filter_cat) = self.tx_filter_category {
                            &tx.category == filter_cat
                        } else {
                            true
                        };
                        
                        // Date filtering
                        let date_match = {
                            let tx_date = &tx.date[..10]; // Extract YYYY-MM-DD from "YYYY-MM-DD HH:MM:SS"
                            
                            let start_match = if !self.tx_filter_start_date.is_empty() {
                                tx_date >= self.tx_filter_start_date.as_str()
                            } else {
                                true
                            };
                            
                            let end_match = if !self.tx_filter_end_date.is_empty() {
                                tx_date <= self.tx_filter_end_date.as_str()
                            } else {
                                true
                            };
                            
                            start_match && end_match
                        };
                        
                        account_match && category_match && date_match
                    })
                    .collect();

                if filtered_transactions.is_empty() {
                    ui.label("No transactions match the filter.");
                } else {
                    for tx in filtered_transactions {
                        let account_name = self.accounts_list
                            .iter()
                            .find(|a| a.id == tx.user_account_id)
                            .map(|a| a.name.as_str())
                            .unwrap_or("Unknown");
                        
                        let color = if tx.amount >= 0.0 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else {
                            egui::Color32::from_rgb(200, 50, 50)
                        };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, format!("${:.2}", tx.amount));
                            ui.label(format!("| {} | {} | {}", tx.category, account_name, tx.date));
                            ui.label(format!("| Balance: ${:.2}", tx.balance_after));
                            
                            if ui.button("Edit").clicked() {
                                tx_to_edit = Some(tx.clone());
                            }
                            
                            if ui.button("Delete").clicked() {
                                tx_to_delete = Some(tx.id);
                            }
                        });
                        ui.separator();
                    }
                }
            });

            // Handle edit action
            if let Some(tx) = tx_to_edit {
                self.tx_editing_id = Some(tx.id);
                self.tx_editor_open = true;
                self.tx_editor_account_id = tx.user_account_id;
                self.tx_editor_amount = tx.amount.abs();
                self.tx_editor_category = tx.category.clone();
                // Extract just the date part (YYYY-MM-DD) from the full datetime string
                self.tx_editor_date = tx.date[..10].to_string();
                self.tx_editor_is_expense = tx.amount < 0.0;
            }

            // Handle delete action
            if let Some(tx_id) = tx_to_delete {
                if let Err(e) = db::delete_transaction(&mut self.conn, tx_id) {
                    self.message = format!("Error deleting transaction: {}", e);
                } else {
                    self.load_user_transactions();
                    self.load_user_budgets();
                    self.compute_budget_progress(self.period_offset);

                    // Reload accounts to show updated balance
                    if let Some(uid) = self.user_id {
                        self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                    }
                    self.message = "Transaction deleted successfully".to_string();
                }
            }
        });
        
        // Render transaction editor if open
        if self.tx_editor_open {
            self.show_transaction_editor(ctx);
        }
    }

    fn show_transaction_editor(&mut self, ctx: &egui::Context) {
        let mut should_close = false;

        egui::Window::new("Edit Transaction")
            .collapsible(false)
            .show(ctx, |ui| {
                // Account selector
                ui.horizontal(|ui| {
                    ui.label("Account:");
                    egui::ComboBox::from_id_source("tx_editor_account_selector")
                        .selected_text(
                            self.accounts_list
                                .iter()
                                .find(|a| a.id == self.tx_editor_account_id)
                                .map(|a| a.name.as_str())
                                .unwrap_or("Select Account")
                        )
                        .show_ui(ui, |ui| {
                            for acc in &self.accounts_list {
                                ui.selectable_value(&mut self.tx_editor_account_id, acc.id, &acc.name);
                            }
                        });
                });

                // Category selector
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    let all_categories = self.get_all_categories();
                    egui::ComboBox::from_id_source("tx_editor_category_selector")
                        .selected_text(&self.tx_editor_category)
                        .show_ui(ui, |ui| {
                            for cat in &all_categories {
                                ui.selectable_value(&mut self.tx_editor_category, cat.clone(), cat);
                            }
                        });
                });

                // Amount
                ui.horizontal(|ui| {
                    ui.label("Amount:");
                    ui.add(egui::DragValue::new(&mut self.tx_editor_amount).speed(0.1));
                    ui.checkbox(&mut self.tx_editor_is_expense, "Expense");
                });

                // Date
                ui.horizontal(|ui| {
                    ui.label("Date:");
                    Self::show_date_selector(ui, &mut self.tx_editor_date, "tx_editor_date");
                });

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if let Some(tx_id) = self.tx_editing_id {
                            // Always use positive value for amount
                            let entered_amount = self.tx_editor_amount.abs();
                            let amount = if self.tx_editor_is_expense {
                                -entered_amount
                            } else {
                                entered_amount
                            };

                            if entered_amount <= 0.0 {
                                self.message = "Amount must be positive.".to_string();
                            } else {
                                // Format date with time for database
                                let date_time = format!("{} 00:00:00", self.tx_editor_date);
                                
                                match db::update_transaction(
                                    &mut self.conn,
                                    tx_id,
                                    self.tx_editor_account_id,
                                    amount,
                                    self.tx_editor_category.clone(),
                                    date_time,
                                ) {
                                    Ok(_) => {
                                        self.message = "Transaction updated successfully".to_string();
                                        self.load_user_transactions();
                                        self.load_user_budgets();
                                        self.compute_budget_progress(self.period_offset);

                                        // Reload accounts to show updated balance
                                        if let Some(uid) = self.user_id {
                                            self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                                        }
                                        should_close = true;
                                    }
                                    Err(e) => {
                                        self.message = format!("Error updating transaction: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.tx_editor_open = false;
            self.tx_editing_id = None;
        }
    }

    fn show_transfers(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Account Transfers");

            ui.horizontal(|ui| {
                if ui.button("Back to Dashboard").clicked() {
                    self.screen = AppState::Dashboard;
                }
            });

            ui.separator();
            ui.heading("Create Transfer");

            // From Account selector
            ui.horizontal(|ui| {
                ui.label("From Account:");
                egui::ComboBox::from_id_source("transfer_from_account")
                    .selected_text(
                        self.accounts_list
                            .iter()
                            .find(|a| a.id == self.transfer_from_account_id)
                            .map(|a| format!("{} (${:.2})", a.name, a.balance))
                            .unwrap_or_else(|| "Select Account".to_string())
                    )
                    .show_ui(ui, |ui| {
                        for account in &self.accounts_list {
                            ui.selectable_value(
                                &mut self.transfer_from_account_id,
                                account.id,
                                format!("{} (${:.2})", account.name, account.balance)
                            );
                        }
                    });
            });

            // To Account selector
            ui.horizontal(|ui| {
                ui.label("To Account:");
                egui::ComboBox::from_id_source("transfer_to_account")
                    .selected_text(
                        self.accounts_list
                            .iter()
                            .find(|a| a.id == self.transfer_to_account_id)
                            .map(|a| format!("{} (${:.2})", a.name, a.balance))
                            .unwrap_or_else(|| "Select Account".to_string())
                    )
                    .show_ui(ui, |ui| {
                        for account in &self.accounts_list {
                            // Don't allow selecting the same account as source
                            if account.id != self.transfer_from_account_id {
                                ui.selectable_value(
                                    &mut self.transfer_to_account_id,
                                    account.id,
                                    format!("{} (${:.2})", account.name, account.balance)
                                );
                            }
                        }
                    });
            });

            // Amount
            ui.horizontal(|ui| {
                ui.label("Amount:");
                ui.add(egui::DragValue::new(&mut self.transfer_amount).speed(1.0).prefix("$"));
            });

            // Date
            ui.horizontal(|ui| {
                ui.label("Date:");
                Self::show_date_selector(ui, &mut self.transfer_date, "transfer_date");
            });

            // Transfer button
            if ui.button("Execute Transfer").clicked() {
                if self.transfer_from_account_id > 0 
                    && self.transfer_to_account_id > 0 
                    && self.transfer_from_account_id != self.transfer_to_account_id
                    && self.transfer_amount > 0.0 {
                    
                    let date_time = format!("{} 00:00:00", self.transfer_date);
                    
                    match db::create_transfer(
                        &mut self.conn,
                        self.transfer_from_account_id,
                        self.transfer_to_account_id,
                        self.transfer_amount,
                        date_time,
                    ) {
                        Ok(_) => {
                            self.message = format!(
                                "Transfer of ${:.2} completed successfully!",
                                self.transfer_amount
                            );
                            self.load_user_transactions();
                            self.load_user_budgets();
                            self.compute_budget_progress(self.period_offset);
                            
                            // Reload accounts to show updated balances
                            if let Some(uid) = self.user_id {
                                self.accounts_list = db::get_user_accounts(&mut self.conn, uid).unwrap_or_default();
                            }
                            
                            // Reset form
                            self.transfer_amount = 0.0;
                        }
                        Err(e) => {
                            self.message = format!("Transfer failed: {:?}", e);
                        }
                    }
                } else if self.transfer_from_account_id == self.transfer_to_account_id {
                    self.message = "Cannot transfer to the same account.".to_string();
                } else {
                    self.message = "Please select both accounts and enter a positive amount.".to_string();
                }
            }

            ui.separator();
            ui.label(&self.message);

            ui.separator();
            ui.heading("Transfer History");
            
            // Date filter for transfers
            ui.horizontal(|ui| {
                ui.label("Filter by Date Range:");
                
                // From date selector
                ui.label("From:");
                Self::show_date_selector(ui, &mut self.transfer_filter_start_date, "transfer_filter_start");
                
                ui.separator();
                
                // To date selector
                ui.label("To:");
                Self::show_date_selector(ui, &mut self.transfer_filter_end_date, "transfer_filter_end");
                
                if ui.button("This Month").clicked() {
                    let today = chrono::Local::now().date_naive();
                    self.transfer_filter_start_date = today.with_day(1).unwrap().format("%Y-%m-%d").to_string();
                    self.transfer_filter_end_date = today.format("%Y-%m-%d").to_string();
                }
                
                if ui.button("All Time").clicked() {
                    self.transfer_filter_start_date = "2000-01-01".to_string();
                    self.transfer_filter_end_date = "2099-12-31".to_string();
                }
            });

            // Show only Transfer transactions
            egui::ScrollArea::vertical().show(ui, |ui| {
                let transfer_transactions: Vec<&Transaction> = self.transactions_list
                    .iter()
                    .filter(|tx| {
                        if tx.category != "Transfer" {
                            return false;
                        }
                        
                        // Date filtering
                        let tx_date = &tx.date[..10]; // Extract YYYY-MM-DD
                        
                        let start_match = if !self.transfer_filter_start_date.is_empty() {
                            tx_date >= self.transfer_filter_start_date.as_str()
                        } else {
                            true
                        };
                        
                        let end_match = if !self.transfer_filter_end_date.is_empty() {
                            tx_date <= self.transfer_filter_end_date.as_str()
                        } else {
                            true
                        };
                        
                        start_match && end_match
                    })
                    .collect();

                if transfer_transactions.is_empty() {
                    ui.label("No transfers yet.");
                } else {
                    for tx in transfer_transactions {
                        let account_name = self.accounts_list
                            .iter()
                            .find(|a| a.id == tx.user_account_id)
                            .map(|a| a.name.as_str())
                            .unwrap_or("Unknown");
                        
                        let color = if tx.amount >= 0.0 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else {
                            egui::Color32::from_rgb(200, 50, 50)
                        };

                        let transfer_type = if tx.amount >= 0.0 { "TO" } else { "FROM" };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, format!("${:.2}", tx.amount.abs()));
                            ui.label(format!("| {} {} | {} | Balance: ${:.2}", 
                                transfer_type, account_name, tx.date, tx.balance_after));
                        });
                        ui.separator();
                    }
                }
            });
        });
    }

}

impl eframe::App for FinancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            AppState::Login => self.show_login(ctx),
            AppState::Register => self.show_register(ctx),
            AppState::Dashboard => self.show_dashboard(ctx),
            AppState::Budgeting => self.show_budgets(ctx),
            AppState::Transactions => self.show_transactions(ctx),
            AppState::Transfers => self.show_transfers(ctx),
        }
    }
}
