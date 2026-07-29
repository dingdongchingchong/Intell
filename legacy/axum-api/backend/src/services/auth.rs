use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, TokenPair};
use crate::error::{AppError, AppResult};
use crate::models::user::{PublicUser, User, UserRole};
use crate::repositories::UserRepo;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub typ: String,
}

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> AppResult<String> {
        Ok(User::hash_password(password)?)
    }

    pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
        // Invalid / non-bcrypt hashes should fail closed as "wrong password", not 500.
        match bcrypt::verify(password, hash) {
            Ok(ok) => Ok(ok),
            Err(_) => Ok(false),
        }
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn issue_access_token(state: &AppState, user: &User) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(state.settings.jwt_access_ttl_secs);
        let claims = Claims {
            sub: user.id,
            role: user.role.as_str().to_string(),
            username: user.username.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: state.settings.jwt_issuer.clone(),
            typ: "access".into(),
        };
        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.settings.jwt_secret.as_bytes()),
        )?)
    }

    pub async fn issue_refresh_token(state: &AppState, user_id: Uuid) -> AppResult<String> {
        let token = format!("rt_{}", Uuid::new_v4());
        let hash = Self::hash_token(&token);
        let exp = Utc::now() + Duration::seconds(state.settings.jwt_refresh_ttl_secs);
        UserRepo::store_refresh_token(&state.db, user_id, &hash, exp).await?;
        Ok(token)
    }

    pub fn decode_access_token(state: &AppState, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&state.settings.jwt_issuer]);
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.settings.jwt_secret.as_bytes()),
            &validation,
        )?;
        if data.claims.typ != "access" {
            return Err(AppError::Unauthorized("invalid token type".into()));
        }
        Ok(data.claims)
    }

    pub async fn register(
        state: &AppState,
        email: &str,
        username: &str,
        password: &str,
        display_name: &str,
    ) -> AppResult<AuthResponse> {
        if UserRepo::find_by_email(&state.db, email).await?.is_some() {
            return Err(AppError::Conflict("email already registered".into()));
        }
        if UserRepo::find_by_username(&state.db, username).await?.is_some() {
            return Err(AppError::Conflict("username already taken".into()));
        }
        let hash = Self::hash_password(password)?;
        let user = UserRepo::create(
            &state.db,
            email,
            username,
            Some(&hash),
            display_name,
            UserRole::Author,
            None,
            false,
        )
        .await?;
        Self::tokens_for(state, user).await
    }

    pub async fn login(state: &AppState, login: &str, password: &str) -> AppResult<AuthResponse> {
        let user = UserRepo::find_by_login(&state.db, login)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
        if !user.is_active {
            return Err(AppError::Forbidden("account disabled".into()));
        }
        if !user.verify_password(password)? {
            return Err(AppError::Unauthorized("invalid credentials".into()));
        }
        UserRepo::touch_login(&state.db, user.id).await?;
        Self::tokens_for(state, user).await
    }

    pub async fn refresh(state: &AppState, refresh_token: &str) -> AppResult<TokenPair> {
        let hash = Self::hash_token(refresh_token);
        let (user_id, _) = UserRepo::find_valid_refresh(&state.db, &hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid refresh token".into()))?;
        UserRepo::revoke_refresh_token(&state.db, &hash).await?;
        let user = UserRepo::find_by_id(&state.db, user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;
        if !user.is_active {
            return Err(AppError::Forbidden("account disabled".into()));
        }
        let access = Self::issue_access_token(state, &user)?;
        let refresh = Self::issue_refresh_token(state, user.id).await?;
        Ok(TokenPair {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer".into(),
            expires_in: state.settings.jwt_access_ttl_secs,
        })
    }

    pub async fn logout(state: &AppState, refresh_token: &str) -> AppResult<()> {
        let hash = Self::hash_token(refresh_token);
        UserRepo::revoke_refresh_token(&state.db, &hash).await?;
        Ok(())
    }

    pub async fn change_password(
        state: &AppState,
        user_id: Uuid,
        current: &str,
        new_password: &str,
    ) -> AppResult<()> {
        let user = UserRepo::find_by_id(&state.db, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        if user.password_hash.is_none() {
            return Err(AppError::BadRequest("no password set".into()));
        }
        if !user.verify_password(current)? {
            return Err(AppError::Unauthorized("current password incorrect".into()));
        }
        let new_hash = Self::hash_password(new_password)?;
        UserRepo::set_password(&state.db, user_id, &new_hash).await?;
        Ok(())
    }

    async fn tokens_for(state: &AppState, user: User) -> AppResult<AuthResponse> {
        let access = Self::issue_access_token(state, &user)?;
        let refresh = Self::issue_refresh_token(state, user.id).await?;
        Ok(AuthResponse {
            user: PublicUser::from(user),
            tokens: TokenPair {
                access_token: access,
                refresh_token: refresh,
                token_type: "Bearer".into(),
                expires_in: state.settings.jwt_access_ttl_secs,
            },
        })
    }

    pub async fn seed_admin_if_needed(state: &AppState) -> AppResult<()> {
        if let Some(existing) =
            UserRepo::find_by_email(&state.db, &state.settings.seed_admin_email).await?
        {
            let needs_repair = match existing.password_hash.as_deref() {
                Some(hash) if Self::is_bcrypt_hash(hash) => {
                    !Self::verify_password(&state.settings.seed_admin_password, hash)?
                }
                _ => true,
            };
            if needs_repair {
                let hash = Self::hash_password(&state.settings.seed_admin_password)?;
                UserRepo::set_password(&state.db, existing.id, &hash).await?;
                tracing::warn!(
                    email = %state.settings.seed_admin_email,
                    "repaired admin password hash from SEED_ADMIN_PASSWORD"
                );
            }
            return Ok(());
        }
        let hash = Self::hash_password(&state.settings.seed_admin_password)?;
        UserRepo::create(
            &state.db,
            &state.settings.seed_admin_email,
            &state.settings.seed_admin_username,
            Some(&hash),
            &state.settings.seed_admin_name,
            UserRole::Admin,
            None,
            true,
        )
        .await?;
        tracing::info!(
            email = %state.settings.seed_admin_email,
            "seeded admin user"
        );
        Ok(())
    }

    fn is_bcrypt_hash(hash: &str) -> bool {
        hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$")
    }

    fn twitter_client(state: &AppState) -> AppResult<BasicClient> {
        let client_id = state
            .settings
            .twitter_client_id
            .clone()
            .ok_or_else(|| AppError::BadRequest("Twitter OAuth not configured".into()))?;
        let client_secret = state
            .settings
            .twitter_client_secret
            .clone()
            .ok_or_else(|| AppError::BadRequest("Twitter OAuth not configured".into()))?;
        let redirect = state
            .settings
            .twitter_redirect_uri
            .clone()
            .ok_or_else(|| AppError::BadRequest("Twitter OAuth not configured".into()))?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://twitter.com/i/oauth2/authorize".into())
                .map_err(|e| AppError::Internal(e.to_string()))?,
            Some(
                TokenUrl::new("https://api.twitter.com/2/oauth2/token".into())
                    .map_err(|e| AppError::Internal(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect).map_err(|e| AppError::Internal(e.to_string()))?,
        ))
    }

    pub async fn twitter_auth_url(state: &AppState) -> AppResult<(String, String)> {
        let client = Self::twitter_client(state)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("tweet.read".into()))
            .add_scope(Scope::new("users.read".into()))
            .add_scope(Scope::new("offline.access".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let expires = Utc::now() + Duration::minutes(10);
        UserRepo::save_oauth_state(
            &state.db,
            csrf.secret(),
            Some(pkce_verifier.secret()),
            state.settings.twitter_redirect_uri.as_deref(),
            expires,
        )
        .await?;
        Ok((auth_url.to_string(), csrf.secret().clone()))
    }

    pub async fn twitter_callback(
        state: &AppState,
        code: &str,
        oauth_state: &str,
    ) -> AppResult<AuthResponse> {
        let (verifier, _) = UserRepo::take_oauth_state(&state.db, oauth_state)
            .await?
            .ok_or_else(|| AppError::BadRequest("invalid or expired OAuth state".into()))?;
        let verifier = verifier
            .ok_or_else(|| AppError::BadRequest("missing PKCE verifier".into()))?;

        let client = Self::twitter_client(state)?;
        let token = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::Unauthorized(format!("oauth token exchange failed: {e}")))?;

        let access = token.access_token().secret();
        let http = reqwest::Client::new();
        let me: serde_json::Value = http
            .get("https://api.twitter.com/2/users/me")
            .bearer_auth(access)
            .query(&[("user.fields", "profile_image_url,name,username")])
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .error_for_status()
            .map_err(|e| AppError::Unauthorized(format!("twitter userinfo failed: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let data = me
            .get("data")
            .ok_or_else(|| AppError::Unauthorized("invalid twitter response".into()))?;
        let twitter_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Unauthorized("missing twitter id".into()))?;
        let username = data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("twitter_user");
        let display_name = data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(username);
        let avatar = data
            .get("profile_image_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let user = if let Some(existing) =
            UserRepo::find_by_twitter_id(&state.db, twitter_id).await?
        {
            existing
        } else {
            let email = format!("{twitter_id}@twitter.oauth.local");
            let uname = format!("tw_{username}");
            let mut final_username = uname.clone();
            let mut i = 0;
            while UserRepo::find_by_username(&state.db, &final_username)
                .await?
                .is_some()
            {
                i += 1;
                final_username = format!("{uname}{i}");
            }
            let mut user = UserRepo::create(
                &state.db,
                &email,
                &final_username,
                None,
                display_name,
                UserRole::Author,
                Some(twitter_id),
                true,
            )
            .await?;
            if avatar.is_some() {
                user = UserRepo::update_profile(
                    &state.db,
                    user.id,
                    None,
                    None,
                    avatar.as_deref(),
                )
                .await?;
            }
            user
        };

        UserRepo::touch_login(&state.db, user.id).await?;
        Self::tokens_for(state, user).await
    }
}
