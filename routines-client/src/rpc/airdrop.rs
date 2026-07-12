pub struct RpcRequestAirdrop {
    pub lamports: Lamports,
    pub address: String,
}

impl RpcRequestAirdrop {
    pub fn to_json(&self) -> jzon::Array {
        let address: jzon::JsonValue = self.address.as_str().into();
        let lamports: jzon::JsonValue = (*self.lamports.as_ref()).into();

        let mut array = jzon::Array::new();
        array.push(address);
        array.push(lamports);

        array
    }
}

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Lamports(u64);

impl Lamports {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_sol(sol: u32) -> Self {
        Self(sol as u64 * LAMPORTS_PER_SOL)
    }
}

impl AsRef<u64> for Lamports {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Default for Lamports {
    fn default() -> Self {
        Self(LAMPORTS_PER_SOL)
    }
}
