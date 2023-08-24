pub mod bitcoind;
pub mod error;
pub mod receive;
pub mod send;
pub mod transaction;
pub use bitcoin::blockdata::script::Script as BitcoinScript;
pub use bitcoin::Address as BitcoinAdrress;
use bitcoin::{Amount, ScriptBuf};
pub use payjoin::Error as PdkError;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs::OpenOptions, str::FromStr};
uniffi::include_scaffolding!("pdk");
pub struct CachedOutputs {
	pub outputs: HashSet<OutPoint>,
	pub file: std::fs::File,
}
impl CachedOutputs {
	pub fn new(path: String) -> Result<Self, bitcoincore_rpc::Error> {
		let mut file = OpenOptions::new().write(true).read(true).create(true).open(path)?;
		let outputs = bitcoincore_rpc::jsonrpc::serde_json::from_reader(&mut file)
			.unwrap_or_else(|_| HashSet::new());
		Ok(Self { outputs, file })
	}
}

/// A reference to a transaction output.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct OutPoint {
	/// The referenced transaction's txid.
	pub txid: String,
	/// The index of the referenced output in its transaction's vout.
	pub vout: u32,
}

impl From<OutPoint> for bitcoin::OutPoint {
	fn from(outpoint: OutPoint) -> Self {
		bitcoin::OutPoint {
			txid: bitcoin::Txid::from_str(&outpoint.txid).expect("Invalid txid"),
			vout: outpoint.vout,
		}
	}
}
impl From<bitcoin::OutPoint> for OutPoint {
	fn from(outpoint: bitcoin::OutPoint) -> Self {
		OutPoint { txid: outpoint.txid.to_string(), vout: outpoint.vout }
	}
}

#[derive(Debug, Clone)]
pub struct Script {
	inner: Vec<u8>,
}

impl Script {
	fn new(raw_output_script: Vec<u8>) -> Self {
		let script = BitcoinScript::from_bytes(raw_output_script.as_slice());
		Script { inner: script.to_bytes() }
	}

	fn to_bytes(&self) -> Vec<u8> {
		self.inner.to_owned()
	}
}
#[derive(Debug, Clone)]
pub struct TxOut {
	/// The value of the output, in satoshis.
	value: u64,
	/// The address of the output.
	script_pubkey: Script,
}
impl From<TxOut> for bitcoin::TxOut {
	fn from(tx_out: TxOut) -> Self {
		bitcoin::TxOut {
			value: tx_out.value,
			script_pubkey: ScriptBuf::from_bytes(tx_out.script_pubkey.inner),
		}
	}
}

impl From<bitcoin::TxOut> for TxOut {
	fn from(tx_out: bitcoin::TxOut) -> Self {
		TxOut {
			value: tx_out.value,
			script_pubkey: Script { inner: tx_out.script_pubkey.to_bytes() },
		}
	}
}
pub struct Uri {
	pub internal: String,
}

impl Uri {
	pub fn new(
		amount: u64, endpoint: String, address: String,
	) -> Result<Self, bitcoincore_rpc::Error> {
		let addr = bitcoin::address::Address::from_str(address.as_str()).expect("Invalid address");
		let uri_str =
			format!("{:?}?amount={}&pj={}", addr, Amount::from_sat(amount).to_btc(), endpoint);
		let _ = payjoin::Uri::from_str(uri_str.as_ref())
			.map_err(|e| panic!("Constructed a bad URI string from args: {}", e));
		Ok(Self { internal: uri_str })
	}

	pub fn try_from(bip21_str: String) -> Result<Self, bitcoincore_rpc::Error> {
		match payjoin::Uri::from_str(bip21_str.as_ref()) {
			Ok(e) => Ok(Self { internal: bip21_str }),
			Err(e) => panic!("Constructed a bad URI string from args: {}", e),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum AddressType {
	Legacy,
	P2shSegwit,
	Bech32,
	Bech32m,
}
impl From<AddressType> for bitcoincore_rpc::json::AddressType {
	fn from(value: AddressType) -> Self {
		return match value {
			AddressType::Legacy => bitcoincore_rpc::json::AddressType::Legacy,
			AddressType::P2shSegwit => bitcoincore_rpc::json::AddressType::P2shSegwit,
			AddressType::Bech32 => bitcoincore_rpc::json::AddressType::Bech32,
			AddressType::Bech32m => bitcoincore_rpc::json::AddressType::Bech32m,
		};
	}
}
pub struct Input {
	pub txid: String,
	pub vout: u32,
	pub sequence: Option<u32>,
}
impl Input {
	pub fn new(txid: String, vout: u32, sequence: Option<u32>) -> Self {
		Self { txid, vout, sequence }
	}
}
impl From<&Input> for bitcoincore_rpc::json::CreateRawTransactionInput {
	fn from(value: &Input) -> Self {
		bitcoincore_rpc::json::CreateRawTransactionInput {
			txid: bitcoin::Txid::from_str(&value.txid).expect("Invalid Txid"),
			vout: value.vout,
			sequence: value.sequence,
		}
	}
}
pub struct Address {
	pub internal: BitcoinAdrress,
}
