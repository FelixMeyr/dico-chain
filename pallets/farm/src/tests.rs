//! Unit tests for the farm module.

#![cfg(test)]

use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, Origin,
	System, Test, DOT, ALICE, BOB, USDT, DICO, Farm,
	DEFAULT_ASSET_AMOUNT, PDOTUSDT,
};
use frame_support::{assert_ok};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn last_events(n: usize) -> Vec<TestEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<TestEvent>) {
	assert_eq!(last_events(e.len()), e);
}


#[test]
fn set_halving_period_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Farm::set_halving_period(
			Origin::signed(ALICE),
			1000
		));

		assert_eq!(HalvingPeriod::<Test>::get(), 1000);
		expect_events(vec![Event::HalvingPeriodIsSet(1000).into()]);
	});
}


#[test]
fn set_dico_per_block_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Farm::set_dico_per_block(
			Origin::signed(ALICE),
			10000
		));

		assert_eq!(DicoPerBlock::<Test>::get(), 10000);
		expect_events(vec![Event::DicoPerBlockIsSet(10000).into()]);
	});
}

#[test]
fn set_start_block_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Farm::set_start_block(
			Origin::signed(ALICE),
			10000
		));

		assert_eq!(StartBlock::<Test>::get(), 10000);
		expect_events(vec![Event::StartBlockIsSet(10000).into()]);
	});
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1000u32), BlockNumber::from(10000u32));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			PDOTUSDT,
			alloc_point,
			start_block,
			end_block,
		));

		assert_eq!(TotalAllocPoint::<Test>::get(), alloc_point);
		let pool_info = PoolInfo::new(PDOTUSDT, alloc_point, start_block, start_block, end_block);
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		expect_events(vec![Event::PoolCreated(0).into()]);
	});
}


#[test]
fn update_pool_alloc_point_should_work() {
	new_test_ext().execute_with(|| {
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1000u32), BlockNumber::from(10000u32));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			PDOTUSDT,
			alloc_point,
			start_block,
			end_block
		));

		assert_eq!(TotalAllocPoint::<Test>::get(), alloc_point);
		let pool_info = PoolInfo::new(PDOTUSDT, alloc_point, start_block, start_block, end_block);
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		expect_events(vec![Event::PoolCreated(0).into()]);

		assert_ok!(Farm::update_pool_alloc_point(
			Origin::signed(ALICE),
			0,
			10000u128
		));

		assert_eq!(TotalAllocPoint::<Test>::get(), 10000u128);
		let pool_info = PoolInfo::new(PDOTUSDT, 10000u128, start_block, start_block, end_block);
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		expect_events(vec![Event::PoolAllocPointUpdated(0, 10000u128).into()]);
	});
}


#[test]
fn deposit_lp_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Farm::set_halving_period(
			Origin::signed(ALICE),
			5000
		));

		assert_ok!(Farm::set_dico_per_block(
			Origin::signed(ALICE),
			100_000_000_000_000
		));

		assert_ok!(Farm::set_start_block(
			Origin::signed(ALICE),
			1000
		));

		let liquidity_id: AssetId = DOT;
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1000u32), BlockNumber::from(20000u32));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			liquidity_id,
			alloc_point,
			start_block,
			end_block
		));

		assert_eq!(TotalAllocPoint::<Test>::get(), alloc_point);
		let pool_info = PoolInfo::new(liquidity_id, alloc_point, 1000, start_block, end_block);
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);
		expect_events(vec![Event::PoolCreated(0).into()]);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(ALICE),
			0,
			100_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1000, start_block, end_block);
		pool_info.total_amount = 100_000_000_000_000;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(100_000_000_000_000, 0);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let module_id_account = Farm::account_id();

		assert_eq!(Currency::free_balance(liquidity_id, &module_id_account), 100_000_000_000_000);
		assert_eq!(Currency::free_balance(liquidity_id, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);

		expect_events(vec![Event::LpDeposited(ALICE, 0, 100_000_000_000_000).into()]);

		System::set_block_number(16001);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(BOB),
			0,
			200_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 300_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8750125000000000u64);
		pool_info.last_reward_block = 16001;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(100_000_000_000_000, 0);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let participant = Participant::new(200_000_000_000_000, 1750025000000000000);
		assert_eq!(Participants::<Test>::get(0, BOB).unwrap(), participant);

		System::set_block_number(30000);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(BOB),
			0,
			200_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 500_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8916750000000000u64);
		pool_info.last_reward_block = end_block;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);
	});
}

#[test]
fn deposit_lp_should_work2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Farm::set_halving_period(
			Origin::signed(ALICE),
			5000
		));

		assert_ok!(Farm::set_dico_per_block(
			Origin::signed(ALICE),
			100_000_000_000_000
		));

		assert_ok!(Farm::set_start_block(
			Origin::signed(ALICE),
			1000
		));

		let liquidity_id: AssetId = DOT;
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1500u32), BlockNumber::from(20000u32));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			liquidity_id,
			alloc_point,
			start_block,
			end_block
		));

		assert_eq!(TotalAllocPoint::<Test>::get(), alloc_point);
		let pool_info = PoolInfo::new(liquidity_id, alloc_point, start_block, start_block, end_block);
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);
		expect_events(vec![Event::PoolCreated(0).into()]);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(ALICE),
			0,
			100_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, start_block, start_block, end_block);
		pool_info.total_amount = 100_000_000_000_000;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(100_000_000_000_000, 0);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let module_id_account = Farm::account_id();

		assert_eq!(Currency::free_balance(liquidity_id, &module_id_account), 100_000_000_000_000);
		assert_eq!(Currency::free_balance(liquidity_id, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);

		expect_events(vec![Event::LpDeposited(ALICE, 0, 100_000_000_000_000).into()]);

		System::set_block_number(16001);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(BOB),
			0,
			200_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 300_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8250125000000000u64);
		pool_info.last_reward_block = 16001;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(100_000_000_000_000, 0);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let participant = Participant::new(200_000_000_000_000, 1650025000000000000);
		assert_eq!(Participants::<Test>::get(0, BOB).unwrap(), participant);

		System::set_block_number(30000);

		assert_ok!(Farm::deposit_lp(
			Origin::signed(BOB),
			0,
			200_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 500_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8416750000000000u64);
		pool_info.last_reward_block = end_block;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);
	});
}


#[test]
fn withdraw_lp_should_work() {
	new_test_ext().execute_with(|| {
		let liquidity_id: AssetId = DOT;
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1000u32), BlockNumber::from(20000u32));

		assert_ok!(Farm::set_halving_period(
			Origin::signed(ALICE),
			5000
		));

		assert_ok!(Farm::set_dico_per_block(
			Origin::signed(ALICE),
			100_000_000_000_000
		));

		assert_ok!(Farm::set_start_block(
			Origin::signed(ALICE),
			1000
		));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			liquidity_id,
			alloc_point,
			start_block,
			end_block
		));

		assert_ok!(Farm::deposit_lp(
			Origin::signed(ALICE),
			0,
			100_000_000_000_000
		));

		System::set_block_number(16001);

		assert_ok!(Farm::withdraw_lp(
			Origin::signed(ALICE),
			0,
			0
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 100_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8750125000000000u64);
		pool_info.last_reward_block = 16001;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(100_000_000_000_000, 875012500000000000);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let module_id_account = Farm::account_id();

		assert_eq!(Currency::free_balance(liquidity_id, &module_id_account), 100_000_000_000_000);
		assert_eq!(Currency::free_balance(0, &module_id_account), 0);
	});
}

#[test]
fn withdraw_lp_2_should_work() {
	new_test_ext().execute_with(|| {
		let liquidity_id: AssetId = DOT;
		let alloc_point = 1000u128;
		let (start_block, end_block) = (BlockNumber::from(1000u32), BlockNumber::from(20000u32));

		assert_ok!(Farm::set_halving_period(
			Origin::signed(ALICE),
			5000
		));

		assert_ok!(Farm::set_dico_per_block(
			Origin::signed(ALICE),
			100_000_000_000_000
		));

		assert_ok!(Farm::set_start_block(
			Origin::signed(ALICE),
			1000
		));

		assert_ok!(Farm::create_pool(
			Origin::signed(ALICE),
			liquidity_id,
			alloc_point,
			start_block,
			end_block
		));

		assert_ok!(Farm::deposit_lp(
			Origin::signed(ALICE),
			0,
			100_000_000_000_000
		));

		System::set_block_number(16001);

		assert_ok!(Farm::withdraw_lp(
			Origin::signed(ALICE),
			0,
			50_000_000_000_000
		));

		let mut pool_info = PoolInfo::new(liquidity_id, alloc_point, 1, start_block, end_block);
		pool_info.total_amount = 50_000_000_000_000;
		pool_info.acc_dico_per_share = Balance::from(8750125000000000u64);
		pool_info.last_reward_block = 16001;
		assert_eq!(Pools::<Test>::get(0).unwrap(), pool_info);

		let participant = Participant::new(50_000_000_000_000, 437506250000000000);
		assert_eq!(Participants::<Test>::get(0, ALICE).unwrap(), participant);

		let module_id_account = Farm::account_id();

		assert_eq!(Currency::free_balance(liquidity_id, &module_id_account), 50_000_000_000_000);
		assert_eq!(Currency::free_balance(liquidity_id, &ALICE), DEFAULT_ASSET_AMOUNT - 50_000_000_000_000);
		assert_eq!(Currency::free_balance(0, &module_id_account), 0);
	});
}



