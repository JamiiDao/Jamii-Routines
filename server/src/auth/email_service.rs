use std::sync::LazyLock;

use mail_list::{EmailEnvelopeDetails, Smtps, SmtpsBuilder};

use crate::SECRETS;

pub(crate) static MAILER: LazyLock<Smtps> = LazyLock::new(|| {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async move {
        let mut mailer = SmtpsBuilder::new();
        mailer
            .set_from("Support <support@jamiidao.app>")
            .set_hello_name("jamiidao.app")
            .set_reply_to("Support <support@jamiidao.app>");

        mailer.build(SECRETS.smtps()).unwrap()
    })
});

pub struct EmailService;

impl EmailService {
    pub async fn init_smtps() {
        MAILER.test_connection().await.unwrap();
    }

    pub fn send_auth(envelope: EmailEnvelopeDetails) {
        tokio::spawn(async move {
            if let Err(error) = MAILER.send(&envelope).await {
                tracing::error!("MAILER: {error:?}");
            }
        });
    }
}
