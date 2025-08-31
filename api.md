# Existing APIs

This document lists the existing API endpoints in the sso-rs application.

## Hello World
- `GET /`: Returns a hello world message.

## User
- `GET /api/user`: Retrieves the current user's information.
- `PATCH /api/user`: Updates the current user's information.

## Authentication
- `POST /api/auth/register`: Registers a new user.
- `POST /api/auth/login`: Logs in a user.
- `POST /api/auth/logout`: Logs out a user.

## Crypto
- `GET /api/crypto/rsa`: Retrieves the RSA public key.

## Image
- `POST /api/image`: Uploads an image.
- `PATCH /api/image/:image_id`: Updates an image.

## Application
- `POST /api/application`: Creates a new application.
- `GET /api/application`: Retrieves a list of applications.
- `GET /api/application/:application_id`: Retrieves a single application.
- `GET /api/application/:application_id/secrets`: Retrieves a list of secrets for an application.
- `POST /api/application/:application_id/secrets`: Creates a new secret for an application.

## OIDC Core APIs

This section outlines the core OpenID Connect (OIDC) endpoints for handling authentication, token issuance, and metadata discovery.

### 1. Authorization Endpoint

-   **HTTP Method & Path**: `GET /api/oidc/authorize`
-   **Purpose**: To handle the authentication of the end-user and obtain their consent (authorization) for the client application to access their data. Upon successful authentication, it returns an authorization code to the client's redirect URI.
-   **Key Request Parameters (Query String)**:
    -   `response_type`: Must be `code`.
    -   `client_id`: The client application's unique identifier.
    -   `redirect_uri`: The callback URL where the response is sent.
    -   `scope`: A space-delimited list of scopes, which must include `openid`.
    -   `state`: An opaque value used by the client to maintain state between the request and callback to prevent CSRF attacks.
    -   `nonce`: A string value used to associate a client session with an ID token and to mitigate replay attacks.
-   **Successful Response Summary**: A `302 Found` redirect to the client's `redirect_uri` with the `code` and original `state` value in the query string.

### 2. Token Endpoint

-   **HTTP Method & Path**: `POST /api/oidc/token`
-   **Purpose**: To exchange an authorization code for an ID token, access token, and refresh token. This interaction is done server-to-server and requires client authentication.
-   **Key Request Parameters (Request Body - `application/x-www-form-urlencoded`)**:
    -   `grant_type`: Must be `authorization_code`.
    -   `code`: The authorization code received from the authorization endpoint.
    -   `redirect_uri`: The same redirect URI that was used in the authorization request.
    -   `client_id`: The client application's unique identifier.
    -   `client_secret`: The client application's secret for authentication.
-   **Successful Response Summary**: A `200 OK` response with a JSON body containing `access_token`, `id_token`, `token_type` (e.g., "Bearer"), `expires_in`, and optionally a `refresh_token`.

### 3. UserInfo Endpoint

-   **HTTP Method & Path**: `GET /api/oidc/userinfo`
-   **Purpose**: To retrieve claims about the authenticated end-user. This endpoint is protected and must be accessed using the access token obtained from the token endpoint.
-   **Key Request Parameters**:
    -   `Authorization` (HTTP Header): `Bearer <access_token>`
-   **Successful Response Summary**: A `200 OK` response with a JSON body containing user claims, such as `sub` (subject identifier), `name`, `email`, and other claims associated with the requested scopes.

### 4. JWKS Endpoint

-   **HTTP Method & Path**: `GET /.well-known/jwks.json`
-   **Purpose**: To expose the provider's public signing keys as a JSON Web Key Set (JWKS). Clients use this metadata to verify the signature of the ID Token.
-   **Key Request Parameters**: None.
-   **Successful Response Summary**: A `200 OK` response with a JSON body containing a `keys` array. Each object in the array represents a public key in JWK format, including properties like `kty` (key type), `kid` (key ID), and `use` (key use, e.g., "sig").

### 5. Discovery Endpoint

-   **HTTP Method & Path**: `GET /.well-known/openid-configuration`
-   **Purpose**: To provide a machine-readable JSON document describing the OIDC provider's configuration. This allows clients to dynamically discover endpoint URLs and capabilities.
-   **Key Request Parameters**: None.
-   **Successful Response Summary**: A `200 OK` response with a JSON body containing metadata about the provider, such as the `issuer` URL, and the paths to the `authorization_endpoint`, `token_endpoint`, `userinfo_endpoint`, and `jwks_uri`, among other configuration details.

