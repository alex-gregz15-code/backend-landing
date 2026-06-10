// BASE_APP/src/api/auth.rs

use axum::{extract::{State, Json}, http::StatusCode, response::Json as AxumJson};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

// ── App State (unchanged) ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

// ── JWT Claims (unchanged) ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,   // user id
    email: String,
    role: String,
    exp: usize,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    pub full_name: Option<String>,

    #[validate(custom(function = "validate_role"))]
    pub role: String,  // "researcher" or "commissioner"
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub message: String,
    pub user: UserPayload,
}

#[derive(Serialize)]
pub struct UserPayload {
    pub id: String,
    pub email: String,
    pub role: String,
    pub full_name: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ── Validator helper ──────────────────────────────────────────────────────────

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    if role == "researcher" || role == "commissioner" {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_role"))
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/auth/register
/// Validates input → hashes password → inserts into public.users → returns JWT
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, AxumJson<AuthResponse>), (StatusCode, AxumJson<ErrorResponse>)> {

    // 1. Validate input fields
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            AxumJson(ErrorResponse { error: e.to_string() }),
        ));
    }

    // 2. Check if email already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public.users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB error: {}", e) }),
    ))?;

    if existing > 0 {
        return Err((
            StatusCode::CONFLICT,
            AxumJson(ErrorResponse { error: "Email already registered".to_string() }),
        ));
    }

    // 3. Hash the password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ErrorResponse { error: format!("Hash error: {}", e) }),
        ))?
        .to_string();

    // 4. Generate a new UUID for the user
    let user_id = Uuid::new_v4().to_string();

    // 5. Insert into public.users (matches your Supabase schema)
    sqlx::query(
        r#"
        INSERT INTO public.users (id, email, full_name, role, password_hash)
        VALUES ($1::uuid, $2, $3, $4, $5)
        "#
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .bind(&password_hash)
    .execute(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("Insert error: {}", e) }),
    ))?;

    // 6. Issue JWT
    let token = create_jwt(&state.jwt_secret, &user_id, &payload.email, &payload.role)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ErrorResponse { error: format!("Token error: {}", e) }),
        ))?;

    Ok((
        StatusCode::CREATED,
        AxumJson(AuthResponse {
            access_token: token,
            message: format!("Welcome, {}!", payload.email),
            user: UserPayload {
                id: user_id,
                email: payload.email,
                role: payload.role,
                full_name: payload.full_name,
            },
        }),
    ))
}

/// POST /api/auth/login
/// Looks up user → verifies password → returns JWT
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<AxumJson<AuthResponse>, (StatusCode, AxumJson<ErrorResponse>)> {

    // 1. Validate input
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            AxumJson(ErrorResponse { error: e.to_string() }),
        ));
    }

    // 2. Find user by email
    let row = sqlx::query(
        "SELECT id::text AS id, email, role, full_name, password_hash FROM public.users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB error: {}", e) }),
    ))?;

    let user = row.ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        AxumJson(ErrorResponse { error: "Invalid email or password".to_string() }),
    ))?;

    let user_id: String = user.try_get("id").map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB row error: {}", e) }),
    ))?;
    let email: String = user.try_get("email").map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB row error: {}", e) }),
    ))?;
    let role: String = user.try_get("role").map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB row error: {}", e) }),
    ))?;
    let full_name: Option<String> = user.try_get("full_name").map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB row error: {}", e) }),
    ))?;
    let password_hash: String = user.try_get("password_hash").map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("DB row error: {}", e) }),
    ))?;

    // 3. Verify password
    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ErrorResponse { error: "Password verification error".to_string() }),
        ))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            AxumJson(ErrorResponse { error: "Invalid email or password".to_string() }),
        ))?;

    // 4. Issue JWT
    let token = create_jwt(
        &state.jwt_secret,
        &user_id,
        &email,
        &role,
    )
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        AxumJson(ErrorResponse { error: format!("Token error: {}", e) }),
    ))?;

    Ok(AxumJson(AuthResponse {
        access_token: token,
        message: format!("Welcome back, {}!", email),
        user: UserPayload {
            id: user_id,
            email,
            role,
            full_name,
        },
    }))
}

// ── JWT helper ────────────────────────────────────────────────────────────────

fn create_jwt(secret: &str, user_id: &str, email: &str, role: &str) -> Result<String> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(3600 * 24)) // 24 hours
        .expect("time overflow")
        .duration_since(UNIX_EPOCH)?
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        email: email.to_owned(),
        role: role.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(Into::into)
}
