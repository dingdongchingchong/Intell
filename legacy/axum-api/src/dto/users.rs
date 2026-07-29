use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::user::UserRole;
use crate::models::user::PublicUser;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 120))]
    pub display_name: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateUserRequest {
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
    pub display_name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListItem {
    pub user: PublicUser,
    pub email: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FollowStats {
    pub user_id: Uuid,
    pub followers: i64,
    pub following: i64,
}
