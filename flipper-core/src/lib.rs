use parity_scale_codec::{Decode, Encode, HasCompact};

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AsCompact<T: HasCompact>(#[codec(compact)] T);

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Call {
	Flip,
	Add(AsCompact<u32>),
	Multiply(AsCompact<u32>),
	Upgrade { password: Vec<u8>, payload: Vec<u8> },
	Kill { password: Vec<u8> },
}

#[cfg(test)]
mod tests {
	use crate::{AsCompact, Call};
	use parity_scale_codec::{Decode, Encode};
	use sp_core::Pair;
	use std::io::Read;

	type Address = sp_core::H256;
	type Signature = sp_core::H512;

	const ADMIN_SEED: &str =
		"dignity fatal coconut isolate evolve cloth scorpion squirrel sentence gate chase olympic";

	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
	struct TestExtrinsic {
		call: Call,
		signature: (Address, Signature),
	}

	fn output(value: &[u8]) -> String {
		value.iter().map(|b| format!("{:02x?}", b)).fold(
			String::with_capacity(value.len() * 2),
			|mut r, b| {
				r.push_str(&b);
				r
			},
		)
	}

	fn author_submit_extrinsic(
		call: &[u8],
		public_key: sp_core::sr25519::Public,
		signature: sp_core::sr25519::Signature,
		nonce: u32,
	) -> String {
		let address = sp_core::H256(public_key.0);
		let signature = sp_core::H512(signature.0);
		assert!(sp_io::crypto::sr25519_verify(
			&sp_core::sr25519::Signature::from_raw(signature.0),
			call,
			&sp_core::sr25519::Public::from_raw(address.0)
		));

		format!(
			r#"curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{{
	"jsonrpc":"2.0",
	"id":1,
	"method":"author_submitExtrinsic",
	"params": ["0x{}{}"]
}}'"#,
			output(call),
			output(&(address, signature, AsCompact(nonce)).encode())
		)
	}

	#[test]
	fn encode_flip() {
		let pair: sp_core::sr25519::Pair = sp_core::Pair::generate().0;
		let call = Call::Flip.encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 1))
	}

	#[test]
	fn encode_add() {
		let pair: sp_core::sr25519::Pair = sp_core::Pair::generate().0;
		let call = Call::Add(AsCompact(5)).encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 1))
	}

	#[test]
	fn encode_multiply() {
		let pair: sp_core::sr25519::Pair = sp_core::Pair::generate().0;
		let call = Call::Multiply(AsCompact(128)).encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 0))
	}

	#[test]
	fn upgrade() {
		let pair: sp_core::sr25519::Pair = sp_core::Pair::generate().0;
		let call = Call::Upgrade {
			password: "obsolescence".to_string().into_bytes(),
			payload: "wasm_blob".to_string().into_bytes(),
		}
		.encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 0))
	}

	#[test]
	fn upgrade_wasm_admin() {
		let runtime = std::fs::File::open(
			"/home/fb/PBA/flipper-runtime/target/release/wbuild/frameless-runtime/frameless_runtime.compact.compressed.wasm",
		)
		.unwrap();
		let mut reader = std::io::BufReader::new(runtime);
		let mut payload = Vec::new();
		reader.read_to_end(&mut payload).unwrap();

		let pair = sp_core::sr25519::Pair::from_phrase(ADMIN_SEED, None).unwrap().0;
		let call =
			Call::Upgrade { password: "obsolescence".to_string().into_bytes(), payload }.encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 0))
	}

	#[test]
	fn kills() {
		let pair = sp_core::sr25519::Pair::from_phrase(ADMIN_SEED, None).unwrap().0;
		let call = Call::Kill { password: "bye".to_string().into_bytes() }.encode();
		println!("{}", author_submit_extrinsic(&call, pair.public(), pair.sign(&call), 0))
	}
}
