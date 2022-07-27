#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode, HasCompact};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_api::impl_runtime_apis;
use sp_block_builder::runtime_decl_for_BlockBuilder::BlockBuilder;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Extrinsic},
	transaction_validity::{TransactionSource, TransactionValidity, ValidTransaction},
	ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::OpaqueMetadata;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/*
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
	"jsonrpc":"2.0",
	"id":1,
	"method":"author_submitExtrinsic",
	"params": ["0x011403"]
}'

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
	"jsonrpc":"2.0",
	"id":1,
	"method":"state_getStorage",
	"params": ["0x626F6F6C65616E"]
}'
*/

/// An index to a block.
pub type BlockNumber = u32;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data-structures.
pub mod opaque {
	use sp_runtime::OpaqueExtrinsic;

	use super::*;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, BasicExtrinsic>;

	// This part is necessary for generating session keys in the runtime
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: AuraAppPublic,
			pub grandpa: GrandpaAppPublic,
		}
	}

	// Typically these are not implemented manually, but rather for the pallet associated with the
	// keys. Here we are not using the pallets, and these implementations are trivial, so we just
	// re-write them.
	pub struct AuraAppPublic;
	impl BoundToRuntimeAppPublic for AuraAppPublic {
		type Public = AuraId;
	}

	pub struct GrandpaAppPublic;
	impl BoundToRuntimeAppPublic for GrandpaAppPublic {
		type Public = sp_finality_grandpa::AuthorityId;
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("frameless-runtime"),
	impl_name: create_runtime_str!("frameless-runtime"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// The type that provides the genesis storage values for a new chain
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
pub struct GenesisConfig;

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		// we have nothing to put into storage in genesis, except this:
		storage.top.insert(well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec());

		Ok(())
	}
}

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, BasicExtrinsic>;

// this extrinsic type does nothing other than fulfill the compiler.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic {
	action: Action,
	salt: u8,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AsCompact<T: HasCompact>(#[codec(compact)] T);

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Action {
	Flip,
	Add(AsCompact<u32>),
	Multiply(AsCompact<u32>),
	Upgrade { password: Vec<u8>, payload: Vec<u8> },
	Kill { password: Vec<u8> },
}

impl Extrinsic for BasicExtrinsic {
	type Call = Action;
	type SignaturePayload = ();

	fn new(data: Self::Call, _: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(Self { action: data, salt: 0 })
	}
}

// 686561646572 raw storage key
pub const HEADER_KEY: [u8; 6] = *b"header";
const BIT_KEY: [u8; 3] = *b"bit";
const VALUE_KEY: [u8; 5] = *b"value";
const KILL_PASSWORD: [u8; 3] = *b"bye";
const UPGRADE_PASSWORD: [u8; 12] = *b"obsolescence";
const EMOJI: &str = "🤖";

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

impl_runtime_apis! {
// https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
impl sp_api::Core<Block> for Runtime {
	fn version() -> RuntimeVersion {
		VERSION
	}

	fn execute_block(block: Block) {
		info!(target: "frameless", "🖼{EMOJI}️ Entering execute_block. block: {:?}", block);
		Self::initialize_block(&block.header);

		for _transaction in block.extrinsics {
			// we have no notion of transaction, so nothing to execute yet.
			todo!();
		}


		Self::finalize_block();
	}

	fn initialize_block(header: &<Block as BlockT>::Header) {
		info!(target: "frameless", "🖼{EMOJI}️ Entering initialize_block. header: {:?}", header);
		sp_io::storage::set(&HEADER_KEY, &header.encode());
	}
}

// https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
impl sp_block_builder::BlockBuilder<Block> for Runtime {
	fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
		info!(target: "frameless", "🖼{EMOJI}️ Entering apply_extrinsic: {:?}", extrinsic);

		match extrinsic.action {
			Action::Flip => {
				let mut bit = sp_io::storage::get(&BIT_KEY)
					.map_or(false, |v| bool::decode(&mut &*v).unwrap_or(false));
					info!(target: "flipper", "{EMOJI} current bit: {bit}");
					bit = !bit;
					sp_io::storage::set(&BIT_KEY, &bit.encode());
					info!(target: "flipper", "{EMOJI} stored flipped bit: {bit}");
			},
			Action::Add(value) => {
					let existing = sp_io::storage::get(&VALUE_KEY)
					.map_or(0, |v| u32::decode(&mut &*v).unwrap_or(0));
					info!(target: "adder", "{EMOJI} existing value: {existing} supplied value: {}", value.0);
					let result = existing + value.0;
					sp_io::storage::set(&VALUE_KEY, &result.encode());
					info!(target: "adder", "{EMOJI} stored result: {result}");
			},
			Action::Multiply(value) => {
				let existing = sp_io::storage::get(&VALUE_KEY)
					.map_or(1, |v| u32::decode(&mut &*v).unwrap_or(1));
					info!(target: "multiplier", "{EMOJI} existing value: {existing} supplied value: {}", value.0);
					let result = existing * value.0;
				sp_io::storage::set(&VALUE_KEY, &result.encode());
					info!(target: "multiplier", "{EMOJI} stored result: {result}");
			},
			Action::Upgrade{password, payload, ..} => {
				if password == UPGRADE_PASSWORD {
						info!(target: "upgrader", "{EMOJI} upgrade initiated");
						sp_io::storage::set(sp_storage::well_known_keys::CODE.into(), &payload);
						}
					else {
						info!(target: "upgrader", "{EMOJI} upgrade rejected");
					}
			},
			Action::Kill{password, ..} => {
				if password == KILL_PASSWORD {
						info!(target: "killer", "{EMOJI} kill switch engaged");
						sp_io::storage::set(sp_storage::well_known_keys::CODE.into(), &vec![]);
						}
					else {
						info!(target: "killer", "{EMOJI} kill switch denied");
					}

			},
		}

		Ok(Ok(()))
	}

	fn finalize_block() -> <Block as BlockT>::Header {
		info!(target: "frameless", "🖼{EMOJI}️ Entering finalize block.");

		let raw_header = sp_io::storage::get(&HEADER_KEY)
			.expect("We initialized with header, it never got mutated, qed");
		sp_io::storage::clear(&HEADER_KEY);

		let mut header = <Block as BlockT>::Header::decode(&mut &*raw_header)
			.expect("we put a valid header in in the first place, qed");
		let raw_state_root = &sp_io::storage::root(sp_storage::StateVersion::default())[..];

		header.state_root = sp_core::H256::decode(&mut &raw_state_root[..]).unwrap();
		header
	}

	// This runtime does not expect any inherents so it does not insert any into blocks it builds.
	fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
		info!(target: "frameless", "🖼{EMOJI}️ Entering inherent_extrinsics.");
		Vec::new()
	}

	// This runtime does not expect any inherents, so it does not do any inherent checking.
	fn check_inherents(
		block: Block,
		_data: sp_inherents::InherentData,
	) -> sp_inherents::CheckInherentsResult {
		info!(target: "frameless", "🖼{EMOJI}️ Entering check_inherents. block: {:?}", block);
		sp_inherents::CheckInherentsResult::default()
	}
}

impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
	fn validate_transaction(
		source: TransactionSource,
		tx: <Block as BlockT>::Extrinsic,
		block_hash: <Block as BlockT>::Hash,
	) -> TransactionValidity {
		info!(target: "frameless", "🖼{EMOJI}️ Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}", source, tx, block_hash);

		// we don't know how to validate this -- It should be fine??
		let data = tx.action;
		Ok(ValidTransaction { provides: vec![data.encode()], ..Default::default() })
	}
}

// Ignore everything after this.

impl sp_api::Metadata<Block> for Runtime {
	fn metadata() -> OpaqueMetadata {
		OpaqueMetadata::new(vec![0])
	}
}

impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
	fn offchain_worker(_header: &<Block as BlockT>::Header) {
		// we do not do anything.
	}
}

impl sp_session::SessionKeys<Block> for Runtime {
	fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
		info!(target: "frameless", "🖼{EMOJI}️ Entering generate_session_keys. seed: {:?}", seed);
		opaque::SessionKeys::generate(seed)
	}

	fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
		opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
	}
}

// Here is the Aura API for the sake of making this runtime work with the node template node
impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
	fn slot_duration() -> sp_consensus_aura::SlotDuration {
		// Three-second blocks
		sp_consensus_aura::SlotDuration::from_millis(3000)
	}

	fn authorities() -> Vec<AuraId> {
		// The only authority is Alice. This makes things work nicely in `--dev` mode
		use sp_application_crypto::ByteArray;

		vec![AuraId::from_slice(
			&hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
				.to_vec(),
		)
		.unwrap()]
	}
}

impl sp_finality_grandpa::GrandpaApi<Block> for Runtime {
	fn grandpa_authorities() -> sp_finality_grandpa::AuthorityList {
		use sp_application_crypto::ByteArray;
		vec![(
			sp_finality_grandpa::AuthorityId::from_slice(
				&hex_literal::hex!(
					"88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee"
				)
				.to_vec(),
			)
			.unwrap(),
			1,
		)]
	}

	fn current_set_id() -> sp_finality_grandpa::SetId {
		0u64
	}

	fn submit_report_equivocation_unsigned_extrinsic(
		_equivocation_proof: sp_finality_grandpa::EquivocationProof<
			<Block as BlockT>::Hash,
			sp_runtime::traits::NumberFor<Block>,
		>,
		_key_owner_proof: sp_finality_grandpa::OpaqueKeyOwnershipProof,
	) -> Option<()> {
		None
	}

	fn generate_key_ownership_proof(
		_set_id: sp_finality_grandpa::SetId,
		_authority_id: sp_finality_grandpa::AuthorityId,
	) -> Option<sp_finality_grandpa::OpaqueKeyOwnershipProof> {
		None
	}
}
}
