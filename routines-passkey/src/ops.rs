#[cfg(feature = "client")]
use benki_common::{BenkiError, BenkiResult};
#[cfg(feature = "client")]
use p256::ecdsa::{Signature as P256Signature, SigningKey as P256SigningKey};

#[cfg(feature = "client")]
use solana_instruction::Instruction;
#[cfg(any(feature = "program", feature = "client"))]
use solana_program_error::ProgramError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PasskeyProgramOps {
    pub signature: [u8; 64],
    pub public_key: [u8; 33],
    pub message: Vec<u8>,
}

impl PasskeyProgramOps {
    // Constants from agave code
    pub const COMPRESSED_PUBKEY_SERIALIZED_SIZE: usize = 33;
    pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 14;
    pub const SIGNATURE_OFFSETS_START: usize = 2;
    pub const DATA_START: usize =
        Self::SIGNATURE_OFFSETS_SERIALIZED_SIZE + Self::SIGNATURE_OFFSETS_START;

    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "client")]
    pub fn set_message(&mut self, message: impl AsRef<[u8]>) -> &mut Self {
        self.message = message.as_ref().to_vec();

        self
    }

    #[cfg(all(feature = "client", feature = "random_keypair"))]
    pub fn sign(&mut self) -> BenkiResult<&mut Self> {
        use p256::ecdsa::SigningKey as P256SigningKey;
        use rand_core::OsRng;

        let p256_signing_key = P256SigningKey::random(&mut OsRng);

        self.sign_with_key(&p256_signing_key)
    }

    #[cfg(feature = "client")]
    pub fn sign_with_key(&mut self, signing_key: &P256SigningKey) -> BenkiResult<&mut Self> {
        use p256::ecdsa::signature::Signer;

        let signature: P256Signature = signing_key.sign(&self.message);
        let signature_bytes: [u8; 64] = signature
            .to_bytes()
            .to_vec()
            .try_into()
            .or(Err(BenkiError::InvalidEcdsaSignature))?;

        let public_key: [u8; 33] = signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .try_into()
            .or(Err(BenkiError::InvalidEcdsaVerifyingKey))?;
        self.signature = signature_bytes;
        self.public_key = public_key;

        Ok(self)
    }

    #[cfg(feature = "client")]
    pub fn get_signature(&self) -> BenkiResult<P256Signature> {
        P256Signature::from_slice(&self.signature)
            .or(Err(BenkiError::UnableToParseP256SignatureFromBytes))
    }

    #[cfg(feature = "client")]
    pub fn build(&self) -> Vec<u8> {
        let mut p256_data_bytes = Vec::<u8>::with_capacity(
            Self::DATA_START
                .saturating_add(Self::SIGNATURE_SERIALIZED_SIZE)
                .saturating_add(Self::COMPRESSED_PUBKEY_SERIALIZED_SIZE)
                .saturating_add(self.message.len()),
        );

        let num_signatures: u8 = 1;
        let public_key_offset = Self::DATA_START;
        let signature_offset =
            public_key_offset.saturating_add(Self::COMPRESSED_PUBKEY_SERIALIZED_SIZE);
        let message_data_offset = signature_offset.saturating_add(Self::SIGNATURE_SERIALIZED_SIZE);

        // Encode the header
        p256_data_bytes.extend_from_slice(&[num_signatures, 0]);

        /*
                // From agave code
        struct Secp256r1SignatureOffsets {
            signature_offset: u16, // offset to compact secp256r1 signature of 64 bytes
            signature_instruction_index: u16, // instruction index to find signature
            public_key_offset: u16, // offset to compressed public key of 33 bytes
            public_key_instruction_index: u16, // instruction index to find public key
            message_data_offset: u16, // offset to start of message data
            message_data_size: u16, // size of message data
            message_instruction_index: u16, // index of instruction data to get message data
        } */
        p256_data_bytes.extend_from_slice(&(signature_offset as u16).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(u16::MAX).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(public_key_offset as u16).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(u16::MAX).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(message_data_offset as u16).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(self.message.len() as u16).to_le_bytes());
        p256_data_bytes.extend_from_slice(&(u16::MAX).to_le_bytes());

        // Encode the data
        p256_data_bytes.extend_from_slice(&self.public_key);
        p256_data_bytes.extend_from_slice(&self.signature);
        p256_data_bytes.extend_from_slice(&self.message);

        p256_data_bytes
    }

    #[cfg(feature = "client")]
    pub fn passkey_instruction(&self) -> Instruction {
        Instruction {
            program_id: solana_sdk_ids::secp256r1_program::ID,
            accounts: Vec::default(),
            data: self.build(),
        }
    }

    #[cfg(any(feature = "program", feature = "client"))]
    pub fn parse_instruction_data(secp256r1_data: &[u8]) -> Result<Self, ProgramError> {
        let header: [u8; 2] = secp256r1_data[0..2]
            .try_into()
            .or(Err(ProgramError::Custom(0)))?;
        // Check that only one signature exists,
        // since only verifying one signature is supported
        if u16::from_le_bytes(header) != 1 {
            return Err(ProgramError::Custom(1));
        }

        let public_key: [u8; 33] = secp256r1_data[16..=48]
            .try_into()
            .or(Err(ProgramError::Custom(2)))?;
        let signature: [u8; 64] = secp256r1_data[49..=112]
            .try_into()
            .or(Err(ProgramError::Custom(3)))?;
        let message: Vec<u8> = secp256r1_data[113..].to_vec();

        Ok(Self {
            signature,
            public_key,
            message,
        })
    }

    #[cfg(feature = "program")]
    pub fn read_p256_verify(
        accounts: &[pinocchio::AccountView],
    ) -> Result<Self, solana_program_error::ProgramError> {
        use pinocchio::sysvars::instructions::{Instructions, IntrospectedInstruction};
        use solana_program_error::ProgramError;

        let [sysvar_ixs_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Deserialize the instructions sysvar
        let instructions = Instructions::try_from(sysvar_ixs_program)?;
        let secp256r1_ix: IntrospectedInstruction = instructions.load_instruction_at(0)?;

        let secp256r1_ix_data = secp256r1_ix.get_instruction_data().to_vec();

        let outcome = Self::parse_instruction_data(&secp256r1_ix_data)?;

        Ok(outcome)
    }
}

impl Default for PasskeyProgramOps {
    fn default() -> Self {
        Self {
            signature: [0u8; 64],
            public_key: [0u8; 33],
            message: Vec::default(),
        }
    }
}
