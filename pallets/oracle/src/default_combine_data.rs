use crate::{Config, MomentOf, TimestampedValueOf};
use frame_support::log;
use sp_runtime::{traits::Member, DispatchResult, RuntimeDebug};
use sp_std::{fmt::Debug, marker, prelude::*};

use frame_support::traits::{Get, Time};
use orml_traits::CombineData;

/// Sort by value and returns median timestamped value.
/// Returns prev_value if not enough valid values.
pub struct DefaultCombineData<T, MinimumCount, ExpiresIn, I = ()>(marker::PhantomData<(T, I, MinimumCount, ExpiresIn)>);

impl<T, I, MinimumCount, ExpiresIn> CombineData<<T as Config<I>>::OracleKey, TimestampedValueOf<T, I>>
	for DefaultCombineData<T, MinimumCount, ExpiresIn, I>
where
	T: Config<I>,
	I: 'static,
	MinimumCount: Get<u32>,
	ExpiresIn: Get<MomentOf<T, I>>,
{
	fn combine_data(
		_key: &<T as Config<I>>::OracleKey,
		mut values: Vec<TimestampedValueOf<T, I>>,
		prev_value: Option<TimestampedValueOf<T, I>>,
	) -> Option<TimestampedValueOf<T, I>> {
		let expires_in = ExpiresIn::get();
		let now = T::Time::now();
		values.retain(|x| x.timestamp + expires_in > now);

		let count = values.len() as u32;
		let minimum_count = MinimumCount::get();
		log::info!(
			"-----------combine_data: {:?} {:?} {:?}----------",
			values,
			prev_value,
			minimum_count
		);
		if count < minimum_count {
			return prev_value;
		}

		values.sort_by(|a, b| a.value.cmp(&b.value));

		let median_index = count / 2;
		Some(values[median_index as usize].clone())
	}
}
