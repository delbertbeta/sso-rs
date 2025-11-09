use crate::{
    error::{AppError, ServiceError},
    response::OkResponse,
};
use std::collections::HashSet;

use axum::extract::{Extension, Query};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

use entity::{application, authorization_code};

#[derive(Serialize)]
pub struct SuccessResponse {
    redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: Option<String>,
    #[allow(dead_code)]
    nonce: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

pub async fn handler(
    Extension(conn): Extension<sea_orm::DatabaseConnection>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    // Validate response_type is "code"
    if query.response_type != "code" {
        return Err(ServiceError::UnsupportedResponseType.into());
    }

    // Fetch the application by client_id
    let app = application::Entity::find()
        .filter(application::Column::Id.eq(query.client_id.clone()))
        .one(&conn)
        .await?
        .ok_or(ServiceError::InvalidClient)?;

    // Validate the redirect_uri
    let redirect_uris: HashSet<String> = serde_json::from_str(&app.redirect_uris).unwrap();
    if !redirect_uris.contains(&query.redirect_uri) {
        return Err(ServiceError::InvalidRedirectUri.into());
    }

    // Assume user is authenticated and get user ID (hardcoded for now)
    let user_id = 1;

    // Generate a new authorization code
    let code = Uuid::new_v4().to_string();

    let scopes: Vec<&str> = query.scope.split(' ').collect();
    let scopes_json = serde_json::to_value(scopes).unwrap();

    // Create and save the authorization_code model
    let new_code = authorization_code::ActiveModel {
        code: Set(code.clone()),
        user_id: Set(user_id),
        application_id: Set(app.id),
        scopes: Set(scopes_json.to_string()),
        redirect_uri: Set(query.redirect_uri.clone()),
        expires_at: Set(chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10)),
        code_challenge: Set(query.code_challenge.clone()),
        code_challenge_method: Set(query.code_challenge_method.clone()),
        ..Default::default()
    };

    new_code.insert(&conn).await?;

    // Construct the redirect URL
    let mut redirect_url =
        url::Url::parse(&query.redirect_uri).map_err(|_| ServiceError::InvalidUriFormat)?;

    redirect_url.query_pairs_mut().append_pair("code", &code);
    if let Some(state_val) = query.state {
        redirect_url
            .query_pairs_mut()
            .append_pair("state", &state_val);
    }

    Ok(OkResponse::new(SuccessResponse {
        redirect_uri: redirect_url.to_string(),
    }))
}
