use parity_scale_codec::{Decode, Encode, HasCompact};

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AsCompact<T: HasCompact>(#[codec(compact)] T);

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Action {
	Flip,
	Add(AsCompact<u32>),
	Multiply(AsCompact<u32>),
	Upgrade { password: Vec<u8>, payload: Vec<u8> },
	Kill { password: Vec<u8> },
}

#[cfg(test)]
mod tests {
	use crate::{Action, AsCompact};
	use parity_scale_codec::{Decode, Encode};
	use std::io::Read;

	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
	struct TestExtrinsic {
		action: Action,
		salt: u8,
	}

	fn output(value: &Vec<u8>) -> String {
		value.iter().map(|b| format!("{:02x?}", b)).fold(
			String::with_capacity(value.len() * 2),
			|mut r, b| {
				r.push_str(&b);
				r
			},
		)
	}

	fn author_submit_extrinsic(action: &str, salt: u8) -> String {
		format!(
			r#"curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{{
	"jsonrpc":"2.0",
	"id":1,
	"method":"author_submitExtrinsic",
	"params": ["0x{action}{}"]
}}'"#,
			output(&salt.encode())
		)
	}

	#[test]
	fn encode_flip() {
		let encoded = Action::Flip.encode();
		assert_eq!("[00]", format!("{:02x?}", encoded));
		let encoded = output(&encoded);
		assert_eq!("00", encoded);
		println!("{}", author_submit_extrinsic(&encoded, 0))
	}

	#[test]
	fn encode_add() {
		let encoded = Action::Add(AsCompact(5)).encode();
		assert_eq!("[01, 14]", format!("{:02x?}", encoded));
		let encoded = output(&encoded);
		assert_eq!("0114", encoded);
		println!("{}", author_submit_extrinsic(&encoded, 0))
	}

	#[test]
	fn encode_multiply() {
		let encoded = Action::Multiply(AsCompact(128)).encode();
		assert_eq!("[02, 01, 02]", format!("{:02x?}", encoded));
		let encoded = output(&encoded);
		assert_eq!("020102", encoded);
		println!("{}", author_submit_extrinsic(&encoded, 0))
	}

	#[test]
	fn upgrade() {
		let encoded = Action::Upgrade {
			password: "obsolescence".to_string().into_bytes(),
			payload: "wasm_blob".to_string().into_bytes(),
		}
		.encode();
		assert_eq!(
			"[03, 30, 6f, 62, 73, 6f, 6c, 65, 73, 63, 65, 6e, 63, 65, 24, 77, 61, 73, 6d, 5f, 62, 6c, 6f, 62]",
			format!("{:02x?}", encoded)
		);
		let encoded = output(&encoded);
		assert_eq!("03306f62736f6c657363656e6365247761736d5f626c6f62", encoded);
		println!("{}", author_submit_extrinsic(&encoded, 0))
	}

	#[test]
	fn upgrade_wasm() {
		let runtime = std::fs::File::open(
			"/home/fb/PBA/flipper-runtime/target/release/wbuild/frameless-runtime/frameless_runtime.compact.compressed.wasm",
		)
		.unwrap();
		let mut reader = std::io::BufReader::new(runtime);
		let mut payload = Vec::new();
		reader.read_to_end(&mut payload).unwrap();

		let encoded =
			Action::Upgrade { password: "obsolescence".to_string().into_bytes(), payload }.encode();
		println!("{}", author_submit_extrinsic(&*output(&encoded), 0));
	}

	#[test]
	fn kills() {
		let encoded = Action::Kill { password: "bye".to_string().into_bytes() }.encode();
		assert_eq!("[04, 0c, 62, 79, 65]", format!("{:02x?}", encoded));
		let encoded = output(&encoded);
		assert_eq!("040c627965", encoded);
		println!("{}", author_submit_extrinsic(&encoded, 0));
	}
}
