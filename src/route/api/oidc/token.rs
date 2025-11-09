use axum::{
    extract::{Form, Extension},
    response::{IntoResponse, Json},
};
use jsonwebtoken::{encode, Algorithm, Header, EncodingKey};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use entity::{application, authorization_code, token, user};

use crate::{
    error::{AppError, ServiceError},
    constants::PARSED_FRONTEND_URL,
};
use super::well_known::OidcKeys;
use entity::application_secret;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: Option<String>,
    code_verifier: Option<String>,
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
    Extension(conn): Extension<sea_orm::DatabaseConnection>,
    Extension(oidc_keys): Extension<OidcKeys>,
    Form(form): Form<TokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if form.grant_type != "authorization_code" {
        return Err(AppError::ServiceError(ServiceError::InvalidGrant));
    }

    let txn = conn.begin().await?;

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
        info!(
            "invalid client_id, expected: {}, actual: {}",
            app.id, form.client_id
        );
        return Err(AppError::ServiceError(ServiceError::InvalidClient));
    }
    
    if auth_code.redirect_uri != form.redirect_uri {
        return Err(AppError::ServiceError(ServiceError::InvalidGrant));
    }

    // PKCE and client secret validation.
    let pkce_verified = if let Some(code_challenge) = auth_code.code_challenge.as_ref() {
        let code_verifier = form
            .code_verifier
            .as_ref()
            .ok_or_else(|| AppError::ServiceError(ServiceError::InvalidGrant))?;

        let method = auth_code
            .code_challenge_method
            .as_deref()
            .unwrap_or("plain");

        let transformed_verifier = match method {
            "S256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(code_verifier.as_bytes());
                let result = hasher.finalize();
                base64_url::encode(&result)
            }
            "plain" => code_verifier.to_string(),
            _ => return Err(AppError::ServiceError(ServiceError::InvalidGrant)),
        };

        if &transformed_verifier != code_challenge {
            return Err(AppError::ServiceError(ServiceError::InvalidGrant));
        }
        true
    } else {
        false
    };

    // Delete the authorization code as it's for single-use.
    let _code_to_delete = auth_code.code.clone();
    authorization_code::Entity::delete_by_id(auth_code.code)
        .exec(&txn)
        .await?;

    // Verify the client secret or enforce PKCE for public clients.
    let app_secret = app.find_related(application_secret::Entity).one(&txn).await?;

    match app_secret {
        // Confidential client
        Some(secret) => {
            // For confidential clients, we must verify the secret if PKCE was not used.
            if !pkce_verified {
                if let Some(provided_secret) = form.client_secret.as_ref() {
                    if provided_secret != &secret.secret {
                        info!(
                            "invalid client_secret, expected: {}, actual: {}",
                            secret.secret, provided_secret
                        );
                        return Err(AppError::ServiceError(ServiceError::InvalidClient));
                    }
                } else {
                    info!("client_secret is required for confidential client when not using PKCE");
                    return Err(AppError::ServiceError(ServiceError::InvalidClient));
                }
            }
        }
        // Public client
        None => {
            if !pkce_verified {
                // Public clients must use PKCE
                return Err(AppError::ServiceError(ServiceError::InvalidGrant));
            }
            if form.client_secret.is_some() {
                info!("public client should not send client_secret");
                // Public clients must not send a secret
                return Err(AppError::ServiceError(ServiceError::InvalidClient));
            }
        }
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
        &EncodingKey::from_rsa_pem(&oidc_keys.private_key.private_key_to_pem().unwrap()).unwrap(),
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
