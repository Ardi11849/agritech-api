use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::modules::finance::tax::dto::{
    PayTaxRequest, TaxReportItem, TaxSettingsDto, TaxStatus, UpdateTaxSettingsRequest,
};

#[async_trait]
pub trait TaxRepository: Send + Sync {
    async fn get_settings(&self) -> Result<TaxSettingsDto, AppError>;
    async fn update_settings(
        &self,
        req: UpdateTaxSettingsRequest,
        actor: &str,
    ) -> Result<TaxSettingsDto, AppError>;
    async fn get_realized_revenue_for_period(
        &self,
        month: i32,
        year: i32,
    ) -> Result<(Decimal, i64), AppError>;
    async fn get_report_by_period(
        &self,
        month: i32,
        year: i32,
    ) -> Result<Option<TaxReportItem>, AppError>;
    async fn upsert_payment(
        &self,
        req: PayTaxRequest,
        total_revenue: Decimal,
        tax_rate: Decimal,
        tax_amount: Decimal,
        actor: &str,
    ) -> Result<TaxReportItem, AppError>;
    async fn get_tax_history(&self) -> Result<Vec<TaxReportItem>, AppError>;
    async fn delete_report(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgTaxRepository {
    pool: PgPool,
}

impl PgTaxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_tax_report(&self, r: &sqlx::postgres::PgRow) -> TaxReportItem {
        let status_str: String = r.get("status");
        let status = TaxStatus::from_str(&status_str).unwrap_or(TaxStatus::Unpaid);
        let tax_rate: Decimal = r.get("tax_rate");
        let tax_rate_percent = round_to_two_decimals(tax_rate * Decimal::from(100));

        TaxReportItem {
            id: r.get("id"),
            period_month: r.get("period_month"),
            period_year: r.get("period_year"),
            total_revenue: r.get("total_revenue"),
            tax_rate,
            tax_rate_percent,
            tax_amount: r.get("tax_amount"),
            status,
            payment_date: r.get("payment_date"),
            billing_code: r.get("billing_code"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            created_by: r.get("created_by"),
            modify_at: r.get("modify_at"),
            modify_by: r.get("modify_by"),
        }
    }
}

#[async_trait]
impl TaxRepository for PgTaxRepository {
    async fn get_settings(&self) -> Result<TaxSettingsDto, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, scheme_name, rate, description, modify_at, modify_by
            FROM tax_settings
            WHERE is_active = TRUE
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let r = row.ok_or_else(|| {
            AppError::Internal("Pengaturan tarif pajak aktif tidak ditemukan di sistem".into())
        })?;

        let rate: Decimal = r.get("rate");
        let rate_percent = round_to_two_decimals(rate * Decimal::from(100));

        Ok(TaxSettingsDto {
            id: r.get("id"),
            scheme_name: r.get("scheme_name"),
            rate,
            rate_percent,
            description: r.get("description"),
            modify_at: r.get("modify_at"),
            modify_by: r.get("modify_by"),
        })
    }

    async fn update_settings(
        &self,
        req: UpdateTaxSettingsRequest,
        actor: &str,
    ) -> Result<TaxSettingsDto, AppError> {
        let active_id_row = sqlx::query(
            "SELECT id FROM tax_settings WHERE is_active = TRUE ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        let id: String = match active_id_row {
            Some(row) => row.get("id"),
            None => "tax_cfg_default".to_string(),
        };

        let description = req.description.as_deref().unwrap_or("");

        let updated_row = sqlx::query(
            r#"
            INSERT INTO tax_settings (id, scheme_name, rate, description, is_active, created_by, modify_by)
            VALUES ($1, $2, $3, $4, TRUE, $5, $5)
            ON CONFLICT (id) DO UPDATE SET
                scheme_name = EXCLUDED.scheme_name,
                rate = EXCLUDED.rate,
                description = EXCLUDED.description,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = EXCLUDED.modify_by
            RETURNING id, scheme_name, rate, description, modify_at, modify_by
            "#,
        )
        .bind(&id)
        .bind(&req.scheme_name)
        .bind(req.rate)
        .bind(description)
        .bind(actor)
        .fetch_one(&self.pool)
        .await?;

        let rate: Decimal = updated_row.get("rate");
        let rate_percent = round_to_two_decimals(rate * Decimal::from(100));

        Ok(TaxSettingsDto {
            id: updated_row.get("id"),
            scheme_name: updated_row.get("scheme_name"),
            rate,
            rate_percent,
            description: updated_row.get("description"),
            modify_at: updated_row.get("modify_at"),
            modify_by: updated_row.get("modify_by"),
        })
    }

    async fn get_realized_revenue_for_period(
        &self,
        month: i32,
        year: i32,
    ) -> Result<(Decimal, i64), AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(total_revenue), 0.00) AS total_rev,
                COUNT(id) AS sales_count
            FROM sales
            WHERE status = 'Realized'
              AND CAST(EXTRACT(MONTH FROM sale_date) AS INTEGER) = $1
              AND CAST(EXTRACT(YEAR FROM sale_date) AS INTEGER) = $2
            "#,
        )
        .bind(month)
        .bind(year)
        .fetch_one(&self.pool)
        .await?;

        let total_rev: Decimal = row.get("total_rev");
        let sales_count: i64 = row.get("sales_count");

        Ok((round_to_two_decimals(total_rev), sales_count))
    }

    async fn get_report_by_period(
        &self,
        month: i32,
        year: i32,
    ) -> Result<Option<TaxReportItem>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, period_month, period_year, total_revenue, tax_rate, tax_amount,
                status, payment_date, billing_code, notes, created_at, created_by, modify_at, modify_by
            FROM tax_reports
            WHERE period_month = $1 AND period_year = $2
            "#,
        )
        .bind(month)
        .bind(year)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(|r| self.row_to_tax_report(r)))
    }

    async fn upsert_payment(
        &self,
        req: PayTaxRequest,
        total_revenue: Decimal,
        tax_rate: Decimal,
        tax_amount: Decimal,
        actor: &str,
    ) -> Result<TaxReportItem, AppError> {
        let report_id = format!("tax_{}", Uuid::new_v4());
        let payment_date: DateTime<Utc> = req.payment_date.unwrap_or_else(Utc::now);
        let notes = req.notes.as_deref().unwrap_or("");

        let row = sqlx::query(
            r#"
            INSERT INTO tax_reports (
                id, period_month, period_year, total_revenue, tax_rate, tax_amount,
                status, payment_date, billing_code, notes, created_by, modify_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                'Paid', $7, $8, $9, $10, $10
            )
            ON CONFLICT (period_month, period_year) DO UPDATE SET
                total_revenue = EXCLUDED.total_revenue,
                tax_rate = EXCLUDED.tax_rate,
                tax_amount = EXCLUDED.tax_amount,
                status = 'Paid',
                payment_date = EXCLUDED.payment_date,
                billing_code = EXCLUDED.billing_code,
                notes = EXCLUDED.notes,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = EXCLUDED.modify_by
            RETURNING
                id, period_month, period_year, total_revenue, tax_rate, tax_amount,
                status, payment_date, billing_code, notes, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&report_id)
        .bind(req.period_month)
        .bind(req.period_year)
        .bind(total_revenue)
        .bind(tax_rate)
        .bind(tax_amount)
        .bind(payment_date)
        .bind(&req.billing_code)
        .bind(notes)
        .bind(actor)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_tax_report(&row))
    }

    async fn get_tax_history(&self) -> Result<Vec<TaxReportItem>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, period_month, period_year, total_revenue, tax_rate, tax_amount,
                status, payment_date, billing_code, notes, created_at, created_by, modify_at, modify_by
            FROM tax_reports
            ORDER BY period_year DESC, period_month DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows.iter().map(|r| self.row_to_tax_report(r)).collect();
        Ok(items)
    }

    async fn delete_report(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM tax_reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Laporan pajak dengan ID '{id}' tidak ditemukan"
            )));
        }

        Ok(())
    }
}
