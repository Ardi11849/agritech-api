use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use super::dto::*;
use super::repository::SopRepository;
use crate::common::error::AppError;

pub fn next_sop_version(current_version: i32) -> i32 {
    current_version + 1
}

pub fn normalize_sop_steps(steps: &[SopStepInput]) -> Result<Vec<NormalizedSopStep>, AppError> {
    if steps.is_empty() {
        return Err(AppError::BadRequest(
            "SOP wajib memiliki minimal 1 langkah prosedur".to_string(),
        ));
    }

    if steps.len() > 100 {
        return Err(AppError::BadRequest(
            "Jumlah langkah SOP melebihi batas maksimum (100 langkah)".to_string(),
        ));
    }

    let mut normalized = Vec::with_capacity(steps.len());

    for (idx, step) in steps.iter().enumerate() {
        let instruction = step.instruction.trim().to_string();
        if instruction.is_empty() {
            return Err(AppError::BadRequest(format!(
                "Instruksi pada langkah #{} tidak boleh kosong",
                idx + 1
            )));
        }

        if instruction.chars().count() > 2000 {
            return Err(AppError::BadRequest(format!(
                "Instruksi pada langkah #{} melebihi 2000 karakter",
                idx + 1
            )));
        }

        let notes = step
            .notes
            .as_deref()
            .map(|n| n.trim().to_string())
            .unwrap_or_default();

        if notes.chars().count() > 1000 {
            return Err(AppError::BadRequest(format!(
                "Catatan pada langkah #{} melebihi 1000 karakter",
                idx + 1
            )));
        }

        normalized.push(NormalizedSopStep {
            step_number: (idx + 1) as i32,
            instruction,
            notes,
        });
    }

    Ok(normalized)
}

#[async_trait]
pub trait SopService: Send + Sync {
    async fn get_sops(&self, query: &SopQuery) -> Result<SopListResponse, AppError>;
    async fn get_sop_by_id(&self, id: &str) -> Result<SopResponse, AppError>;
    async fn create_sop(&self, req: CreateSopRequest, actor: &str)
        -> Result<SopResponse, AppError>;
    async fn update_sop(
        &self,
        id: &str,
        req: UpdateSopRequest,
        actor: &str,
    ) -> Result<SopResponse, AppError>;
    async fn delete_sop(&self, id: &str) -> Result<(), AppError>;
}

pub struct SopServiceImpl {
    repo: Arc<dyn SopRepository>,
}

impl SopServiceImpl {
    pub fn new(repo: Arc<dyn SopRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SopService for SopServiceImpl {
    async fn get_sops(&self, query: &SopQuery) -> Result<SopListResponse, AppError> {
        let (kpi, items) = self.repo.find_all(query).await?;
        Ok(SopListResponse { kpi, items })
    }

    async fn get_sop_by_id(&self, id: &str) -> Result<SopResponse, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("SOP with ID '{id}' not found")))
    }

    async fn create_sop(
        &self,
        req: CreateSopRequest,
        actor: &str,
    ) -> Result<SopResponse, AppError> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::BadRequest(
                "Judul SOP tidak boleh kosong".to_string(),
            ));
        }

        if title.chars().count() > 150 {
            return Err(AppError::BadRequest(
                "Judul SOP maksimal 150 karakter".to_string(),
            ));
        }

        let description = req
            .description
            .as_deref()
            .map(|d| d.trim().to_string())
            .unwrap_or_default();

        if description.chars().count() > 2000 {
            return Err(AppError::BadRequest(
                "Deskripsi SOP maksimal 2000 karakter".to_string(),
            ));
        }

        let normalized_steps = normalize_sop_steps(&req.steps)?;

        let now = Utc::now();
        let sop_id = format!("sop_{}", Uuid::new_v4());
        let status = req.status.unwrap_or(SopStatus::Draft);

        let sop_row = SopRow {
            id: sop_id,
            title,
            category: req.category.to_string(),
            description,
            version: 1, // AC-1: Versi awal selalu 1
            status: status.to_string(),
            created_at: now,
            created_by: actor.to_string(),
            modify_at: now,
            modify_by: actor.to_string(),
        };

        self.repo.create(sop_row, normalized_steps).await
    }

    async fn update_sop(
        &self,
        id: &str,
        req: UpdateSopRequest,
        actor: &str,
    ) -> Result<SopResponse, AppError> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::BadRequest(
                "Judul SOP tidak boleh kosong".to_string(),
            ));
        }

        if title.chars().count() > 150 {
            return Err(AppError::BadRequest(
                "Judul SOP maksimal 150 karakter".to_string(),
            ));
        }

        let description = req
            .description
            .as_deref()
            .map(|d| d.trim().to_string())
            .unwrap_or_default();

        if description.chars().count() > 2000 {
            return Err(AppError::BadRequest(
                "Deskripsi SOP maksimal 2000 karakter".to_string(),
            ));
        }

        let normalized_steps = normalize_sop_steps(&req.steps)?;
        let status = req.status.unwrap_or(SopStatus::Draft);

        self.repo
            .update(
                id,
                &title,
                &req.category.to_string(),
                &description,
                &status.to_string(),
                normalized_steps,
                actor,
            )
            .await
    }

    async fn delete_sop(&self, id: &str) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("SOP with ID '{id}' not found")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simulasi 1: Auto-Versioning & Normalisasi Urutan Langkah SOP (PRD §3.1 & §3.3)
    #[test]
    fn test_sop_simulasi_1_normalization_and_versioning() {
        // 1. Valid Input dengan 3 langkah
        let input_steps = vec![
            SopStepInput {
                instruction: "  Siapkan air baku EC < 0.2  ".to_string(),
                notes: Some("Pastikan tangki bersih".to_string()),
            },
            SopStepInput {
                instruction: "Tuang Pekatan A lalu aduk rata".to_string(),
                notes: None,
            },
            SopStepInput {
                instruction: "Tuang Pekatan B dan ukur PPM".to_string(),
                notes: Some("Target 800-900 PPM".to_string()),
            },
        ];

        let normalized = normalize_sop_steps(&input_steps).unwrap();
        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0].step_number, 1);
        assert_eq!(normalized[0].instruction, "Siapkan air baku EC < 0.2");
        assert_eq!(normalized[0].notes, "Pastikan tangki bersih");

        assert_eq!(normalized[1].step_number, 2);
        assert_eq!(normalized[1].instruction, "Tuang Pekatan A lalu aduk rata");
        assert_eq!(normalized[1].notes, "");

        assert_eq!(normalized[2].step_number, 3);
        assert_eq!(normalized[2].instruction, "Tuang Pekatan B dan ukur PPM");

        // 2. Test Inkrementasi Versi
        let version_v1 = 1;
        let version_v2 = next_sop_version(version_v1);
        assert_eq!(version_v2, 2);
        let version_v3 = next_sop_version(version_v2);
        assert_eq!(version_v3, 3);

        // 3. Invalid Inputs
        // Empty steps
        assert!(normalize_sop_steps(&[]).is_err());

        // Blank instruction
        let invalid_steps = vec![
            SopStepInput {
                instruction: "Valid step".to_string(),
                notes: None,
            },
            SopStepInput {
                instruction: "   ".to_string(),
                notes: None,
            },
        ];
        assert!(normalize_sop_steps(&invalid_steps).is_err());
    }
}
