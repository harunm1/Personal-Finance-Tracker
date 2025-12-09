mod app;
mod db;
mod models;
mod schema;
mod finance_calculations;

use eframe::NativeOptions;
use app::FinancerApp;

fn main() -> eframe::Result<()> {
    dotenv::dotenv().ok();
    let conn = db::establish_connection();
    let options = NativeOptions::default();
    eframe::run_native("FinanceR", options, Box::new(|_cc| Box::new(FinancerApp::new(conn))))
}
