#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use financer::finance_calculations::{
        real_rate,
        future_value,
        present_value,
        present_value_of_dated_cash_flows,
        future_value_of_dated_cash_flows,
        price_bond,
        mortgage_monthly_payment,
        mortgage_amortization_schedule_with_frequency,
        PaymentFrequency,
        simple_interest_future_value,
        compound_interest_future_value_with_contributions,
        ContributionFrequency,
        CompoundingFrequency,
    };

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

    #[test]
    fn test_simple_interest_future_value() {
        let fv = simple_interest_future_value(1000.0, 0.05, 3.0);
        assert!((fv - 1150.0).abs() < 1e-9);
    }

    #[test]
    fn test_compound_interest_matches_future_value_without_contributions() {
        let pv = 1234.56;
        let annual_rate = 0.07;
        let years = 8.0;
        let fv_expected = future_value(pv, annual_rate, years, 12);
        let fv = compound_interest_future_value_with_contributions(
            pv,
            0.0,
            ContributionFrequency::Monthly,
            annual_rate,
            CompoundingFrequency::Monthly,
            years,
        );
        assert!((fv - fv_expected).abs() < 1e-9);
    }

    #[test]
    fn test_compound_interest_with_monthly_contributions_matches_closed_form_monthly() {
        let initial: f64 = 1000.0;
        let pmt: f64 = 100.0;
        let annual_rate: f64 = 0.06;
        let years: f64 = 2.0;

        let i: f64 = annual_rate / 12.0;
        let n: f64 = (years * 12.0).round();
        let fv_expected: f64 = initial * (1.0 + i).powf(n)
            + pmt * (((1.0 + i).powf(n) - 1.0) / i);

        let fv = compound_interest_future_value_with_contributions(
            initial,
            pmt,
            ContributionFrequency::Monthly,
            annual_rate,
            CompoundingFrequency::Monthly,
            years,
        );

        assert!((fv - fv_expected).abs() < 1e-6);
    }

    #[test]
    fn test_compound_interest_zero_rate_is_principal_plus_contributions() {
        let initial = 500.0;
        let pmt = 25.0;
        let years = 2.0;
        let expected = initial + pmt * 24.0;

        let fv = compound_interest_future_value_with_contributions(
            initial,
            pmt,
            ContributionFrequency::Monthly,
            0.0,
            CompoundingFrequency::Monthly,
            years,
        );

        assert!((fv - expected).abs() < 1e-9);
    }
}