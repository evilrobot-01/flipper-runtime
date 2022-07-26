use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
enum Action {
	Flip,
	Add(u32),
	Multiply(u32),
}

// this extrinsic type does nothing other than fulfill the compiler.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic(Action);

#[cfg(test)]
mod tests {
	use crate::{Action, BasicExtrinsic};
	use parity_scale_codec::{Decode, Encode};

	#[derive(Encode)]
	struct Example {
		number: u8,
		is_cool: bool,
		optional: Option<u32>,
	}

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
		let encoded = Action::Flip.encode();
		assert_eq!("[00]", format!("{:02x?}", encoded));
		assert_eq!("0x00", output(&encoded));
	}

	#[test]
	fn encode_flip_extrinsic() {
		let encoded = BasicExtrinsic(Action::Flip).encode();
		println!("{:02x?}", encoded);
		assert_eq!("[00]", format!("{:02x?}", encoded));
		assert_eq!("0x00", output(&encoded));
	}

	#[test]
	fn encode_add() {
		let encoded = Action::Add(5).encode();
		assert_eq!("[01, 05, 00, 00, 00]", format!("{:02x?}", encoded));
		assert_eq!("0x0105000000", output(&encoded));
	}

	#[test]
	fn encode_multiply() {
		let encoded = Action::Multiply(128).encode();
		assert_eq!("[02, 80, 00, 00, 00]", format!("{:02x?}", encoded));
		assert_eq!("0x0280000000", output(&encoded));
	}

	#[test]
	fn flips() {
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
