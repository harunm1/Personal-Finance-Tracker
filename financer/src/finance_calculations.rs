/// Financial calculation utilities: present/future value, cash flows, bonds, and mortgages.

use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde::{Deserialize, Serialize};

pub fn real_rate(nominal_rate: f64, inflation_rate: f64) -> f64 {
    (1.0 + nominal_rate) / (1.0 + inflation_rate) - 1.0
}

pub fn future_value(present_value: f64, annual_rate: f64, years: f64, compounding_per_year: u32) -> f64 {
    let n_periods = years * compounding_per_year as f64;
    let rate_per_period = annual_rate / compounding_per_year as f64;
    present_value * (1.0 + rate_per_period).powf(n_periods)
}

pub fn present_value(future_value: f64, annual_rate: f64, years: f64, compounding_per_year: u32) -> f64 {
    let n_periods = years * compounding_per_year as f64;
    let rate_per_period = annual_rate / compounding_per_year as f64;
    future_value / (1.0 + rate_per_period).powf(n_periods)
}

/// Present value of dated cash flows using a continuous-time approximation.
///
/// - `cash_flows`: (date, amount)
/// - `valuation_date`: date at which PV is computed
/// - `real_annual_rate`: effective real annual discount rate (e.g. from `real_rate`)
pub fn present_value_of_dated_cash_flows(
    cash_flows: &[(NaiveDate, f64)],
    valuation_date: NaiveDate,
    real_annual_rate: f64,
) -> f64 {
    if real_annual_rate == 0.0 {
        return cash_flows.iter().map(|(_, cf)| *cf).sum();
    }

    cash_flows
        .iter()
        .map(|(date, cf)| {
            let days = (*date - valuation_date).num_days() as f64;
            let t_years = days / 365.0;
            cf / (1.0 + real_annual_rate).powf(t_years)
        })
        .sum()
}

/// Future value of dated cash flows at a given horizon date.
///
/// - `horizon_date`: the date at which FV is computed
pub fn future_value_of_dated_cash_flows(
    cash_flows: &[(NaiveDate, f64)],
    horizon_date: NaiveDate,
    real_annual_rate: f64,
) -> f64 {
    if real_annual_rate == 0.0 {
        return cash_flows.iter().map(|(_, cf)| *cf).sum();
    }

    cash_flows
        .iter()
        .map(|(date, cf)| {
            let days = (horizon_date - *date).num_days() as f64;
            let t_years = days / 365.0;
            cf * (1.0 + real_annual_rate).powf(t_years)
        })
        .sum()
}

pub fn price_bond(
    face_value: f64,
    coupon_rate: f64,
    yield_to_maturity: f64,
    years_to_maturity: f64,
    payments_per_year: u32,
) -> f64 {
    let total_payments = (years_to_maturity * payments_per_year as f64).round() as u32;
    let coupon_per_period = face_value * coupon_rate / payments_per_year as f64;
    let yield_per_period = yield_to_maturity / payments_per_year as f64;

    let mut price = 0.0;
    for t in 1..=total_payments {
        let discount_factor = (1.0 + yield_per_period).powi(t as i32);
        price += coupon_per_period / discount_factor;
    }

    let discount_factor = (1.0 + yield_per_period).powi(total_payments as i32);
    price += face_value / discount_factor;

    price
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MortgagePayment {
    pub period: u32,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentFrequency {
    Monthly,
    BiWeekly,
    Weekly,
    /// Accelerated weekly: use a weekly schedule but a payment equal to
    /// (monthly payment * 12 / 52), resulting in more paid per year.
    AcceleratedWeekly,
}

impl PaymentFrequency {
    pub fn payments_per_year(self) -> f64 {
        match self {
            PaymentFrequency::Monthly => 12.0,
            PaymentFrequency::BiWeekly => 26.0,
            PaymentFrequency::Weekly | PaymentFrequency::AcceleratedWeekly => 52.0,
        }
    }
}

/// Generic mortgage payment for a given payment frequency.
pub fn mortgage_payment_with_frequency(
    principal: f64,
    annual_rate: f64,
    years: u32,
    frequency: PaymentFrequency,
) -> f64 {
    let periods_per_year = frequency.payments_per_year();
    let total_periods = years as f64 * periods_per_year;

    if annual_rate == 0.0 {
        return principal / total_periods;
    }

    // For accelerated weekly, base the payment on the standard
    // monthly payment and convert to an equivalent weekly amount.
    if matches!(frequency, PaymentFrequency::AcceleratedWeekly) {
        let monthly = mortgage_monthly_payment(principal, annual_rate, years);
        return monthly * 12.0 / 52.0;
    }

    let rate_per_period = annual_rate / periods_per_year;
    let numerator = rate_per_period * principal;
    let denominator = 1.0 - (1.0 + rate_per_period).powf(-total_periods);
    numerator / denominator
}

/// Amortization schedule for a given frequency (including accelerated weekly).
pub fn mortgage_amortization_schedule_with_frequency(
    principal: f64,
    annual_rate: f64,
    years: u32,
    frequency: PaymentFrequency,
) -> Vec<MortgagePayment> {
    let payment = mortgage_payment_with_frequency(principal, annual_rate, years, frequency);
    let periods_per_year = frequency.payments_per_year();
    let rate_per_period = if annual_rate == 0.0 {
        0.0
    } else {
        annual_rate / periods_per_year
    };
    let total_periods = (years as f64 * periods_per_year).round() as u32;

    let mut balance = principal;
    let mut schedule = Vec::with_capacity(total_periods as usize);

    for period in 1..=total_periods {
        let interest = if annual_rate == 0.0 { 0.0 } else { balance * rate_per_period };
        let mut principal_component = payment - interest;

        if principal_component < 0.0 {
            principal_component = 0.0;
        }

        if principal_component > balance {
            principal_component = balance;
        }

        balance -= principal_component;

        schedule.push(MortgagePayment {
            period,
            payment: principal_component + interest,
            principal: principal_component,
            interest,
            remaining_balance: balance.max(0.0),
        });

        if balance <= 0.0 {
            break;
        }
    }

    schedule
}

/// Backwards-compatible monthly payment wrapper.
pub fn mortgage_monthly_payment(principal: f64, annual_rate: f64, years: u32) -> f64 {
    mortgage_payment_with_frequency(principal, annual_rate, years, PaymentFrequency::Monthly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_real_rate() {
        let r = real_rate(0.05, 0.02);
        assert!((r - 0.029411).abs() < 1e-4);
    }

    #[test]
    fn test_present_and_future_value_inverse() {
        let pv = 1000.0;
        let fv = future_value(pv, 0.05, 10.0, 1);
        let pv_back = present_value(fv, 0.05, 10.0, 1);
        assert!((pv - pv_back).abs() < 1e-6);
    }

    #[test]
    fn test_dated_cash_flows_pv_and_fv() {
        let d0 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let d1 = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        let flows = vec![(d0, 100.0), (d1, 100.0)];
        let real_r = 0.03;
        let pv = present_value_of_dated_cash_flows(&flows, d0, real_r);
        let fv = future_value_of_dated_cash_flows(&flows, d1, real_r);
        assert!(pv > 0.0);
        assert!(fv > pv);
    }

    #[test]
    fn test_price_bond_par_at_equal_coupon_and_yield() {
        let price = price_bond(1000.0, 0.05, 0.05, 10.0, 2);
        assert!((price - 1000.0).abs() < 1.0); // allow small numerical difference
    }

    #[test]
    fn test_mortgage_payment_and_schedule() {
        let principal = 300_000.0;
        let annual_rate = 0.05;
        let years = 30;
        let monthly = mortgage_monthly_payment(principal, annual_rate, years);
        assert!(monthly > 0.0);

        let schedule = mortgage_amortization_schedule_with_frequency(
            principal,
            annual_rate,
            years,
            PaymentFrequency::Monthly,
        );
        assert!(!schedule.is_empty());
        let last = schedule.last().unwrap();
        assert!(last.remaining_balance.abs() < 1.0);
    }

    #[test]
    fn test_accelerated_weekly_pays_off_faster_than_standard_weekly() {
        let principal = 300_000.0;
        let annual_rate = 0.05;
        let years = 30;

        let sched_weekly = mortgage_amortization_schedule_with_frequency(
            principal,
            annual_rate,
            years,
            PaymentFrequency::Weekly,
        );
        let total_weekly: f64 = sched_weekly.iter().map(|p| p.payment).sum();

        let sched_accel = mortgage_amortization_schedule_with_frequency(
            principal,
            annual_rate,
            years,
            PaymentFrequency::AcceleratedWeekly,
        );
        let total_accel: f64 = sched_accel.iter().map(|p| p.payment).sum();

        // Accelerated weekly should result in less total interest, hence lower total paid.
        assert!(total_accel < total_weekly);
    }
}