use passkey::types::{
    rand::random_vec,
    webauthn::{
        AttestationConveyancePreference, AuthenticatorAttachment, AuthenticatorSelectionCriteria,
        CredentialCreationOptions, PublicKeyCredentialCreationOptions,
        PublicKeyCredentialParameters, PublicKeyCredentialRpEntity, PublicKeyCredentialType,
        PublicKeyCredentialUserEntity, ResidentKeyRequirement, UserVerificationRequirement,
    },
};

pub struct PasskeyOps;

impl PasskeyOps {
    pub fn new_passkey(
        username: &str,
        domain: &str,
        description: &str,
    ) -> CredentialCreationOptions {
        let user = PublicKeyCredentialUserEntity {
            id: random_vec(32).into(),
            name: username.to_string(),
            display_name: username.to_string(),
        };

        Self::create_registration_challenge(domain, description, user)
    }

    pub fn create_registration_challenge(
        rp_id: &str,
        rp_name: &str,
        user: PublicKeyCredentialUserEntity,
    ) -> CredentialCreationOptions {
        let challenge: passkey::types::Bytes = random_vec(32).into();

        CredentialCreationOptions {
            public_key: PublicKeyCredentialCreationOptions {
                rp: PublicKeyCredentialRpEntity {
                    id: Some(rp_id.to_string()),
                    name: rp_name.to_string(),
                },
                user,
                challenge,
                pub_key_cred_params: vec![
                    PublicKeyCredentialParameters {
                        ty: PublicKeyCredentialType::PublicKey,
                        alg: coset::iana::Algorithm::ES256,
                    },
                    PublicKeyCredentialParameters {
                        ty: PublicKeyCredentialType::PublicKey,
                        alg: coset::iana::Algorithm::RS256,
                    },
                ],
                timeout: Some(1000 * 60 * 2),
                exclude_credentials: None,
                authenticator_selection: Some(AuthenticatorSelectionCriteria {
                    authenticator_attachment: Some(AuthenticatorAttachment::CrossPlatform),
                    resident_key: Some(ResidentKeyRequirement::Discouraged),
                    require_resident_key: false,
                    user_verification: UserVerificationRequirement::Required,
                }),
                hints: None,
                attestation: AttestationConveyancePreference::Indirect,
                attestation_formats: None,
                extensions: None,
            },
        }
    }
}
