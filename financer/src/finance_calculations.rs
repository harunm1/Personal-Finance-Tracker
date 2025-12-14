/// Financial calculation utilities: present/future value, cash flows, bonds, and mortgages.
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};

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

    if matches!(frequency, PaymentFrequency::AcceleratedWeekly) {
        let monthly = mortgage_monthly_payment(principal, annual_rate, years);
        return monthly * 12.0 / 52.0;
    }

    let rate_per_period = annual_rate / periods_per_year;
    let numerator = rate_per_period * principal;
    let denominator = 1.0 - (1.0 + rate_per_period).powf(-total_periods);
    numerator / denominator
}

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

pub fn mortgage_monthly_payment(principal: f64, annual_rate: f64, years: u32) -> f64 {
    mortgage_payment_with_frequency(principal, annual_rate, years, PaymentFrequency::Monthly)
}
