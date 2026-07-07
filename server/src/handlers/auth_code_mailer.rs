use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use mail_list::EmailEnvelopeDetails;
use random_detached::RandomChars;
use serde::Deserialize;
use sqlx::{Pool, Row, Sqlite};
use tai64::Tai64N;

use crate::{AppDb, EmailService, HttpErrorWrapper, RouteHandler, RoutinesError};

impl RouteHandler {
    pub async fn process_resend_code(
        State(state): State<AppDb>,
        credentials: Json<ResendEmailAuthData>,
    ) -> Result<StatusCode, HttpErrorWrapper> {
        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    email_auth_code,
                    email_auth_time
                FROM users
                WHERE name = ?
            "#,
        )
        .bind(credentials.email.as_str())
        .fetch_one(&state.db)
        .await?;

        let email_auth_time: [u8; 12] = row
            .try_get::<Vec<u8>, _>("email_auth_time")?
            .try_into()
            .inspect_err(|_| {
                tracing::error!("Unable to convert sqlite blob for `email_auth_time` to [u8;12]");
            })
            .or(Err(HttpErrorWrapper::new()))?;

        let email_auth_time: Tai64N = email_auth_time
            .try_into()
            .inspect_err(|error| {
                tracing::error!(
                    "Unable to convert `email_auth_time` [u8;12] to Tai64N. Error: {error:?}."
                );
            })
            .or(Err(HttpErrorWrapper::new()))?;

        if Tai64N::now() < email_auth_time + Duration::from_secs(60) {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message("Wait at least 60 seconds"));
        }

        Self::send_auth(true, &credentials.email, &state.db).await?;
        Ok(StatusCode::OK)
    }

    pub(crate) async fn send_auth(
        resend: bool,
        email: &str,
        db: &Pool<Sqlite>,
    ) -> Result<(), HttpErrorWrapper> {
        let code = *RandomChars::<6>::new()
            .map_err(|error| {
                let error: RoutinesError = error.into();

                error
            })?
            .0;

        let code_for_storage = code.iter().collect::<String>();

        let code_formatted = format!(
            "{}-{}-{}-{}-{}-{}",
            code[0], code[1], code[2], code[3], code[4], code[5]
        );
        let code_timestamp = Tai64N::now().to_bytes();

        if resend {
            sqlx::query(
                r#"
                    UPDATE users
                    SET
                        email_auth_code = ?,
                        email_auth_time = ?
                    WHERE name = ?
                    "#,
            )
            .bind(code_for_storage.as_str())
            .bind(code_timestamp.as_slice())
            .bind(email)
            .execute(db)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO users (
                    name,
                    passkey,
                    create_time,
                    email_auth_code,
                    email_auth_time
                )
                VALUES (
                    ?,
                    NULL,
                    datetime('now'),
                    ?,
                    ?
                )
                "#,
            )
            .bind(email)
            .bind(code_for_storage.as_str())
            .bind(code_timestamp.as_slice())
            .execute(db)
            .await?;
        }

        let envelope = EmailEnvelopeDetails::new()
            .set_to(email)
            .set_body(&Self::email_body(code_formatted.as_str()))
            .set_subject("Jamii Routines Email verification code");

        EmailService::send_auth(envelope);

        Ok(())
    }

    fn email_body(code: &str) -> String {
        format!(
            r#"
        <!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Verify Your Email</title>
</head>
<body style="margin:0;padding:0;background-color:#f4f6f8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;color:#1f2937;">

<table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="padding:40px 16px;">
    <tr>
        <td align="center">

            <table role="presentation" width="600" cellspacing="0" cellpadding="0" style="max-width:600px;width:100%;border-radius:16px;overflow:hidden;border:1px solid #e5e7eb;box-shadow:0 8px 30px rgba(0,0,0,0.06);">

                <!-- Header -->
                <tr>
                    <td align="center" style="padding:20px 13px 24px;background:#3636c9;">
                        <h1 style="margin:0;font-size:20px;font-weight:700;color:#ffffff;">
                            Verification Code
                        </h1>
                    </td>
                </tr>

                <!-- Body -->
                <tr>
                    <td style="padding:20px 12px;">

                        <p style="margin:0 0 16px;font-size:16px;line-height:1.6;">
                            Hello,
                        </p>

                        <p style="margin:0 0 28px;font-size:16px;line-height:1.6;">
                            Use the verification code below to continue. This code is valid for
                            <strong>10 minutes</strong>.
                        </p>

                        <!-- Verification Code -->
                        <table role="presentation" width="100%" cellspacing="0" cellpadding="0">
                            <tr>
                                <td align="center">
                                    <div style="
                                        display:inline-block;
                                        background:#f9fafb;
                                        border:2px dashed #d1d5db;
                                        border-radius:14px;
                                        padding:10px 20px;
                                        font-size:18px;
                                        font-weight:700;
                                        letter-spacing:6px;
                                        color:#111827;
                                        font-family:'Courier New',monospace;
                                    ">
                                         {code}
                                    </div>
                                </td>
                            </tr>
                        </table>

                        <p style="margin:32px 0 0;font-size:15px;line-height:1.7;color:#6b7280;">
                            If you didn't request this code, you can safely ignore this email.
                        </p>

                    </td>
                </tr>

                <!-- Divider -->
                <tr>
                    <td style="border-top:1px solid #e5e7eb;"></td>
                </tr>

                <!-- Footer -->
                <tr>
                    <td align="center" style="padding:24px 32px;color:#9ca3af;font-size:13px;line-height:1.6;">
                        This is an automated message. Please do not reply.<br />
                        © 2026 Jamii Routines. All rights reserved. A product of JamiiDao
                    </td>
                </tr>

            </table>

        </td>
    </tr>
</table>

</body>
</html>

        "#,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ResendEmailAuthData {
    pub email: String,
}
