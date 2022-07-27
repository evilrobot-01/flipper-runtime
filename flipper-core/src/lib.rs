use parity_scale_codec::{Decode, Encode, HasCompact};

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AsCompact<T: HasCompact>(#[codec(compact)] T);

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Action {
	Flip { salt: u8 },
	Add { value: AsCompact<u32>, salt: u8 },
	Multiply { value: AsCompact<u32>, salt: u8 },
	Upgrade { password: Vec<u8>, payload: Vec<u8>, salt: u8 },
	Kill { password: Vec<u8>, salt: u8 },
}

#[cfg(test)]
mod tests {
	use crate::{Action, AsCompact};
	use parity_scale_codec::{Decode, Encode};
	use std::io::Read;

	#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
	struct TestExtrinsic(Action);

	fn output(value: &Vec<u8>) -> String {
		format!(
			"0x{}",
			value.iter().map(|b| format!("{:02x?}", b)).fold(
				String::with_capacity(value.len() * 2),
				|mut r, b| {
					r.push_str(&b);
					r
				}
			)
		)
	}

	#[test]
	fn encode_flip() {
		let encoded = Action::Flip { salt: 0 }.encode();
		assert_eq!("[00, 00]", format!("{:02x?}", encoded));
		assert_eq!("0x0000", output(&encoded));

		let encoded = Action::Flip { salt: 1 }.encode();
		assert_eq!("0x0001", output(&encoded));
	}

	#[test]
	fn encode_add() {
		let encoded = Action::Add { value: AsCompact(5), salt: 3 }.encode();
		assert_eq!("[01, 14, 03]", format!("{:02x?}", encoded));
		assert_eq!("0x011403", output(&encoded));
	}

	#[test]
	fn encode_multiply() {
		let encoded = Action::Multiply { value: AsCompact(128), salt: 0 }.encode();
		assert_eq!("[02, 01, 02, 00]", format!("{:02x?}", encoded));
		assert_eq!("0x02010200", output(&encoded));
	}

	#[test]
	fn upgrade() {
		let encoded = Action::Upgrade {
			password: "obsolescence".to_string().into_bytes(),
			payload: "wasm_blob".to_string().into_bytes(),
			salt: 0,
		}
		.encode();
		assert_eq!(
			"[03, 30, 6f, 62, 73, 6f, 6c, 65, 73, 63, 65, 6e, 63, 65, 24, 77, 61, 73, 6d, 5f, 62, 6c, 6f, 62, 00]",
			format!("{:02x?}", encoded)
		);
		assert_eq!("0x03306f62736f6c657363656e6365247761736d5f626c6f6200", output(&encoded));
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
			Action::Upgrade { password: "obsolescence".to_string().into_bytes(), payload, salt: 0 }
				.encode();
		println!("{}", output(&encoded));
	}

	#[test]
	fn kills() {
		let encoded = Action::Kill { password: "bye".to_string().into_bytes(), salt: 0 }.encode();
		assert_eq!("[04, 0c, 62, 79, 65, 00]", format!("{:02x?}", encoded));
		assert_eq!("0x040c62796500", output(&encoded));
	}

	#[test]
	fn flips_storage() {
		let mut e = sp_io::TestExternalities::new_empty();
		e.execute_with(|| {
			const KEY: [u8; 3] = *b"BIT";
			let bit = sp_io::storage::get(&KEY)
				.map_or(false, |v| bool::decode(&mut &*v).unwrap_or(false));
			assert_eq!(false, bit);
			sp_io::storage::set(&KEY, &(!bit).encode());
		});
	}

	// 362
	// 363 #[test]
	// 364 fn test() {
	// 365     let mut e = sp_io::TestExternalities::new_empty();
	// 366     e.execute_with(|| {
	// 	367         let sc = SignedCall { operation: Operation::Add(5), ..SignedCall::default() };
	// 	368         let e = BasicExtrinsic::new( sc, None ).unwrap();
	// 	369         println!("{:?}", e.encode());
	// 	370         Runtime::apply_extrinsic(e);
	// 	371         assert_eq!(u32::decode(&mut &*sp_io::storage::get(&STATE_VALUE_0_KEY).unwrap()).unwrap(), 5);
	// 	372
	// 	373         let sc = SignedCall { operation: Operation::Add(7), ..SignedCall::default() };
	// 	374         let e = BasicExtrinsic::new( sc, None ).unwrap();
	// 	375         println!("{:?}", e.encode());
	// 	376         Runtime::apply_extrinsic(e);
	// 	377         assert_eq!(u32::decode(&mut &*sp_io::storage::get(&STATE_VALUE_0_KEY).unwrap()).unwrap(), 12);
	// 	378     });
	// 379 }
}
