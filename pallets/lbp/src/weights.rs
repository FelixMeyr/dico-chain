//! Autogenerated weights for lbp
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-09-08, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/dico-dev
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=lbp
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/lbp/src/weights.rs
// --template=./.maintain/pallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for lbp.
pub trait WeightInfo {
	fn create_lbp() -> Weight;
	fn exit_lbp() -> Weight;
	fn swap_exact_amount_supply() -> Weight;
	fn swap_exact_amount_target() -> Weight;
}

/// Weights for lbp using the dico-chain node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	fn create_lbp() -> Weight {
		(107_321_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	fn exit_lbp() -> Weight {
		(90_930_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn swap_exact_amount_supply() -> Weight {
		(91_872_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn swap_exact_amount_target() -> Weight {
		(88_355_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_lbp() -> Weight {
		(107_321_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	fn exit_lbp() -> Weight {
		(90_930_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(7 as Weight))
	}
	fn swap_exact_amount_supply() -> Weight {
		(91_872_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn swap_exact_amount_target() -> Weight {
		(88_355_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
}
