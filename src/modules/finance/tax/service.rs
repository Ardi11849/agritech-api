use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::finance::tax::dto::{
    PayTaxRequest, TaxAlertStatus, TaxCurrentPeriodDto, TaxDashboardData, TaxDashboardQuery,
    TaxReportItem, TaxSettingsDto, TaxStatus, UpdateTaxSettingsRequest,
};
use crate::modules::finance::tax::repository::TaxRepository;

pub fn calculate_tax_amount(total_revenue: Decimal, rate: Decimal) -> Decimal {
    round_to_two_decimals(total_revenue * rate)
}

pub fn calculate_tax_deadline(month: i32, year: i32) -> Result<NaiveDate, AppError> {
    if !(1..=12).contains(&month) {
        return Err(AppError::BadRequest(
            "Bulan periode pajak harus berada di antara 1 dan 12".into(),
        ));
    }
    if !(2000..=2100).contains(&year) {
        return Err(AppError::BadRequest(
            "Tahun periode pajak harus berada di antara 2000 dan 2100".into(),
        ));
    }

    let deadline = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 15)
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 15)
    };

    deadline
        .ok_or_else(|| AppError::Internal("Format tanggal jatuh tempo pajak tidak valid".into()))
}

pub fn evaluate_tax_alert(
    status: TaxStatus,
    deadline_date: NaiveDate,
    today: NaiveDate,
) -> (i64, TaxAlertStatus) {
    let days_remaining = (deadline_date - today).num_days();

    let alert_status = match status {
        TaxStatus::Paid => TaxAlertStatus::Paid,
        TaxStatus::Unpaid => {
            if days_remaining < 0 {
                TaxAlertStatus::Overdue
            } else if days_remaining <= 7 {
                TaxAlertStatus::WarningH7
            } else {
                TaxAlertStatus::Normal
            }
        }
    };

    (days_remaining, alert_status)
}

#[async_trait]
pub trait TaxService: Send + Sync {
    async fn get_dashboard(&self, query: &TaxDashboardQuery) -> Result<TaxDashboardData, AppError>;
    async fn pay_tax(&self, req: PayTaxRequest, actor: &str) -> Result<TaxReportItem, AppError>;
    async fn update_settings(
        &self,
        req: UpdateTaxSettingsRequest,
        actor: &str,
    ) -> Result<TaxSettingsDto, AppError>;
    async fn delete_report(&self, id: &str) -> Result<(), AppError>;
}

pub struct TaxServiceImpl {
    repo: Arc<dyn TaxRepository>,
}

impl TaxServiceImpl {
    pub fn new(repo: Arc<dyn TaxRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TaxService for TaxServiceImpl {
    async fn get_dashboard(&self, query: &TaxDashboardQuery) -> Result<TaxDashboardData, AppError> {
        let today = query.date.unwrap_or_else(today_wib);
        let month = query.month.unwrap_or_else(|| today.month() as i32);
        let year = query.year.unwrap_or_else(|| today.year());

        let deadline_date = calculate_tax_deadline(month, year)?;
        let settings = self.repo.get_settings().await?;
        let (realized_revenue, sales_count) = self
            .repo
            .get_realized_revenue_for_period(month, year)
            .await?;
        let existing_report = self.repo.get_report_by_period(month, year).await?;

        let (
            report_id,
            total_revenue,
            tax_rate,
            tax_rate_percent,
            estimated_tax_amount,
            status,
            payment_date,
            billing_code,
            notes,
        ) = match existing_report {
            Some(rep) => (
                Some(rep.id),
                rep.total_revenue,
                rep.tax_rate,
                rep.tax_rate_percent,
                rep.tax_amount,
                rep.status,
                rep.payment_date,
                rep.billing_code,
                rep.notes,
            ),
            None => {
                let tax_rate = settings.rate;
                let tax_rate_percent = settings.rate_percent;
                let estimated_tax_amount = calculate_tax_amount(realized_revenue, tax_rate);
                (
                    None,
                    realized_revenue,
                    tax_rate,
                    tax_rate_percent,
                    estimated_tax_amount,
                    TaxStatus::Unpaid,
                    None,
                    "".to_string(),
                    "".to_string(),
                )
            }
        };

        let (days_remaining, alert_status) = evaluate_tax_alert(status, deadline_date, today);

        let current_period = TaxCurrentPeriodDto {
            report_id,
            period_month: month,
            period_year: year,
            total_revenue,
            realized_sales_count: sales_count,
            tax_rate,
            tax_rate_percent,
            estimated_tax_amount,
            status,
            deadline_date,
            days_remaining,
            alert_status,
            payment_date,
            billing_code,
            notes,
        };

        let tax_history = self.repo.get_tax_history().await?;

        Ok(TaxDashboardData {
            settings,
            current_period,
            tax_history,
        })
    }

    async fn pay_tax(&self, req: PayTaxRequest, actor: &str) -> Result<TaxReportItem, AppError> {
        if !(1..=12).contains(&req.period_month) {
            return Err(AppError::BadRequest(
                "Bulan periode pajak harus berada di antara 1 dan 12".into(),
            ));
        }
        if !(2000..=2100).contains(&req.period_year) {
            return Err(AppError::BadRequest(
                "Tahun periode pajak harus berada di antara 2000 dan 2100".into(),
            ));
        }
        if req.billing_code.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Kode billing / NTPN wajib diisi untuk pelunasan pajak".into(),
            ));
        }
        if req.billing_code.trim().len() > 50 {
            return Err(AppError::BadRequest(
                "Kode billing / NTPN maksimal 50 karakter".into(),
            ));
        }
        if let Some(notes) = &req.notes {
            if notes.len() > 500 {
                return Err(AppError::BadRequest(
                    "Catatan pembayaran pajak maksimal 500 karakter".into(),
                ));
            }
        }

        let mut req = req;
        req.billing_code = req.billing_code.trim().to_string();
        if let Some(notes) = req.notes {
            req.notes = Some(notes.trim().to_string());
        }

        let settings = self.repo.get_settings().await?;
        let (realized_revenue, _) = self
            .repo
            .get_realized_revenue_for_period(req.period_month, req.period_year)
            .await?;

        let tax_amount = calculate_tax_amount(realized_revenue, settings.rate);

        self.repo
            .upsert_payment(req, realized_revenue, settings.rate, tax_amount, actor)
            .await
    }

    async fn update_settings(
        &self,
        req: UpdateTaxSettingsRequest,
        actor: &str,
    ) -> Result<TaxSettingsDto, AppError> {
        let scheme = req.scheme_name.trim();
        if scheme.is_empty() {
            return Err(AppError::BadRequest(
                "Nama skema perpajakan wajib diisi".into(),
            ));
        }
        if scheme.len() > 150 {
            return Err(AppError::BadRequest(
                "Nama skema perpajakan maksimal 150 karakter".into(),
            ));
        }

        if req.rate < Decimal::ZERO || req.rate > Decimal::ONE {
            return Err(AppError::BadRequest(
                "Tarif pajak harus berada di antara 0.0000 dan 1.0000 (0% - 100%)".into(),
            ));
        }

        if let Some(desc) = &req.description {
            if desc.len() > 500 {
                return Err(AppError::BadRequest(
                    "Deskripsi pengaturan pajak maksimal 500 karakter".into(),
                ));
            }
        }

        let mut req = req;
        req.rate = req.rate.round_dp(4);

        self.repo.update_settings(req, actor).await
    }

    async fn delete_report(&self, id: &str) -> Result<(), AppError> {
        self.repo.delete_report(id).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_tax_simulasi_2_calculation_and_deadline() {
        // Simulasi 2 PRD:
        // Revenue = 20,000,000, Rate = 0.0050 (0.5%)
        // Period: Month 12, Year 2026
        // Evaluation date: 2027-01-10 -> Days remaining: 5 -> WARNING_H7
        let rev = d("20000000.00");
        let rate = d("0.0050");
        let tax_amount = calculate_tax_amount(rev, rate);
        assert_eq!(tax_amount, d("100000.00"));

        let deadline = calculate_tax_deadline(12, 2026).expect("Deadline must succeed");
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2027, 1, 15).unwrap());

        let today = NaiveDate::from_ymd_opt(2027, 1, 10).unwrap();
        let (days, alert) = evaluate_tax_alert(TaxStatus::Unpaid, deadline, today);
        assert_eq!(days, 5);
        assert_eq!(alert, TaxAlertStatus::WarningH7);
    }

    #[test]
    fn test_tax_alert_statuses_and_rollover() {
        let deadline = calculate_tax_deadline(5, 2026).expect("May deadline is June 15");
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());

        // Normal (more than 7 days)
        let day_normal = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let (days, alert) = evaluate_tax_alert(TaxStatus::Unpaid, deadline, day_normal);
        assert_eq!(days, 14);
        assert_eq!(alert, TaxAlertStatus::Normal);

        // Warning H7 (exactly 7 days)
        let day_h7 = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let (days, alert) = evaluate_tax_alert(TaxStatus::Unpaid, deadline, day_h7);
        assert_eq!(days, 7);
        assert_eq!(alert, TaxAlertStatus::WarningH7);

        // Overdue (passed deadline)
        let day_overdue = NaiveDate::from_ymd_opt(2026, 6, 16).unwrap();
        let (days, alert) = evaluate_tax_alert(TaxStatus::Unpaid, deadline, day_overdue);
        assert_eq!(days, -1);
        assert_eq!(alert, TaxAlertStatus::Overdue);

        // Paid (regardless of date)
        let (_, alert) = evaluate_tax_alert(TaxStatus::Paid, deadline, day_overdue);
        assert_eq!(alert, TaxAlertStatus::Paid);
    }

    #[test]
    fn test_tax_rate_boundary_and_invalid_period() {
        assert!(calculate_tax_deadline(0, 2026).is_err());
        assert!(calculate_tax_deadline(13, 2026).is_err());
        assert!(calculate_tax_deadline(6, 1999).is_err());
        assert!(calculate_tax_deadline(6, 2101).is_err());
    }
}
