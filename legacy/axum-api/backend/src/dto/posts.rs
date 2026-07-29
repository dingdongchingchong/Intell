use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::post::{Post, PostStatus};
use crate::models::tag::Tag;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub excerpt: Option<String>,
    pub category_id: Option<Uuid>,
    pub cover_image_url: Option<String>,
    pub status: Option<PostStatus>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub category_id: Option<Uuid>,
    pub cover_image_url: Option<String>,
    pub status: Option<PostStatus>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 40))]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostDetail {
    pub post: Post,
    pub tags: Vec<Tag>,
    pub liked_by_me: bool,
    pub bookmarked_by_me: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ShareRequest {
    pub platform: Option<String>,
}
