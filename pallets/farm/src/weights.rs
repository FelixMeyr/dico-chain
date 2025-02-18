//! Autogenerated weights for farm
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-08-20, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/dico-dev
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=farm
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/farm/src/weights.rs
// --template=./.maintain/pallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pool.
pub trait WeightInfo {
	fn set_halving_period() -> Weight;
	fn set_dico_per_block() -> Weight;
	fn set_start_block() -> Weight;
	fn create_pool() -> Weight;
	fn update_pool_alloc_point() -> Weight;
	fn deposit_lp() -> Weight;
	fn withdraw_lp() -> Weight;
}

/// Weights for pool using the dico-chain node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	fn set_halving_period() -> Weight {
		(12_293_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_dico_per_block() -> Weight {
		(12_704_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_start_block() -> Weight {
		(12_354_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn create_pool() -> Weight {
		(35_337_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn update_pool_alloc_point() -> Weight {
		(61_796_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn deposit_lp() -> Weight {
		(77_686_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn withdraw_lp() -> Weight {
		(142_016_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn set_halving_period() -> Weight {
		(12_293_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_dico_per_block() -> Weight {
		(12_704_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_start_block() -> Weight {
		(12_354_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn create_pool() -> Weight {
		(35_337_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	fn update_pool_alloc_point() -> Weight {
		(61_796_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	fn deposit_lp() -> Weight {
		(77_686_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	fn withdraw_lp() -> Weight {
		(142_016_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(9 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
}
