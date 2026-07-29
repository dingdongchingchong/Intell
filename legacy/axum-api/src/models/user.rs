use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Editor,
    Author,
    Viewer,
}

impl UserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Author => "author",
            Self::Viewer => "viewer",
        }
    }

    pub fn can_manage_users(self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_moderate(self) -> bool {
        matches!(self, Self::Admin | Self::Editor)
    }

    pub fn can_publish(self) -> bool {
        matches!(self, Self::Admin | Self::Editor | Self::Author)
    }

    pub fn can_edit(self) -> bool {
        !matches!(self, Self::Viewer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub is_verified: bool,
    pub twitter_id: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            bio: u.bio,
            avatar_url: u.avatar_url,
            role: u.role,
            created_at: u.created_at,
        }
    }
}

impl User {
    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        match &self.password_hash {
            Some(hash) => bcrypt::verify(password, hash),
            None => Ok(false),
        }
    }
}
