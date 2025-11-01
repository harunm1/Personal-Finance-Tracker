mod app;

use eframe::NativeOptions;
use app::FinancerApp;

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "FinanceR",
        options,
        Box::new(|_cc| Box::new(FinancerApp::default())),
    );
}