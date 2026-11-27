use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum InstallationStatus {
    Active,
    Maintenance,
    Inactive,
}

impl fmt::Display for InstallationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Maintenance => write!(f, "Maintenance"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

impl FromStr for InstallationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(Self::Active),
            "Maintenance" => Ok(Self::Maintenance),
            "Inactive" => Ok(Self::Inactive),
            other => Err(format!("Unknown installation status: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationEntity {
    pub id: String,
    pub location_id: String,
    pub name: String,
    pub capacity_holes: i32,
    pub status: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRow {
    pub id: String,
    pub location_id: String,
    pub location_name: String,
    pub name: String,
    pub capacity_holes: i32,
    pub status: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateInstallationRequest {
    #[validate(length(min = 1, message = "Location ID is required"))]
    #[schema(example = "loc_550e8400-e29b-41d4-a716-446655440000")]
    pub location_id: String,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Modul A1")]
    pub name: String,

    #[schema(example = 400)]
    pub capacity_holes: i32,

    pub status: Option<InstallationStatus>,

    #[validate(length(max = 500, message = "Notes cannot exceed 500 characters"))]
    #[schema(example = "Instalasi rak NFT baris utara")]
    pub notes: Option<String>,
}

pub type UpdateInstallationRequest = CreateInstallationRequest;

#[derive(Debug, Deserialize, IntoParams)]
pub struct InstallationListQuery {
    pub location_id: Option<String>,
    pub status: Option<InstallationStatus>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InstallationResponse {
    pub id: String,
    pub location_id: String,
    pub location_name: String,
    pub name: String,
    pub capacity_holes: i32,
    pub status: InstallationStatus,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}
