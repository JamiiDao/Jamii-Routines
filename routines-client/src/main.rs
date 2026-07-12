use std::sync::LazyLock;

use base64ct::{Base64, Encoding};
use reqwest::Client;
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::{Keypair, Signer};
use solana_transaction::{Message, Transaction};

mod rpc;
pub use rpc::*;

pub const PROGRAM_ADDRESS: &str = "DYhV4X9TFEBLNpz8aZBr4qeXtpFLbEXzhqtfpb2Je8wb";

pub(crate) static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::main]
async fn main() {
    let payer = Keypair::new();

    RpcRequest::new()
        .set_method("requestAirdrop")
        .set_position_args_with_config_object(
            RpcRequestAirdrop {
                lamports: Lamports::new(),
                address: payer.pubkey().to_string(),
            }
            .to_json(),
            ConfigObject::new().set_finalized().build_json(),
        )
        .send_and_decode::<String>()
        .await
        .unwrap();

    println!("Checking if enough SOL exists for program deployment");
    while let Ok(balance) = RpcRequest::new()
        .set_method("getBalance")
        .set_position_arg_with_defaults(payer.pubkey().to_string().into())
        .send_and_decode_with_context::<u64>()
        .await
    {
        println!("Looping Balance: {}", balance.result.value);

        if balance.result.value >= LAMPORTS_PER_SOL {
            println!("Balance: {}", balance.result.value);

            break;
        }
    }
    println!("Enough SOL exists for program deployment, continuing...");

    let program_id = Address::from_str_const(PROGRAM_ADDRESS);

    let p256_ix = p256_ix_test(&COMP_PUBKEY, &RAW_SIGNATURE, MESSAGE);

    let hello_ix = Instruction {
        program_id,
        accounts: vec![],
        data: vec![],
    };

    let message = Message::new(&[p256_ix, hello_ix], Some(&payer.pubkey()));

    let recent_blockhash = RpcRequest::default()
        .set_method("getLatestBlockhash")
        .set_config_object(ConfigObject::new().set_processed().build_json())
        .send_and_decode_with_context::<BlockHash>()
        .await
        .unwrap()
        .result
        .value;
    dbg!(recent_blockhash.to_string());

    let mut tx = Transaction::new_unsigned(message);
    tx.sign(&[&payer], recent_blockhash.blockhash.into());
    let tx_as_bytes = wincode::serialize(&tx).unwrap();
    let encoded_transaction = Base64::encode_string(&tx_as_bytes);
    let signature = RpcRequest::new()
        .set_method("sendTransaction")
        .set_position_arg_with_defaults(jzon::JsonValue::String(encoded_transaction))
        .send()
        .await
        .unwrap();

    dbg!(signature);
}

fn p256_ix_test(public_key: &[u8; 33], signature: &[u8; 64], message: &[u8]) -> Instruction {
    let num_signatures = 1u8;
    let padding = 0u8;

    let signature_offset = 2 + 14;
    let public_key_offset = signature_offset + 64;
    let message_offset = public_key_offset + 33;

    let mut data = Vec::new();

    // Header
    data.push(num_signatures);
    data.push(padding);

    // Secp256r1SignatureOffsets
    data.extend_from_slice(&(signature_offset as u16).to_le_bytes());
    data.extend_from_slice(&u16::MAX.to_le_bytes());

    data.extend_from_slice(&(public_key_offset as u16).to_le_bytes());
    data.extend_from_slice(&u16::MAX.to_le_bytes());

    data.extend_from_slice(&(message_offset as u16).to_le_bytes());
    data.extend_from_slice(&(message.len() as u16).to_le_bytes());
    data.extend_from_slice(&u16::MAX.to_le_bytes());

    // Signature: raw r || s (64 bytes)
    data.extend_from_slice(signature);

    // Compressed SEC1 public key (33 bytes)
    data.extend_from_slice(public_key);

    // WebAuthn signed bytes:
    // authenticatorData || SHA256(clientDataJSON)
    data.extend_from_slice(message);

    Instruction {
        program_id: solana_sdk_ids::secp256r1_program::ID,
        accounts: vec![],
        data,
    }
}

const COMP_PUBKEY: [u8; 33] = [0u8; 33];
const RAW_SIGNATURE: [u8; 64] = [0u8; 64];
const MESSAGE: &[u8] = &[];
