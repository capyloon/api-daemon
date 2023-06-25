use crate::generated::common::*;
use common::traits::{SimpleObjectTracker, TrackerId};
use ethers_core::types::transaction::{eip2718::TypedTransaction, request::TransactionRequest};
use ethers_core::types::{Bytes, NameOrAddress};
use ethers_providers::{Http, Middleware, Provider};
use log::error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

// TODO: make the chain list configurable.
lazy_static::lazy_static! {
    static ref CHAINS_URL: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Ethereum Mainnet", "https://eth.drpc.org/");
        m.insert("Gnosis Chain", "https://rpc.gnosischain.com");
        m
    };
}

pub fn get_chain_url(chain_name: &str) -> Option<&str> {
    CHAINS_URL.get(chain_name).copied()
}

pub struct EvmProvider {
    id: TrackerId,
    inner: Provider<Http>,
}

impl EvmProvider {
    pub fn new(url: &str, id: TrackerId) -> Option<Self> {
        if let Ok(inner) = Provider::<Http>::try_from(url) {
            Some(EvmProvider { inner, id })
        } else {
            None
        }
    }
}

impl SimpleObjectTracker for EvmProvider {
    fn id(&self) -> TrackerId {
        self.id
    }
}

// Runs async code in a tokio runtime.
macro_rules! tokioize {
    ($name:expr, $body:block) => {
        let _ = std::thread::Builder::new()
            .name($name.into())
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move { $body });
            });
    };
}

impl ProviderMethods for EvmProvider {
    fn call(&mut self, responder: ProviderCallResponder, params: CallParams, block: Option<i64>) {
        let inner = self.inner.clone();
        tokioize!("Provider.call", {
            if let (Ok(data), Ok(address)) = (
                Bytes::from_str(&params.data),
                NameOrAddress::from_str(&params.to),
            ) {
                let transaction = TransactionRequest::new().to(address).data(data);
                match inner
                    .call(
                        &TypedTransaction::Legacy(transaction),
                        block.map(|i| (i as u64).into()),
                    )
                    .await
                {
                    Ok(result) => responder.resolve(result.to_string()),
                    Err(err) => {
                        error!("call error: {}", err);
                        responder.reject(EvmError::RpcError)
                    }
                }
            } else {
                responder.reject(EvmError::InvalidParameters)
            }
        });
    }
}
