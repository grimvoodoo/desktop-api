use axum::{
    async_trait,
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_login::{
    // route guards
    login_required,
    // session machinery
    tower_sessions::{MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
    AuthSession,
    // core traits
    AuthUser,
    AuthnBackend,
    UserId,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::process::Command;
use tracing::{error, info};
use uuid::Uuid;

// 1) Your “User” type just needs to implement AuthUser
#[derive(Clone, Debug)]
struct User {
    id: Uuid,
    // whatever you store as your auth-hash (must be stable per user)
    session_auth_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = Uuid;
    fn id(&self) -> Self::Id {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        &self.session_auth_hash
    }
}

// 2) Credentials: what you accept at login time
#[derive(Clone, Debug)]
struct Credentials {
    /// fixed “api” user
    user_id: Uuid,
    /// plain-text token
    token: String,
}

// 3) Your in-memory backend
#[derive(Clone, Default)]
struct Backend {
    users: HashMap<Uuid, User>,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(&self, creds: Credentials) -> Result<Option<Self::User>, Self::Error> {
        // Look up by ID, then compare creds.token → session_auth_hash
        if let Some(user) = self.users.get(&creds.user_id) {
            // constant-time compare
            if subtle::ConstantTimeEq::ct_eq(
                user.session_auth_hash.as_slice(),
                creds.token.as_bytes(),
            )
            .into()
            {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // --- bootstrap a one-time token + user on disk ---
    let token_file = "token.txt";
    let token = if let Ok(txt) = std::fs::read_to_string(token_file) {
        txt.trim().to_owned()
    } else {
        let t = Uuid::new_v4().to_string();
        std::fs::write(token_file, &t)?;
        t
    };
    let user_id = Uuid::new_v4();

    // Create our backend and insert the one API user
    let mut backend = Backend::default();
    backend.users.insert(
        user_id,
        User {
            id: user_id,
            session_auth_hash: token.clone().into_bytes(),
        },
    );
    info!("Generated token for API user {}: {}", user_id, token);

    // --- build the axum-login layer ---
    let session_store = MemoryStore::new();
    let session_layer = SessionManagerLayer::new(session_store);
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    // --- build our app ---
    let app = Router::new()
        // login page: you’d POST a form with `user_id` & `token`
        .route("/login", post(login_handler))
        // protected toggle endpoint
        .route(
            "/playpause",
            post(playpause_handler).layer(login_required!(Backend, login_url = "/login")),
        )
        .layer(auth_layer);

    let addr: SocketAddr = "0.0.0.0:5000".parse()?;
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

/// Handler for POST /login
async fn login_handler(
    mut session: AuthSession<Backend>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    match session.authenticate(creds.clone()).await {
        Ok(Some(user)) => {
            if session.login(&user).await.is_ok() {
                // redirects set the session cookie
                return Redirect::to("/playpause").into_response();
            }
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Protected handler for POST /playpause
/// login_required! ensures only a logged-in session can reach this.
async fn playpause_handler() -> Html<&'static str> {
    // your xdotool logic here…
    // e.g. tokio::process::Command::new("xdotool")…
    Html("<h1>Media toggled!</h1>")
}
