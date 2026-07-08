use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use routines_passkey::{PasskeyOps, passkey::types::webauthn::CredentialCreationOptions};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tai64::Tai64N;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper};

pub struct PasskeyHandler;

impl PasskeyHandler {
    pub async fn new_passkey(
        State(state): State<AppDb>,
        jar: CookieJar,
    ) -> Result<(StatusCode, String), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    passkey
                FROM users
                WHERE name = ?
                        "#,
        )
        .bind(&email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("expired session. New login required!")
        })?;

        let passkey_raw = row.try_get::<Option<String>, _>("passkey")?;

        if let Some(value) = passkey_raw {
            let passkey_data = serde_json::from_str::<PasskeyData>(&value)?;

            match passkey_data.state {
                PasskeyState::CreatePasskeyChallenge(value) => {
                    return Ok((StatusCode::OK, serde_json::to_string(&value)?));
                }
                _ => todo!(),
            }
        }

        let domain = if cfg!(debug_assertions) {
            "localhost"
        } else {
            "jamiidao.app"
        };

        let creds = PasskeyOps::new_passkey(
            &email,
            domain,
            "Jamii Routines - Solana Subscriptions with passkeys",
        );

        let ser_ui = serde_json::to_string(&creds)?;

        let data = PasskeyData {
            create_time: Tai64N::now(),
            verified: bool::default(),
            state: PasskeyState::CreatePasskeyChallenge(Box::new(creds)),
        };

        let ser_state = serde_json::to_string(&data)?;

        sqlx::query(
            r#"
                UPDATE users
                SET passkey = ?
                WHERE name = ?
                "#,
        )
        .bind(ser_state) // &[u8], Vec<u8>, or whatever BLOB you're storing
        .bind(email)
        .execute(&state.db)
        .await?;

        Ok((StatusCode::OK, ser_ui))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyData {
    pub create_time: Tai64N,
    pub verified: bool,
    pub state: PasskeyState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PasskeyState {
    CreatePasskeyChallenge(Box<CredentialCreationOptions>),
    VerifyCreatePasskeyChallenge,
}

impl PasskeyState {
    pub fn index(&self) -> u8 {
        match self {
            Self::CreatePasskeyChallenge(_) => 0,
            Self::VerifyCreatePasskeyChallenge => 1,
        }
    }
}

impl PartialEq for PasskeyState {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}

impl Eq for PasskeyState {}

impl Ord for PasskeyState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index().cmp(&other.index())
    }
}

impl PartialOrd for PasskeyState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
