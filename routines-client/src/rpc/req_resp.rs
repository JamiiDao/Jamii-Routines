use jzon::JsonValue;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::HTTP_CLIENT;

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u8,
    pub result: T,
}

#[derive(Debug)]
pub struct RpcRequest {
    uri: String,
    method: String,
    params: Option<JsonValue>,
}

impl RpcRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_method(mut self, method: &str) -> Self {
        self.method = method.to_string();

        self
    }

    pub fn set_testnet(mut self) -> Self {
        self.uri = "https://api.testnet.solana.com".to_string();

        self
    }

    pub fn set_uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();

        self
    }

    pub fn set_position_arg_with_defaults(mut self, position_args: JsonValue) -> Self {
        let config_object = ConfigObject::default().build_json();

        self.params
            .replace(jzon::array![position_args, config_object]);

        self
    }

    pub fn set_position_arg_with_config_object(
        mut self,
        position_args: JsonValue,
        config_object: JsonValue,
    ) -> Self {
        self.params
            .replace(jzon::array![position_args, config_object]);

        self
    }

    pub fn set_position_args_with_defaults(mut self, mut position_args: jzon::Array) -> Self {
        let config_object = ConfigObject::default().build_json();
        position_args.push(config_object);

        self.params.replace(position_args.into());

        self
    }

    pub fn set_position_args_with_config_object(
        mut self,
        mut position_args: jzon::Array,
        config_object: JsonValue,
    ) -> Self {
        position_args.push(config_object);

        self.params.replace(position_args.into());

        self
    }

    pub fn set_config_object(mut self, config_object: JsonValue) -> Self {
        self.params.replace(jzon::array![config_object]);

        self
    }

    pub async fn send(self) -> Result<String, reqwest::Error> {
        let text = HTTP_CLIENT
            .post(&self.uri)
            .header("Content-Type", "application/json")
            .body(self.build())
            .send()
            .await?
            .text()
            .await?;

        Ok(text)
    }

    pub async fn send_and_decode<T: DeserializeOwned>(
        self,
    ) -> Result<RpcResponse<T>, reqwest::Error> {
        self.send_and_decode_inner::<T>().await
    }

    pub async fn send_and_decode_with_context<T: DeserializeOwned>(
        self,
    ) -> Result<RpcResponse<RpcResultWithContext<T>>, reqwest::Error> {
        self.send_and_decode_inner::<RpcResultWithContext<T>>()
            .await
    }

    async fn send_and_decode_inner<T: DeserializeOwned>(
        self,
    ) -> Result<RpcResponse<T>, reqwest::Error> {
        let decoded = HTTP_CLIENT
            .post(&self.uri)
            .header("Content-Type", "application/json")
            .body(self.build())
            .send()
            .await?
            .json::<RpcResponse<T>>()
            .await?;

        Ok(decoded)
    }

    fn build(self) -> String {
        if let Some(params) = self.params {
            jzon::object! {
               jsonrpc: "2.0",
               id: 1,
               method: self.method,
               params: params
            }
            .to_string()
        } else {
            jzon::object! {
               jsonrpc: "2.0",
               id: 1,
               method: self.method,
            }
            .to_string()
        }
    }
}

impl Default for RpcRequest {
    fn default() -> Self {
        Self {
            uri: "http://localhost:8899".to_string(),
            method: "getVersion".to_string(),
            params: Option::default(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigObject {
    pub commitment: String,
    pub encoding: String,
    pub skip_preflight: bool,
}

impl ConfigObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_finalized(mut self) -> Self {
        self.commitment = "finalized".to_string();

        self
    }

    pub fn set_processed(mut self) -> Self {
        self.commitment = "processed".to_string();

        self
    }

    pub fn set_encoding(mut self, encoding: &str) -> Self {
        self.encoding = encoding.to_string();

        self
    }

    pub fn build_json(&self) -> JsonValue {
        jzon::object! {
            "commitment":self.commitment.as_str(),
            "encoding":self.encoding.as_str(),
        }
    }
}

impl Default for ConfigObject {
    fn default() -> Self {
        Self {
            commitment: "confirmed".to_string(),
            encoding: "base64".to_string(),
            skip_preflight: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResultWithContext<T> {
    pub context: RpcResponseContext,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponseContext {
    pub slot: u64,
    pub api_version: Option<String>,
}
