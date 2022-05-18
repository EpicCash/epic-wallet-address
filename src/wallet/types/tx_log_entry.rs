// Copyright 2018 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Identifier, TxLogEntryType};
use chrono::prelude::*;
use epic_core::libtx::secp_ser;
use epic_core::ser;

use super::slate::versions::ser as dalek_ser;
use super::slate::ParticipantMessages;
use ed25519_dalek::PublicKey as DalekPublicKey;
use ed25519_dalek::Signature as DalekSignature;
use epic_util::secp::pedersen::Commitment;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// Optional transaction information, recorded when an event happens
/// to add or remove funds from a wallet. One Transaction log entry
/// maps to one or many outputs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxLogEntry {
	/// BIP32 account path used for creating this tx
	pub parent_key_id: Identifier,
	/// Local id for this transaction (distinct from a slate transaction id)
	pub id: u32,
	/// Slate transaction this entry is associated with, if any
	pub tx_slate_id: Option<Uuid>,
	/// Transaction type (as above)
	pub tx_type: TxLogEntryType,
	/// Address of the other party
	#[serde(default)]
	pub address: Option<String>,
	/// Time this tx entry was created
	/// #[serde(with = "tx_date_format")]
	pub creation_ts: DateTime<Utc>,
	/// Time this tx was confirmed (by this wallet)
	/// #[serde(default, with = "opt_tx_date_format")]
	pub confirmation_ts: Option<DateTime<Utc>>,
	/// Whether the inputs+outputs involved in this transaction have been
	/// confirmed (In all cases either all outputs involved in a tx should be
	/// confirmed, or none should be; otherwise there's a deeper problem)
	pub confirmed: bool,
	/// number of inputs involved in TX
	pub num_inputs: usize,
	/// number of outputs involved in TX
	pub num_outputs: usize,
	/// Amount credited via this transaction
	#[serde(with = "secp_ser::string_or_u64")]
	pub amount_credited: u64,
	/// Amount debited via this transaction
	#[serde(with = "secp_ser::string_or_u64")]
	pub amount_debited: u64,
	/// Fee
	#[serde(with = "secp_ser::opt_string_or_u64")]
	pub fee: Option<u64>,
	/// Cutoff block height
	#[serde(with = "secp_ser::opt_string_or_u64")]
	#[serde(default)]
	pub ttl_cutoff_height: Option<u64>,
	/// Message data, stored as json
	pub messages: Option<ParticipantMessages>,
	/// Location of the store transaction, (reference or resending)
	pub stored_tx: Option<String>,
	/// Associated kernel excess, for later lookup if necessary
	#[serde(with = "secp_ser::option_commitment_serde")]
	#[serde(default)]
	pub kernel_excess: Option<Commitment>,
	/// Height reported when transaction was created, if lookup
	/// of kernel is necessary
	#[serde(default)]
	pub kernel_lookup_min_height: Option<u64>,
	/// Additional info needed to stored payment proof
	#[serde(default)]
	pub payment_proof: Option<StoredProofInfo>,
}

impl TxLogEntry {
	/// Return a new blank with TS initialised with next entry
	pub fn new(parent_key_id: Identifier, t: TxLogEntryType, id: u32) -> Self {
		TxLogEntry {
			parent_key_id: parent_key_id,
			tx_type: t,
			id: id,
			address: None,
			tx_slate_id: None,
			creation_ts: Utc::now(),
			confirmation_ts: None,
			confirmed: false,
			amount_credited: 0,
			amount_debited: 0,
			num_inputs: 0,
			num_outputs: 0,
			fee: None,
			ttl_cutoff_height: None,
			messages: None,
			stored_tx: None,
			kernel_excess: None,
			kernel_lookup_min_height: None,
			payment_proof: None,
		}
	}

	/// Update confirmation TS with now
	pub fn update_confirmation_ts(&mut self) {
		self.confirmation_ts = Some(Utc::now());
	}
}

impl ser::Writeable for TxLogEntry {
	fn write<W: ser::Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
		writer.write_bytes(&serde_json::to_vec(self).map_err(|_| ser::Error::CorruptedData)?)
	}
}

impl ser::Readable for TxLogEntry {
	fn read(reader: &mut dyn ser::Reader) -> Result<TxLogEntry, ser::Error> {
		let data = reader.read_bytes_len_prefix()?;
		serde_json::from_slice(&data[..]).map_err(|_| ser::Error::CorruptedData)
	}
}

/// Payment proof information. Differs from what is sent via
/// the slate
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredProofInfo {
	/// receiver address
	#[serde(with = "dalek_ser::dalek_pubkey_serde")]
	pub receiver_address: DalekPublicKey,
	#[serde(with = "dalek_ser::option_dalek_sig_serde")]
	/// receiver signature
	pub receiver_signature: Option<DalekSignature>,
	/// sender address derivation path index
	pub sender_address_path: u32,
	/// sender address
	#[serde(with = "dalek_ser::dalek_pubkey_serde")]
	pub sender_address: DalekPublicKey,
	/// sender signature
	#[serde(with = "dalek_ser::option_dalek_sig_serde")]
	pub sender_signature: Option<DalekSignature>,
}

impl ser::Writeable for StoredProofInfo {
	fn write<W: ser::Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
		writer.write_bytes(&serde_json::to_vec(self).map_err(|_| ser::Error::CorruptedData)?)
	}
}

impl ser::Readable for StoredProofInfo {
	fn read(reader: &mut dyn ser::Reader) -> Result<StoredProofInfo, ser::Error> {
		let data = reader.read_bytes_len_prefix()?;
		serde_json::from_slice(&data[..]).map_err(|_| ser::Error::CorruptedData)
	}
}
