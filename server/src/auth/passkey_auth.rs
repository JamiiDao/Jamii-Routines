use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use base64ct::{Base64UrlUnpadded, Encoding};
use routines_passkey::{
    PasskeyOps, coset,
    p256::{self, ecdsa::signature},
    passkey::types::{
        Bytes,
        crypto::sha256,
        ctap2::AuthenticatorData,
        rand::random_vec,
        webauthn::{
            AttestationConveyancePreference, AuthenticatedPublicKeyCredential,
            AuthenticatorAttachment, AuthenticatorTransport, ClientDataType, CollectedClientData,
            CreatedPublicKeyCredential, CredentialCreationOptions, CredentialRequestOptions,
            PublicKeyCredentialDescriptor, PublicKeyCredentialRequestOptions,
            PublicKeyCredentialType, UserVerificationRequirement,
        },
    },
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tai64::Tai64N;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper};

pub struct PasskeyHandler;

impl PasskeyHandler {
    pub async fn new_passkey(
        State(state): State<AppDb>,
        jar: CookieJar,
    ) -> Result<(StatusCode, String), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let passkey_raw = Self::get_passkey_details(&state.db, &email).await?;

        if let Some(passkey_data) = passkey_raw
            && Tai64N::now() < (passkey_data.create_time + Duration::from_mins(10))
        {
            match passkey_data.state {
                PasskeyState::CreatePasskeyChallenge(value) => {
                    return Ok((StatusCode::OK, serde_json::to_string(&value)?));
                }
                _ => {
                    return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid passkey state, try enrolling your passkey again"));
                }
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

        Self::update_passkey(&state.db, &email, &data).await?;

        Ok((StatusCode::OK, ser_ui))
    }

    pub async fn register_passkey(
        State(state): State<AppDb>,
        jar: CookieJar,
        mut passkey_input: Json<CreatedPublicKeyCredential>,
    ) -> Result<(StatusCode, Json<RegisteredPasskeyUiData>), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let mut passkey_data = Self::get_passkey_details(&state.db, &email).await?.ok_or(
            HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey data, try enrolling your passkey again"),
        )?;

        if Tai64N::now() > (passkey_data.create_time + Duration::from_mins(10)) {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey data, try enrolling your passkey again"));
        }

        if passkey_data.state.index() > 0 {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid passkey state, you might have tried to register before unsuccessfully. Starting a fresh is required for security reasons. Sorry for the inconvenience!"));
        }

        if passkey_input.ty != PublicKeyCredentialType::PublicKey {
            tracing::error!(
                "PASSKEY INPUT ALGO: {:?}",
                passkey_input.response.public_key_algorithm
            );

            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey type, expected a public key"));
        }
        // Already decoded. Can be transformed into:
        // use p256::ecdsa::VerifyingKey;
        // use p256::pkcs8::DecodePublicKey;
        //
        // let verifying_key =
        // VerifyingKey::from_public_key_der(public_key.as_ref())?;
        let public_key = passkey_input.response.public_key.take().ok_or(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Public key not found yet required. Starting a fresh is required for security reasons. Sorry for the inconvenience!")
       )?;
        let algo = coset::iana::Algorithm::ES256 as i64;
        if passkey_input.response.public_key_algorithm != algo {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey type, expected a P256 Algorithm key"));
        }

        let collected_data_json_raw = str::from_utf8(&passkey_input.response.client_data_json).or(
                      Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid `client_data_json` field!. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))
                )?;
        let client_data = serde_json::from_str::<CollectedClientData>(collected_data_json_raw)?;
        if client_data.ty != ClientDataType::Create {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Expected the operation to create a new passkey. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))
                ;
        }
        let current_options = if let PasskeyState::CreatePasskeyChallenge(current_options) =
            passkey_data.state
        {
            current_options
        } else {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid passkey state. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))
                ;
        };

        let challenge_bytes = Base64UrlUnpadded::decode_vec( &client_data.challenge).or(
                   Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid base64 encoded challenge. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))

              )?;
        if challenge_bytes.as_slice() != current_options.public_key.challenge.as_slice() {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid base64 encoded challenge. Starting a fresh is required for security reasons. Sorry for the inconvenience!"));
        }

        let default_origin = if cfg!(debug_assertions) {
            "localhost"
        } else {
            "jamiidao.app"
        };

        if client_data.origin != default_origin {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey origin challenge!"));
        }

        let origin = client_data.origin;
        let transports = passkey_input.response.transports.take().unwrap_or_default();
        let attachment = passkey_input.authenticator_attachment.take();
        let credential_type = passkey_input.ty;
        let raw_id = passkey_input.raw_id.clone();

        let new_state = RegisteredPasskeyData {
            raw_id,
            public_key,
            algo,
            transports,
            attachment,
            credential_type,
            origin,
            current_challenge: Option::default(),
        };
        passkey_data.verified = true;
        let ui_data = RegisteredPasskeyUiData {
            verified: passkey_data.verified,
            raw_id: Base64UrlUnpadded::encode_string(&new_state.raw_id),
            public_key: Base64UrlUnpadded::encode_string(&new_state.public_key),
            algo: "P256".to_string(),
            transports: new_state.transports.clone(),
            attachment,
            credential_type,
            origin: new_state.origin.clone(),
        };

        tracing::error!("PASSKEY Outcome: {:?}", &new_state);
        passkey_data.state = PasskeyState::PasskeyVerified(new_state);

        Self::update_passkey(&state.db, &email, &passkey_data).await?;

        Ok((StatusCode::OK, Json(ui_data)))
    }

    pub async fn passkey_connect(
        State(state): State<AppDb>,
        jar: CookieJar,
    ) -> Result<(StatusCode, CookieJar, Json<CredentialRequestOptions>), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let mut passkey_data = Self::get_passkey_details(&state.db, &email).await?.ok_or(
            HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey data, try enrolling your passkey again"),
        )?;

        if !passkey_data.verified {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Passkey not verified, try enrolling your passkey again"));
        }

        let mut current_options = if let PasskeyState::PasskeyVerified(passkey_data_inner) =
            passkey_data.state
        {
            passkey_data_inner
        } else {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid passkey state. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))
                ;
        };

        let challenge: Bytes = random_vec(32).into();
        current_options.current_challenge.replace(challenge.clone());

        let rp_id = if cfg!(debug_assertions) {
            "localhost"
        } else {
            "jamiidao.app"
        };
        let rp_id = Some(rp_id.to_string());

        let user_verification = if !(cfg!(debug_assertions)) {
            UserVerificationRequirement::Discouraged
        } else {
            UserVerificationRequirement::Required
        };

        let options = CredentialRequestOptions {
            public_key: PublicKeyCredentialRequestOptions {
                challenge,
                timeout: Some(1000 * 60 * 5),
                rp_id,
                allow_credentials: Some(vec![PublicKeyCredentialDescriptor {
                    ty: PublicKeyCredentialType::PublicKey,
                    id: current_options.raw_id.clone(),
                    transports: Some(current_options.transports.clone()),
                }]),
                user_verification,
                hints: None,
                attestation: AttestationConveyancePreference::Indirect,
                attestation_formats: None,
                extensions: None,
            },
        };

        passkey_data.state = PasskeyState::PasskeyVerified(current_options);
        Self::update_passkey(&state.db, &email, &passkey_data).await?;

        Ok((StatusCode::OK, jar, Json(options)))
    }

    pub async fn passkey_verify(
        State(state): State<AppDb>,
        jar: CookieJar,
        input: Json<AuthenticatedPublicKeyCredential>,
    ) -> Result<(StatusCode, CookieJar), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let mut passkey_data = Self::get_passkey_details(&state.db, &email).await?.ok_or(
            HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey data, try enrolling your passkey again"),
        )?;

        if !passkey_data.verified {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Passkey not verified, try enrolling your passkey again"));
        }

        let mut current_options = if let PasskeyState::PasskeyVerified(passkey_data_inner) =
            passkey_data.state
        {
            passkey_data_inner
        } else {
            return Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid passkey state. Starting a fresh is required for security reasons. Sorry for the inconvenience!"));
        };
        let current_challenge = current_options.current_challenge.take().ok_or(
            HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("No challenge. Ask for a passkey connection first!"),
        )?;

        tracing::error!("TYPE: {:?}", input.ty);
        tracing::error!("Attachment: {:?}", input.authenticator_attachment);

        if input.raw_id != current_options.raw_id {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey credential selected by the identifier!"));
        }

        let collected_data_json_raw =
            str::from_utf8(&input.response.client_data_json).or(Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid `client_data_json` field! It must be UTF-8")))?;
        let client_data = serde_json::from_str::<CollectedClientData>(collected_data_json_raw)?;
        tracing::error!("COLLECTED CLIENT DATA: {client_data:?}");
        let decoded_challenge = Base64UrlUnpadded::decode_vec( &client_data.challenge).or(
                   Err(HttpErrorWrapper::new()
                        .status_code(StatusCode::BAD_REQUEST)
                        .message("Invalid base64 encoded challenge. Starting a fresh is required for security reasons. Sorry for the inconvenience!"))

              )?;
        if decoded_challenge.as_slice() != current_challenge.as_slice() {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey challenge!"));
        }
        if client_data.origin != current_options.origin {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid passkey origin challenge!"));
        }
        if client_data.ty != ClientDataType::Get {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message(
                    "Invalid passkey operation. Expected a connect request to an existing passkey!",
                ));
        }

        let authenticator_data = AuthenticatorData::from_slice(&input.response.authenticator_data)
        .or( Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message(
                    "Invalid passkey authenticator data operation. Expected a connect request to an existing passkey!",
                )))?;
        tracing::error!("authenticator_data parsed DATA: {authenticator_data:?}");

        let rp_id_bytes = if cfg!(debug_assertions) {
            "localhost".as_bytes()
        } else {
            "jamiidao.app".as_bytes()
        };
        let inner_rpid_hash = sha256(rp_id_bytes);
        if inner_rpid_hash != authenticator_data.rp_id_hash() {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message(
                    "Invalid domain name selected by passkey authenticator. Are you connected to the correct website?",
                ));
        }

        let client_hash = sha256(&input.response.client_data_json);
        let mut message_signed = Vec::new();
        message_signed.extend_from_slice(&input.response.authenticator_data);
        message_signed.extend_from_slice(&client_hash);

        use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
        use p256::pkcs8::DecodePublicKey;

        let verifying_key = VerifyingKey::from_public_key_der(current_options.public_key.as_ref())
        .or(
            Err(HttpErrorWrapper::new()
                .status_code(StatusCode::INTERNAL_SERVER_ERROR)
                .message(
                    "Invalid public key for passkey stored in the server. The only solution is to try recovering the correct passkey",
                ))
        )?;
        let signature = Signature::from_der(input.response.signature.as_ref()).or(Err(
            HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Invalid signature provided by the passkey authenticator."),
        ))?;

        verifying_key
            .verify(&message_signed, &signature)
            .or(Err(HttpErrorWrapper::new()
            .status_code(StatusCode::BAD_REQUEST)
            .message(
                "Invalid signature mismatch! Passkey authenticator gave the incorrect signature.",
            )))?;

        passkey_data.state = PasskeyState::PasskeyVerified(current_options);

        Self::update_passkey(&state.db, &email, &passkey_data).await?;

        Ok((StatusCode::OK, jar))
    }

    async fn get_passkey_details(
        db: &Pool<Sqlite>,
        email: &str,
    ) -> Result<Option<PasskeyData>, HttpErrorWrapper> {
        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    passkey
                FROM users
                WHERE name = ?
                        "#,
        )
        .bind(email)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| {
            HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("expired session. New login required!")
        })?;

        row.try_get::<Option<String>, _>("passkey")?
            .map(|exists| Ok::<_, HttpErrorWrapper>(serde_json::from_str::<PasskeyData>(&exists)?))
            .transpose()
    }

    async fn update_passkey(
        db: &Pool<Sqlite>,
        email: &str,
        passkey_data: &PasskeyData,
    ) -> Result<(), HttpErrorWrapper> {
        let ser_state = serde_json::to_string(passkey_data)?;

        sqlx::query(
            r#"
                UPDATE users
                SET passkey = ?
                WHERE name = ?
                "#,
        )
        .bind(ser_state) // &[u8], Vec<u8>, or whatever BLOB you're storing
        .bind(email)
        .execute(db)
        .await?;

        Ok(())
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
    PasskeyVerified(RegisteredPasskeyData),
}

impl PasskeyState {
    pub fn index(&self) -> u8 {
        match self {
            Self::CreatePasskeyChallenge(_) => 0,
            Self::PasskeyVerified(_) => 1,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredPasskeyData {
    pub raw_id: Bytes,
    pub public_key: Bytes,
    pub algo: i64,
    pub transports: Vec<AuthenticatorTransport>,
    pub attachment: Option<AuthenticatorAttachment>,
    pub credential_type: PublicKeyCredentialType,
    pub origin: String,
    pub current_challenge: Option<Bytes>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisteredPasskeyUiData {
    pub verified: bool,
    pub raw_id: String,
    pub public_key: String,
    pub algo: String,
    pub transports: Vec<AuthenticatorTransport>,
    pub attachment: Option<AuthenticatorAttachment>,
    pub credential_type: PublicKeyCredentialType,
    pub origin: String,
}
