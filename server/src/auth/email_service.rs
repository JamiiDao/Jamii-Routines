use std::sync::OnceLock;

use mail_list::{EmailEnvelopeDetails, Smtps, SmtpsBuilder};

pub(crate) static MAILER: OnceLock<Smtps> = OnceLock::new();

use crate::SECRETS;

pub struct EmailService;

impl EmailService {
    pub async fn init_smtps() {
        let mut mailer = SmtpsBuilder::new();
        mailer
            .set_from("Support <support@jamiidao.app>")
            .set_hello_name("jamiidao.app")
            .set_reply_to("Support <support@jamiidao.app>");

        let mailer = mailer.build(SECRETS.smtps()).unwrap();

        #[cfg(not(debug_assertions))]
        mailer.test_connection().await.unwrap();

        MAILER.set(mailer).unwrap();
    }

    pub fn send_auth(envelope: EmailEnvelopeDetails) {
        tokio::spawn(async move {
            if let Err(error) = MAILER.get().unwrap().send(&envelope).await {
                tracing::error!("MAILER: {error:?}");
            }
        });
    }
}
