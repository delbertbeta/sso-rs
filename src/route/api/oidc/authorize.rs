use std::collections::HashSet;

use axum::{
    extract::{Query, Extension},
    response::{IntoResponse, Redirect},
    http::StatusCode,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use entity::{application, authorization_code};

// A basic error type for this handler
#[derive(Debug)]
pub struct AuthError(StatusCode, String);

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

impl From<sea_orm::DbErr> for AuthError {
    fn from(err: sea_orm::DbErr) -> Self {
        eprintln!("Database error: {:?}", err);
        AuthError(StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    }
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
}

pub async fn handler(
    Extension(conn): Extension<sea_orm::DatabaseConnection>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate response_type is "code"
    if query.response_type != "code" {
        return Err(AuthError(StatusCode::BAD_REQUEST, "unsupported_response_type".to_string()));
    }

    // Fetch the application by client_id
    let app = application::Entity::find()
        .filter(application::Column::Id.eq(query.client_id.clone()))
        .one(&conn)
        .await?
        .ok_or_else(|| AuthError(StatusCode::BAD_REQUEST, "invalid_client".to_string()))?;

    // Validate the redirect_uri
    let redirect_uris: HashSet<String> = serde_json::from_str(&app.redirect_uris).unwrap();
    if !redirect_uris.contains(&query.redirect_uri) {
        return Err(AuthError(StatusCode::BAD_REQUEST, "invalid_redirect_uri".to_string()));
    }

    // Assume user is authenticated and get user ID (hardcoded for now)
    let user_id = 1; 

    // Generate a new authorization code
    let code = Uuid::new_v4().to_string();

    
// ... (previous code)

// Create and save the authorization_code model
    let new_code = authorization_code::ActiveModel {
        code: Set(code.clone()),
        user_id: Set(user_id),
        application_id: Set(app.id),
        scopes: Set(query.scope),
        redirect_uri: Set(query.redirect_uri.clone()),
        expires_at: Set(chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10)),
        ..Default::default()
    };

    new_code.insert(&conn).await?;

    // Construct the redirect URL
    let mut redirect_url = url::Url::parse(&query.redirect_uri)
        .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "invalid redirect_uri format".to_string()))?;
        
    redirect_url.query_pairs_mut().append_pair("code", &code);
    if let Some(state_val) = query.state {
        redirect_url.query_pairs_mut().append_pair("state", &state_val);
    }

    // Return a redirect response
    Ok(Redirect::to(redirect_url.as_str()))
}
