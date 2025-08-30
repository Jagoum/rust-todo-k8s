use actix_web::{web, HttpRequest, HttpResponse, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{ApiResponse, Claims, CreateUserRequest, LoginRequest, User};
use crate::utils::jwt::{extract_user_id_from_token, JWT_SECRET};

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub user: AuthUserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(serde::Serialize)]
pub struct AuthUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
}

pub fn get_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str["Bearer ".len()..];
                if let Ok(user_id) = extract_user_id_from_token(token) {
                    return Some(user_id);
                }
            }
        }
    }
    None
}

pub fn extract_optional_user_id(req: &HttpRequest) -> Option<Uuid> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str["Bearer ".len()..];
                if let Ok(user_id) = extract_user_id_from_token(token) {
                    return Some(user_id);
                }
            }
        }
    }
    None
}

pub async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(errors) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            format!("Validation error: {:?}", errors),
        )));
    }

    let user_id = Uuid::new_v4();
    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to hash password".to_string(),
            )));
        }
    };

    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 OR username = $2",
        req.email,
        req.username
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing_user {
        Ok(Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                "User with this email or username already exists".to_string(),
            )));
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )));
        }
        Ok(None) => {}
    }

    // Insert new user
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username, email, password_hash, full_name, bio, is_verified, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, false, $7, $7)
        RETURNING id, username, email, password_hash, full_name, bio, avatar_url, is_verified, created_at, updated_at
        "#,
        user_id,
        req.username,
        req.email,
        password_hash,
        req.full_name,
        req.bio,
        Utc::now()
    )
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(user) => {
            let tokens = generate_tokens(&user);
            match tokens {
                Ok((access_token, refresh_token)) => {
                    let auth_response = AuthResponse {
                        user: AuthUserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            full_name: user.full_name,
                            bio: user.bio,
                            avatar_url: user.avatar_url,
                            is_verified: user.is_verified.unwrap_or(false),
                        },
                        access_token,
                        refresh_token,
                    };
                    Ok(HttpResponse::Created().json(ApiResponse::success(auth_response)))
                }
                Err(_) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to generate tokens".to_string(),
                ))),
            }
        }
        Err(e) => {
            log::error!("Failed to create user: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to create user".to_string(),
            )))
        }
    }
}

pub async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, full_name, bio, avatar_url, is_verified, created_at, updated_at FROM users WHERE email = $1",
        req.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            match verify(&req.password, &user.password_hash) {
                Ok(is_valid) if is_valid => {
                    let tokens = generate_tokens(&user);
                    match tokens {
                        Ok((access_token, refresh_token)) => {
                            let auth_response = AuthResponse {
                                user: AuthUserResponse {
                                    id: user.id,
                                    username: user.username,
                                    email: user.email,
                                    full_name: user.full_name,
                                    bio: user.bio,
                                    avatar_url: user.avatar_url,
                                    is_verified: user.is_verified.unwrap_or(false),
                                },
                                access_token,
                                refresh_token,
                            };
                            Ok(HttpResponse::Ok().json(ApiResponse::success(auth_response)))
                        }
                        Err(_) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Failed to generate tokens".to_string(),
                        ))),
                    }
                }
                _ => Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                    "Invalid credentials".to_string(),
                ))),
            }
        }
        Ok(None) => Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "Invalid credentials".to_string(),
        ))),
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn refresh_token(
    _pool: web::Data<PgPool>,
    _req: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // For now, return a simple response
    // In a production app, you'd validate the refresh token and generate new tokens
    Ok(HttpResponse::Ok().json(ApiResponse::<()>::error(
        "Refresh token functionality not implemented yet".to_string(),
    )))
}

fn generate_tokens(user: &User) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let access_expiration = Utc::now() + Duration::hours(1);
    let refresh_expiration = Utc::now() + Duration::days(30);

    let access_claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: access_expiration.timestamp() as usize,
    };

    let refresh_claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: refresh_expiration.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )?;

    Ok((access_token, refresh_token))
}