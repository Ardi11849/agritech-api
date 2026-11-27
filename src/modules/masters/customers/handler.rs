use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::customers::dto::{
    CreateCustomerRequest, CustomerListQuery, UpdateCustomerRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/customers",
    tag = "Master Data / Customers",
    security(("BearerAuth" = [])),
    params(CustomerListQuery),
    responses(
        (status = 200, description = "Successfully retrieved customers", body = ApiResponse<Vec<CustomerResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_customers(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<CustomerListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let customers = state.customer_service.list_customers(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved customers",
        customers,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/customers/{id}",
    tag = "Master Data / Customers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Customer UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved customer", body = ApiResponse<CustomerResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Customer not found", body = ApiResponse<()>)
    )
)]
pub async fn get_customer(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let customer = state.customer_service.get_customer(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved customer", customer);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/customers",
    tag = "Master Data / Customers",
    security(("BearerAuth" = [])),
    request_body = CreateCustomerRequest,
    responses(
        (status = 201, description = "Successfully created customer", body = ApiResponse<CustomerResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn create_customer(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateCustomerRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let customer = state
        .customer_service
        .create_customer(req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created customer",
        customer,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/customers/{id}",
    tag = "Master Data / Customers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Customer UUID")
    ),
    request_body = UpdateCustomerRequest,
    responses(
        (status = 200, description = "Successfully updated customer", body = ApiResponse<CustomerResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Customer not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn update_customer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(req): AppJson<UpdateCustomerRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let customer = state
        .customer_service
        .update_customer(&id, req, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated customer", customer);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/customers/{id}",
    tag = "Master Data / Customers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Customer UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted customer", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Customer not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_customer(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.customer_service.delete_customer(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted customer", ());
    Ok((status, body))
}
