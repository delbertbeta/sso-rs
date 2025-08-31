use axum::{
    extract::{Form, State},
    response::{IntoResponse, Json},
};
use jsonwebtoken::{encode, Algorithm, Header, EncodingKey};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{application, authorization_code, token, user};

use crate::{
    error::{AppError, ServiceError},
    route::AppState,
    constants::PARSED_FRONTEND_URL,
};
use entity::application_secret;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    access_token: String,
    id_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    iat: usize,
}

pub async fn handler(
    State(state): State<AppState>,
    Form(form): Form<TokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if form.grant_type != "authorization_code" {
        return Err(AppError::ServiceError(ServiceError::InvalidGrant));
    }

    let txn = state.db.begin().await?;

    // Fetch and validate the authorization code from the database.
    let auth_code = authorization_code::Entity::find()
        .filter(authorization_code::Column::Code.eq(form.code.clone()))
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::ServiceError(ServiceError::InvalidGrant))?;

    if auth_code.expires_at < chrono::Utc::now().naive_utc() {
        return Err(AppError::ServiceError(ServiceError::InvalidGrant));
    }

    let app = auth_code.find_related(application::Entity)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::ServiceError(ServiceError::InvalidClient))?;

    if app.id != form.client_id {
        return Err(AppError::ServiceError(ServiceError::InvalidClient));
    }
    
    if auth_code.redirect_uri != form.redirect_uri {
        return Err(AppError::ServiceError(ServiceError::InvalidGrant));
    }

    // Delete the authorization code as it's for single-use.
    let _code_to_delete = auth_code.code.clone();
    authorization_code::Entity::delete_by_id(auth_code.code)
        .exec(&txn)
        .await?;

    // Verify the client secret.
    let app_secret = app.find_related(application_secret::Entity)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::ServiceError(ServiceError::InvalidClient))?;

    if form.client_secret != app_secret.secret {
        return Err(AppError::ServiceError(ServiceError::InvalidClient));
    }
    
    // Ensure the user exists.
    let user = user::Entity::find_by_id(auth_code.user_id.clone())
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::from(DbErr::RecordNotFound("User not found".to_string())))?;

    // Generate new access and refresh tokens.
    let access_token = Uuid::new_v4().to_string();
    let refresh_token = Uuid::new_v4().to_string();
    let expires_in = 3600; // 1 hour in seconds.
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(expires_in);

    // Create and save a new token to the database.
    let new_token = token::ActiveModel {
        access_token: Set(access_token.clone()),
        refresh_token: Set(refresh_token.clone()),
        user_id: Set(auth_code.user_id),
        application_id: Set(app.id.clone()),
        scopes: Set(auth_code.scopes),
        expires_at: Set(
            expires_at.naive_utc(),
        ),
        ..Default::default()
    };
    new_token.insert(&txn).await?;
    
    let claims = Claims {
        iss: PARSED_FRONTEND_URL.to_string(),
        sub: user.id.to_string(),
        aud: app.id,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    let id_token = encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(&state.oidc_keys.private_key.private_key_to_pem().unwrap()).unwrap(),
    )?;

    txn.commit().await?;
    
    // Construct the JSON response.
    let response = TokenResponse {
        access_token,
        id_token,
        token_type: "Bearer".to_string(),
        expires_in,
        refresh_token,
    };

    Ok(Json(response))
}
