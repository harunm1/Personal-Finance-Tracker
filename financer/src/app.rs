use eframe::egui;
use eframe::egui::Ui;
use crate::db;
use diesel::sqlite::SqliteConnection;
use diesel::result::{Error, DatabaseErrorKind};
use crate::models::{Account, Transaction};
use crate::models::{Budget, Period};
use crate::finance_calculations::{
    real_rate,
    future_value,
    present_value,
    present_value_of_dated_cash_flows,
    future_value_of_dated_cash_flows,
    price_bond,
    PaymentFrequency,
    mortgage_payment_with_frequency,
    mortgage_amortization_schedule_with_frequency,
};
use egui_plot::{Plot, Line, PlotPoints, Text as PlotText};
use egui::epaint::Shape;
use std::collections::HashMap;
use std::f32::consts::TAU;
use chrono::{NaiveDateTime,NaiveDate,Datelike};
use eframe::egui::Color32;

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
    CashflowTools,
    BondTools,
    MortgageTools,
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
    // Transfer fields
    transfer_from_account_id: i32,
    transfer_to_account_id: i32,
    transfer_amount: f32,
    transfer_date: String,
    // Cash-flow tools state (A/B scenarios, dated entries)
    cf_a_lines: String,
    cf_b_lines: String,
    cf_nominal_rate_percent: f32,
    cf_inflation_rate_percent: f32,
    cf_valuation_date: String,
    cf_horizon_date: String,
    cf_a_pv: Option<f64>,
    cf_a_fv: Option<f64>,
    cf_b_pv: Option<f64>,
    cf_b_fv: Option<f64>,
    cf_error: Option<String>,
    // Helper to quickly add repetitive monthly flows
    cf_gen_amount: f32,
    cf_gen_start_date: String,
    cf_gen_months: i32,
    // Single-amount PV/FV helper state
    cf_single_amount: f32,
    cf_single_years: f32,
    cf_single_comp_per_year: i32,
    cf_single_pv: Option<f64>,
    cf_single_fv: Option<f64>,
    // Bond tools state (A/B scenarios)
    bond_a_face_value: f32,
    bond_a_coupon_percent: f32,
    bond_a_ytm_percent: f32,
    bond_a_years_to_maturity: f32,
    bond_a_payments_per_year: i32,
    bond_a_price: Option<f64>,
    bond_a_error: Option<String>,
    bond_b_face_value: f32,
    bond_b_coupon_percent: f32,
    bond_b_ytm_percent: f32,
    bond_b_years_to_maturity: f32,
    bond_b_payments_per_year: i32,
    bond_b_price: Option<f64>,
    bond_b_error: Option<String>,
    // Mortgage tools state (A/B scenarios)
    mort_a_principal: f32,
    mort_a_annual_rate_percent: f32,
    mort_a_years: f32,
    mort_a_monthly_payment: Option<f64>,
    mort_a_total_paid: Option<f64>,
    mort_a_total_interest: Option<f64>,
    mort_a_error: Option<String>,
    mort_b_principal: f32,
    mort_b_annual_rate_percent: f32,
    mort_b_years: f32,
    mort_b_monthly_payment: Option<f64>,
    mort_b_total_paid: Option<f64>,
    mort_b_total_interest: Option<f64>,
    mort_b_error: Option<String>,
    mort_a_frequency: PaymentFrequency,
    mort_b_frequency: PaymentFrequency,
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
            // Transfer initialization
            transfer_from_account_id: 0,
            transfer_to_account_id: 0,
            transfer_amount: 0.0,
            transfer_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            // Cash-flow tools initialization
            cf_a_lines: String::new(),
            cf_b_lines: String::new(),
            cf_nominal_rate_percent: 5.0,
            cf_inflation_rate_percent: 2.0,
            cf_valuation_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            cf_horizon_date: String::new(),
            cf_a_pv: None,
            cf_a_fv: None,
            cf_b_pv: None,
            cf_b_fv: None,
            cf_error: None,
            cf_gen_amount: 500.0,
            cf_gen_start_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            cf_gen_months: 5,
            cf_single_amount: 1000.0,
            cf_single_years: 10.0,
            cf_single_comp_per_year: 1,
            cf_single_pv: None,
            cf_single_fv: None,
            // Bond tools initialization
            bond_a_face_value: 1000.0,
            bond_a_coupon_percent: 5.0,
            bond_a_ytm_percent: 5.0,
            bond_a_years_to_maturity: 10.0,
            bond_a_payments_per_year: 2,
            bond_a_price: None,
            bond_a_error: None,
            bond_b_face_value: 1000.0,
            bond_b_coupon_percent: 5.0,
            bond_b_ytm_percent: 5.0,
            bond_b_years_to_maturity: 10.0,
            bond_b_payments_per_year: 2,
            bond_b_price: None,
            bond_b_error: None,
            // Mortgage tools initialization
            mort_a_principal: 300_000.0,
            mort_a_annual_rate_percent: 5.0,
            mort_a_years: 30.0,
            mort_a_monthly_payment: None,
            mort_a_total_paid: None,
            mort_a_total_interest: None,
            mort_a_error: None,
            mort_b_principal: 300_000.0,
            mort_b_annual_rate_percent: 4.5,
            mort_b_years: 30.0,
            mort_b_monthly_payment: None,
            mort_b_total_paid: None,
            mort_b_total_interest: None,
            mort_b_error: None,
            mort_a_frequency: PaymentFrequency::Monthly,
            mort_b_frequency: PaymentFrequency::Monthly,
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

            if ui.button("Budgets").clicked() {
                self.screen = AppState::Budgeting;
                self.load_user_budgets();
                self.load_user_categories();
                self.compute_budget_progress(0);
            }

            if ui.button("Transactions").clicked() {
                self.screen = AppState::Transactions;
                self.load_user_transactions();
                self.load_user_categories();
            }

            if ui.button("Transfers").clicked() {
                self.screen = AppState::Transfers;
                self.load_user_transactions();
            }

            ui.separator();
            ui.heading("Planning Tools");
            ui.horizontal(|ui| {
                if ui.button("Cash Flow Tools").clicked() {
                    self.screen = AppState::CashflowTools;
                }
                if ui.button("Bond Tools").clicked() {
                    self.screen = AppState::BondTools;
                }
                if ui.button("Mortgage Tools").clicked() {
                    self.screen = AppState::MortgageTools;
                }
            });
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
                ui.text_edit_singleline(&mut self.tx_date);
                ui.label("(YYYY-MM-DD)");
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
                        
                        account_match && category_match
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
                self.tx_editor_date = tx.date.clone();
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
                    ui.text_edit_singleline(&mut self.tx_editor_date);
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
                                match db::update_transaction(
                                    &mut self.conn,
                                    tx_id,
                                    self.tx_editor_account_id,
                                    amount,
                                    self.tx_editor_category.clone(),
                                    self.tx_editor_date.clone(),
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
                ui.text_edit_singleline(&mut self.transfer_date);
                ui.label("(YYYY-MM-DD)");
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

            // Show only Transfer transactions
            egui::ScrollArea::vertical().show(ui, |ui| {
                let transfer_transactions: Vec<&Transaction> = self.transactions_list
                    .iter()
                    .filter(|tx| tx.category == "Transfer")
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

impl FinancerApp {
    fn show_cashflow_tools(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cash Flow Tools");

            ui.horizontal(|ui| {
                if ui.button("Back to Dashboard").clicked() {
                    self.screen = AppState::Dashboard;
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Nominal annual interest (%):");
                ui.add(egui::DragValue::new(&mut self.cf_nominal_rate_percent).speed(0.1));
                ui.label("Inflation (%):");
                ui.add(egui::DragValue::new(&mut self.cf_inflation_rate_percent).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Valuation date (YYYY-MM-DD):");
                ui.text_edit_singleline(&mut self.cf_valuation_date);
                ui.label("Horizon date (optional, YYYY-MM-DD):");
                ui.text_edit_singleline(&mut self.cf_horizon_date);
            });

            ui.separator();
            ui.heading("Single Amount Present/Future Value");
            ui.horizontal(|ui| {
                ui.label("Amount today ($):");
                ui.add(egui::DragValue::new(&mut self.cf_single_amount).speed(10.0));
                ui.label("Years:");
                ui.add(egui::DragValue::new(&mut self.cf_single_years).clamp_range(0.0..=100.0).speed(0.5));
            });
            ui.horizontal(|ui| {
                ui.label("Compounding per year:");
                ui.add(egui::DragValue::new(&mut self.cf_single_comp_per_year).clamp_range(1..=365));

                if ui.button("Compute single PV & FV").clicked() {
                    let nominal = self.cf_nominal_rate_percent as f64 / 100.0;
                    let inflation = self.cf_inflation_rate_percent as f64 / 100.0;
                    let real_annual = real_rate(nominal, inflation);

                    let years = self.cf_single_years.max(0.0) as f64;
                    let comp_per_year = self.cf_single_comp_per_year.max(1) as u32;
                    let pv_input = self.cf_single_amount as f64;

                    let fv = future_value(pv_input, real_annual, years, comp_per_year);
                    let pv_back = present_value(fv, real_annual, years, comp_per_year);

                    self.cf_single_fv = Some(fv);
                    self.cf_single_pv = Some(pv_back);
                }
            });

            if let Some(fv) = self.cf_single_fv {
                ui.label(format!("Future value of ${:.2} = ${:.2}", self.cf_single_amount, fv));
            }
            if let Some(pv) = self.cf_single_pv {
                ui.label(format!("Check: discounting FV back gives PV ≈ ${:.2}", pv));
            }

            ui.separator();
            ui.heading("Quick Monthly Series Generator");
            ui.horizontal(|ui| {
                ui.label("Amount per month ($):");
                ui.add(egui::DragValue::new(&mut self.cf_gen_amount).speed(10.0));
                ui.label("Start date (YYYY-MM-DD):");
                ui.text_edit_singleline(&mut self.cf_gen_start_date);
                ui.label("Months:");
                ui.add(egui::DragValue::new(&mut self.cf_gen_months).clamp_range(1..=600));
            });
            ui.horizontal(|ui| {
                if ui.button("Add series to Cash Flow A").clicked() {
                    if let Ok(start) = NaiveDate::parse_from_str(&self.cf_gen_start_date, "%Y-%m-%d") {
                        for i in 0..self.cf_gen_months.max(0) {
                            let date = start
                                .checked_add_months(chrono::Months::new(i as u32))
                                .unwrap_or(start);
                            self.cf_a_lines.push_str(&format!("{} {:.2}\n", date.format("%Y-%m-%d"), self.cf_gen_amount));
                        }
                    }
                }
                if ui.button("Add series to Cash Flow B").clicked() {
                    if let Ok(start) = NaiveDate::parse_from_str(&self.cf_gen_start_date, "%Y-%m-%d") {
                        for i in 0..self.cf_gen_months.max(0) {
                            let date = start
                                .checked_add_months(chrono::Months::new(i as u32))
                                .unwrap_or(start);
                            self.cf_b_lines.push_str(&format!("{} {:.2}\n", date.format("%Y-%m-%d"), self.cf_gen_amount));
                        }
                    }
                }
            });

            ui.separator();

            ui.columns(2, |cols| {
                let ui_a = &mut cols[0];
                ui_a.heading("Cash Flow A");
                ui_a.label("Enter one cash flow per line: 'YYYY-MM-DD amount'");
                ui_a.text_edit_multiline(&mut self.cf_a_lines);

                let ui_b = &mut cols[1];
                ui_b.heading("Cash Flow B");
                ui_b.label("Enter one cash flow per line: 'YYYY-MM-DD amount'");
                ui_b.text_edit_multiline(&mut self.cf_b_lines);
            });

            if ui.button("Compute PV & FV for A and B").clicked() {
                let parsed_date = NaiveDate::parse_from_str(&self.cf_valuation_date, "%Y-%m-%d");
                if let Err(_) = parsed_date {
                    self.cf_error = Some("Invalid valuation date format.".to_string());
                } else {
                    let valuation_date = parsed_date.unwrap();
                    let horizon_date = if self.cf_horizon_date.trim().is_empty() {
                        None
                    } else {
                        NaiveDate::parse_from_str(&self.cf_horizon_date, "%Y-%m-%d").ok()
                    };

                    let nominal = self.cf_nominal_rate_percent as f64 / 100.0;
                    let inflation = self.cf_inflation_rate_percent as f64 / 100.0;
                    let real_annual = real_rate(nominal, inflation);

                    fn parse_lines(lines: &str) -> Result<Vec<(NaiveDate, f64)>, ()> {
                        let mut out = Vec::new();
                        for line in lines.lines() {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }
                            let mut parts = trimmed.split_whitespace();
                            let date_str = parts.next().ok_or(())?;
                            let amt_str = parts.next().ok_or(())?;
                            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| ())?;
                            let amount: f64 = amt_str.parse().map_err(|_| ())?;
                            out.push((date, amount));
                        }
                        Ok(out)
                    }

                    match (parse_lines(&self.cf_a_lines), parse_lines(&self.cf_b_lines)) {
                        (Ok(a_flows), Ok(b_flows)) => {
                            if a_flows.is_empty() && b_flows.is_empty() {
                                self.cf_error = Some("Please enter at least one cash flow in A or B.".to_string());
                            } else {
                                let horizon_a = horizon_date
                                    .or_else(|| a_flows.iter().map(|(d, _)| *d).max())
                                    .unwrap_or(valuation_date);
                                let horizon_b = horizon_date
                                    .or_else(|| b_flows.iter().map(|(d, _)| *d).max())
                                    .unwrap_or(valuation_date);

                                self.cf_a_pv = if a_flows.is_empty() {
                                    None
                                } else {
                                    Some(present_value_of_dated_cash_flows(&a_flows, valuation_date, real_annual))
                                };
                                self.cf_a_fv = if a_flows.is_empty() {
                                    None
                                } else {
                                    Some(future_value_of_dated_cash_flows(&a_flows, horizon_a, real_annual))
                                };

                                self.cf_b_pv = if b_flows.is_empty() {
                                    None
                                } else {
                                    Some(present_value_of_dated_cash_flows(&b_flows, valuation_date, real_annual))
                                };
                                self.cf_b_fv = if b_flows.is_empty() {
                                    None
                                } else {
                                    Some(future_value_of_dated_cash_flows(&b_flows, horizon_b, real_annual))
                                };

                                self.cf_error = None;
                            }
                        }
                        _ => {
                            self.cf_error = Some("Could not parse one of the cash flow lists. Use 'YYYY-MM-DD amount'.".to_string());
                        }
                    }
                }
            }

            if let Some(ref err) = self.cf_error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();
            ui.heading("Results and Comparison");

            ui.columns(2, |cols| {
                let ui_a = &mut cols[0];
                ui_a.heading("Cash Flow A");
                if let Some(pv) = self.cf_a_pv {
                    ui_a.label(format!("Present value: ${:.2}", pv));
                } else {
                    ui_a.label("Present value: n/a");
                }
                if let Some(fv) = self.cf_a_fv {
                    ui_a.label(format!("Future value: ${:.2}", fv));
                } else {
                    ui_a.label("Future value: n/a");
                }

                let ui_b = &mut cols[1];
                ui_b.heading("Cash Flow B");
                if let Some(pv) = self.cf_b_pv {
                    ui_b.label(format!("Present value: ${:.2}", pv));
                } else {
                    ui_b.label("Present value: n/a");
                }
                if let Some(fv) = self.cf_b_fv {
                    ui_b.label(format!("Future value: ${:.2}", fv));
                } else {
                    ui_b.label("Future value: n/a");
                }
            });

            if let (Some(pv_a), Some(pv_b)) = (self.cf_a_pv, self.cf_b_pv) {
                if pv_a > pv_b {
                    ui.label(format!("Cash Flow A has higher present value by ${:.2}", pv_a - pv_b));
                } else if pv_b > pv_a {
                    ui.label(format!("Cash Flow B has higher present value by ${:.2}", pv_b - pv_a));
                }
            }
        });
    }

    fn show_bond_tools(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bond Tools");

            ui.horizontal(|ui| {
                if ui.button("Back to Dashboard").clicked() {
                    self.screen = AppState::Dashboard;
                }
            });

            ui.separator();

            ui.columns(2, |cols| {
                let ui_a = &mut cols[0];
                ui_a.heading("Bond A");
                ui_a.horizontal(|ui| {
                    ui.label("Face value ($):");
                    ui.add(egui::DragValue::new(&mut self.bond_a_face_value).speed(10.0));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Coupon rate (%):");
                    ui.add(egui::DragValue::new(&mut self.bond_a_coupon_percent).speed(0.1));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Yield to maturity (%):");
                    ui.add(egui::DragValue::new(&mut self.bond_a_ytm_percent).speed(0.1));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Years to maturity:");
                    ui.add(egui::DragValue::new(&mut self.bond_a_years_to_maturity).speed(0.5));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Payments per year:");
                    ui.add(egui::DragValue::new(&mut self.bond_a_payments_per_year).clamp_range(1..=12));
                });
                if let Some(ref err) = self.bond_a_error {
                    ui_a.colored_label(egui::Color32::RED, err);
                }
                if let Some(price) = self.bond_a_price {
                    ui_a.label(format!("Price: ${:.2}", price));
                }

                let ui_b = &mut cols[1];
                ui_b.heading("Bond B");
                ui_b.horizontal(|ui| {
                    ui.label("Face value ($):");
                    ui.add(egui::DragValue::new(&mut self.bond_b_face_value).speed(10.0));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Coupon rate (%):");
                    ui.add(egui::DragValue::new(&mut self.bond_b_coupon_percent).speed(0.1));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Yield to maturity (%):");
                    ui.add(egui::DragValue::new(&mut self.bond_b_ytm_percent).speed(0.1));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Years to maturity:");
                    ui.add(egui::DragValue::new(&mut self.bond_b_years_to_maturity).speed(0.5));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Payments per year:");
                    ui.add(egui::DragValue::new(&mut self.bond_b_payments_per_year).clamp_range(1..=12));
                });
                if let Some(ref err) = self.bond_b_error {
                    ui_b.colored_label(egui::Color32::RED, err);
                }
                if let Some(price) = self.bond_b_price {
                    ui_b.label(format!("Price: ${:.2}", price));
                }
            });

            if ui.button("Price Both Bonds").clicked() {
                // Bond A
                if self.bond_a_face_value <= 0.0 || self.bond_a_years_to_maturity <= 0.0 {
                    self.bond_a_price = None;
                    self.bond_a_error = Some("Bond A: Face value and years must be positive.".to_string());
                } else {
                    let face = self.bond_a_face_value as f64;
                    let coupon = self.bond_a_coupon_percent as f64 / 100.0;
                    let ytm = self.bond_a_ytm_percent as f64 / 100.0;
                    let years = self.bond_a_years_to_maturity as f64;
                    let pays = self.bond_a_payments_per_year.max(1) as u32;
                    self.bond_a_price = Some(price_bond(face, coupon, ytm, years, pays));
                    self.bond_a_error = None;
                }

                // Bond B
                if self.bond_b_face_value <= 0.0 || self.bond_b_years_to_maturity <= 0.0 {
                    self.bond_b_price = None;
                    self.bond_b_error = Some("Bond B: Face value and years must be positive.".to_string());
                } else {
                    let face = self.bond_b_face_value as f64;
                    let coupon = self.bond_b_coupon_percent as f64 / 100.0;
                    let ytm = self.bond_b_ytm_percent as f64 / 100.0;
                    let years = self.bond_b_years_to_maturity as f64;
                    let pays = self.bond_b_payments_per_year.max(1) as u32;
                    self.bond_b_price = Some(price_bond(face, coupon, ytm, years, pays));
                    self.bond_b_error = None;
                }
            }

            ui.separator();
            ui.heading("Comparison");
            if let (Some(pa), Some(pb)) = (self.bond_a_price, self.bond_b_price) {
                if pa > pb {
                    ui.label(format!("Bond A is more expensive by ${:.2}", pa - pb));
                } else if pb > pa {
                    ui.label(format!("Bond B is more expensive by ${:.2}", pb - pa));
                } else {
                    ui.label("Both bonds have the same price.");
                }
            } else {
                ui.label("Price both bonds to see a comparison.");
            }
        });
    }

    fn show_mortgage_tools(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mortgage Tools");

            ui.horizontal(|ui| {
                if ui.button("Back to Dashboard").clicked() {
                    self.screen = AppState::Dashboard;
                }
            });

            ui.separator();

            ui.columns(2, |cols| {
                let ui_a = &mut cols[0];
                ui_a.heading("Mortgage A");
                ui_a.horizontal(|ui| {
                    ui.label("Principal ($):");
                    ui.add(egui::DragValue::new(&mut self.mort_a_principal).speed(1000.0));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Annual rate (%):");
                    ui.add(egui::DragValue::new(&mut self.mort_a_annual_rate_percent).speed(0.1));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Years:");
                    ui.add(egui::DragValue::new(&mut self.mort_a_years).speed(1.0));
                });
                ui_a.horizontal(|ui| {
                    ui.label("Payment frequency:");
                    egui::ComboBox::from_id_source("mort_a_freq")
                        .selected_text(match self.mort_a_frequency {
                            PaymentFrequency::Monthly => "Monthly",
                            PaymentFrequency::BiWeekly => "Bi-weekly",
                            PaymentFrequency::Weekly => "Weekly",
                            PaymentFrequency::AcceleratedWeekly => "Accelerated weekly",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.mort_a_frequency, PaymentFrequency::Monthly, "Monthly");
                            ui.selectable_value(&mut self.mort_a_frequency, PaymentFrequency::BiWeekly, "Bi-weekly");
                            ui.selectable_value(&mut self.mort_a_frequency, PaymentFrequency::Weekly, "Weekly");
                            ui.selectable_value(
                                &mut self.mort_a_frequency,
                                PaymentFrequency::AcceleratedWeekly,
                                "Accelerated weekly",
                            );
                        });
                });
                if let Some(ref err) = self.mort_a_error {
                    ui_a.colored_label(egui::Color32::RED, err);
                }
                if let Some(p) = self.mort_a_monthly_payment {
                    ui_a.label(format!("Payment per period: ${:.2}", p));
                }
                if let (Some(t), Some(i)) = (self.mort_a_total_paid, self.mort_a_total_interest) {
                    ui_a.label(format!("Total paid: ${:.2}", t));
                    ui_a.label(format!("Total interest: ${:.2}", i));
                }

                let ui_b = &mut cols[1];
                ui_b.heading("Mortgage B");
                ui_b.horizontal(|ui| {
                    ui.label("Principal ($):");
                    ui.add(egui::DragValue::new(&mut self.mort_b_principal).speed(1000.0));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Annual rate (%):");
                    ui.add(egui::DragValue::new(&mut self.mort_b_annual_rate_percent).speed(0.1));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Years:");
                    ui.add(egui::DragValue::new(&mut self.mort_b_years).speed(1.0));
                });
                ui_b.horizontal(|ui| {
                    ui.label("Payment frequency:");
                    egui::ComboBox::from_id_source("mort_b_freq")
                        .selected_text(match self.mort_b_frequency {
                            PaymentFrequency::Monthly => "Monthly",
                            PaymentFrequency::BiWeekly => "Bi-weekly",
                            PaymentFrequency::Weekly => "Weekly",
                            PaymentFrequency::AcceleratedWeekly => "Accelerated weekly",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.mort_b_frequency, PaymentFrequency::Monthly, "Monthly");
                            ui.selectable_value(&mut self.mort_b_frequency, PaymentFrequency::BiWeekly, "Bi-weekly");
                            ui.selectable_value(&mut self.mort_b_frequency, PaymentFrequency::Weekly, "Weekly");
                            ui.selectable_value(
                                &mut self.mort_b_frequency,
                                PaymentFrequency::AcceleratedWeekly,
                                "Accelerated weekly",
                            );
                        });
                });
                if let Some(ref err) = self.mort_b_error {
                    ui_b.colored_label(egui::Color32::RED, err);
                }
                if let Some(p) = self.mort_b_monthly_payment {
                    ui_b.label(format!("Payment per period: ${:.2}", p));
                }
                if let (Some(t), Some(i)) = (self.mort_b_total_paid, self.mort_b_total_interest) {
                    ui_b.label(format!("Total paid: ${:.2}", t));
                    ui_b.label(format!("Total interest: ${:.2}", i));
                }
            });

            if ui.button("Compute Both Mortgages").clicked() {
                // Mortgage A
                if self.mort_a_principal <= 0.0 || self.mort_a_years <= 0.0 {
                    self.mort_a_monthly_payment = None;
                    self.mort_a_total_paid = None;
                    self.mort_a_total_interest = None;
                    self.mort_a_error = Some("Mortgage A: Principal and years must be positive.".to_string());
                } else {
                    let principal = self.mort_a_principal as f64;
                    let annual = self.mort_a_annual_rate_percent as f64 / 100.0;
                    let years_u32 = self.mort_a_years.round().max(1.0) as u32;
                    let pmt = mortgage_payment_with_frequency(
                        principal,
                        annual,
                        years_u32,
                        self.mort_a_frequency,
                    );
                    let sched = mortgage_amortization_schedule_with_frequency(
                        principal,
                        annual,
                        years_u32,
                        self.mort_a_frequency,
                    );
                    let total: f64 = sched.iter().map(|p| p.payment).sum();
                    let interest = total - principal;
                    self.mort_a_monthly_payment = Some(pmt);
                    self.mort_a_total_paid = Some(total);
                    self.mort_a_total_interest = Some(interest);
                    self.mort_a_error = None;
                }

                // Mortgage B
                if self.mort_b_principal <= 0.0 || self.mort_b_years <= 0.0 {
                    self.mort_b_monthly_payment = None;
                    self.mort_b_total_paid = None;
                    self.mort_b_total_interest = None;
                    self.mort_b_error = Some("Mortgage B: Principal and years must be positive.".to_string());
                } else {
                    let principal = self.mort_b_principal as f64;
                    let annual = self.mort_b_annual_rate_percent as f64 / 100.0;
                    let years_u32 = self.mort_b_years.round().max(1.0) as u32;
                    let pmt = mortgage_payment_with_frequency(
                        principal,
                        annual,
                        years_u32,
                        self.mort_b_frequency,
                    );
                    let sched = mortgage_amortization_schedule_with_frequency(
                        principal,
                        annual,
                        years_u32,
                        self.mort_b_frequency,
                    );
                    let total: f64 = sched.iter().map(|p| p.payment).sum();
                    let interest = total - principal;
                    self.mort_b_monthly_payment = Some(pmt);
                    self.mort_b_total_paid = Some(total);
                    self.mort_b_total_interest = Some(interest);
                    self.mort_b_error = None;
                }
            }

            ui.separator();
            ui.heading("Comparison");
            if let (Some(pmt_a), Some(pmt_b)) = (self.mort_a_monthly_payment, self.mort_b_monthly_payment) {
                if (pmt_a - pmt_b).abs() > f64::EPSILON {
                    if pmt_a < pmt_b {
                        ui.label(format!("Mortgage A has a lower payment per period by ${:.2}", pmt_b - pmt_a));
                    } else {
                        ui.label(format!("Mortgage B has a lower payment per period by ${:.2}", pmt_a - pmt_b));
                    }
                }
            }
            if let (Some(int_a), Some(int_b)) = (self.mort_a_total_interest, self.mort_b_total_interest) {
                if int_a < int_b {
                    ui.label(format!("Mortgage A pays ${:.2} less interest over the term", int_b - int_a));
                } else if int_b < int_a {
                    ui.label(format!("Mortgage B pays ${:.2} less interest over the term", int_a - int_b));
                }
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
            AppState::Budgeting => self.show_budgets(ctx),
            AppState::Transactions => self.show_transactions(ctx),
            AppState::Transfers => self.show_transfers(ctx),
            AppState::CashflowTools => self.show_cashflow_tools(ctx),
            AppState::BondTools => self.show_bond_tools(ctx),
            AppState::MortgageTools => self.show_mortgage_tools(ctx),
        }
    }
}
