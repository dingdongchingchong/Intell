use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Paginated<T: Serialize> {
    pub items: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

impl PaginationQuery {
    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, 100) as i64
    }

    pub fn offset(&self) -> i64 {
        ((self.page.max(1) - 1) * self.per_page.clamp(1, 100)) as i64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IdResponse {
    pub id: Uuid,
}
