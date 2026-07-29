use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::dto::auth::*;
use crate::dto::common::*;
use crate::dto::notifications::*;
use crate::dto::posts::*;
use crate::dto::users::*;
use crate::models::category::Category;
use crate::models::comment::Comment;
use crate::models::engagement::*;
use crate::models::notification::*;
use crate::models::post::*;
use crate::models::tag::Tag;
use crate::models::user::*;
use crate::routes::health::HealthResponse;
use crate::services::admin::{AdminDashboard, StatusCount};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "CaseFlow CMS API",
        version = "0.1.0",
        description = "Production-ready CMS backend with JWT auth, X/Twitter OAuth, engagement, and realtime notifications."
    ),
    paths(
        crate::routes::health::health,
        crate::routes::health::ready,
    ),
    components(
        schemas(
            HealthResponse,
            AuthResponse, TokenPair, RegisterRequest, LoginRequest, RefreshRequest, ChangePasswordRequest,
            MessageResponse, PublicUser, UserRole, Post, PostStatus, PostDetail,
            CreatePostRequest, UpdatePostRequest, Comment, CreateCommentRequest,
            Category, Tag, CreateCategoryRequest, CreateTagRequest,
            Like, Bookmark, Share, Follow, ShareRequest,
            Notification, NotificationType, NotificationList, NotificationEvent,
            UserListItem, UpdateProfileRequest, AdminUpdateUserRequest, FollowStats,
            AdminDashboard, StatusCount, IdResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health checks"),
        (name = "auth", description = "Authentication"),
        (name = "posts", description = "Content"),
        (name = "admin", description = "Administration"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
