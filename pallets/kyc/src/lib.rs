// Copyright 2021 DICO  Developer.
// This file is part of DICO

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Know Your Customer (KYC)
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//!
//! * `set_kyc` - Set the associated KYC of an account; a small deposit is reserved if not already taken.
//! * `clear_kyc` - Remove an account's associated KYC of an account; the deposit is returned.
//! * `request_judgement` - Request a judgement from a IAS, paying a fee.
//! * `apply_certification` - apply certification
//!
//! #### For identity authentication service(IAS)
//!
//! * `ias_set_fee` - Set the fee required to be paid for a judgement to be given by the IAS.
//! * `ias_provide_judgement` - Provide a judgement to an KYC account.
//! * `ias_request_sword_holder` - Certification is handed over to sword holder
//!
//!
//!
//! #### For sword holder
//!
//! * `sword_holder_provide_judgement` - Provide a judgement to an kyc account.
//! * `sword_holder_set_fee` -  Set the fee required to be paid for a judgement to be given by the sword holder.
//!
//!
//! #### For sudo super-users(Sudo)
//! * `add_ias` - Add a new ias provider to the system. tips: Formed by election
//! * `add_sword_holder` - Add a new sword holder to the system. tips: Formed by election
//! * `kill_ias` - Forcibly remove the associated ias; the deposit is lost.
//! * `kill_sword_holder` - Forcibly remove the associated sword holder; the deposit is lost.
//! * `remove_kyc` - Forcibly remove kyc from kyc list and add to black list.


#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod types;
pub mod weights;
pub mod traits;
use traits::KycHandler;
use crate::types::{KYCInfo, AreaCode};
use frame_system::Account;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::*;
	use crate::weights::WeightInfo;
	use codec::{Decode, Encode};
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{
			BalanceStatus, Currency, EnsureOrigin, ExistenceRequirement, Get, OnUnbalanced, Randomness,
			ReservableCurrency, WithdrawReasons,
		},PalletId,
	};
	use sp_std::prelude::*;
	use sp_std::{iter::once, ops::Add};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{
		AppendZerosInput, CheckedAdd, CheckedDiv, SaturatedConversion, Saturating, StaticLookup, Zero,
	};

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type NegativeImbalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Something that provides randomness in the runtime.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The amount held on deposit for a registered user.
		#[pallet::constant]
		type BasicDeposit: Get<BalanceOf<Self>>;

		/// The amount held on deposit for a ias/sword holder
		#[pallet::constant]
		type ServiceDeposit: Get<BalanceOf<Self>>;

		/// Maxmimum number of IAS
		#[pallet::constant]
		type MaxIAS: Get<u32>;

		/// MaxSupervisor:
		#[pallet::constant]
		type MaxSwordHolder: Get<u32>;

		/// What to do with slashed funds.
		type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The origin which may forcibly set or remove a ias/sword holder. Root can always do
		/// this.
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The IAS origin。
		/// Root can always do this.
		type IASOrigin: EnsureOrigin<Self::Origin>;

		/// The origin  Root can always do this.
		type SwordHolderOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Keeps track of the Nonce used in the randomness generator.
	#[pallet::storage]
	#[pallet::getter(fn get_nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// area data of user account
	#[pallet::storage]
	#[pallet::getter(fn area_data)]
	pub(super) type AreaData<T: Config> = StorageMap<_, Twox64Concat, AreaCode, u64, ValueQuery>;

	/// kyc of account
	#[pallet::storage]
	#[pallet::getter(fn kyc)]
	pub(super) type KYCOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Registration<BalanceOf<T>>, OptionQuery>;

	/// the black list of kyc user
	#[pallet::storage]
	#[pallet::getter(fn blacklist)]
	pub(super) type BlackListOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BlackInfo<BalanceOf<T>>, OptionQuery>;

	/// List of identity authentication service(IAS) in a  kyc field
	#[pallet::storage]
	#[pallet::getter(fn ias_list)]
	pub(super) type IASListOf<T: Config> =
		StorageMap<_, Twox64Concat, KYCFields, Vec<Option<IASInfo<BalanceOf<T>, T::AccountId>>>, ValueQuery>;

	/// List of SwordHolder in a  kyc field
	#[pallet::storage]
	#[pallet::getter(fn sword_holder)]
	pub(super) type SwordHolderOf<T: Config> =
		StorageMap<_, Twox64Concat, KYCFields, Vec<Option<IASInfo<BalanceOf<T>, T::AccountId>>>, ValueQuery>;

	/// Records Of IAS/SwordHolder
	#[pallet::storage]
	#[pallet::getter(fn records)]
	pub(super) type RecordsOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<Record<T::AccountId>>, ValueQuery>;

	/// Unique information storage filtering
	#[pallet::storage]
	#[pallet::getter(fn unique_id)]
	pub(super) type UniqueIdOf<T: Config> = StorageMap<_, Twox64Concat, KYCFields, Vec<Data>, ValueQuery>;

	/// message: (sender, recipient -> data)
	#[pallet::storage]
	#[pallet::getter(fn message)]
	pub(super) type MessageList<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, T::AccountId, Vec<Message>, ValueQuery>;

	///ApplicationFormList: AccountId -> Vec<ApplicationForm>
	#[pallet::storage]
	#[pallet::getter(fn application_form)]
	pub(super) type ApplicationFormList<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<Option<ApplicationForm<BalanceOf<T>, T::AccountId>>>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		BalanceOf<T> = "Balance"
	)]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A identity authentication service(IAS)  provider was added.\[kyc_index\]
		IASAdded(KYCIndex),
		/// A sword holder  provider was added. \[kyc_index\]
		SwordHolderAdded(KYCIndex),
		/// A ApplyCertification . \[kyc_index, kyc_index\]
		ApplyCertification(T::AccountId),
		/// A kyc was set or reset (which will remove all judgements). \[who\]
		KYCSet(T::AccountId),
		/// IAS killed. \[who\]
		IASKilled(T::AccountId),
		/// remove kyc. \[who\]
		KYCRemove(T::AccountId),
		/// SetFee
		SetFee(BalanceOf<T>),
		/// SwordHolder killed. \[who\]
		SwordHolderKilled(T::AccountId),
		/// A kyc was cleared, and the given balance returned. \[who, deposit\]
		KYCCleared(T::AccountId, BalanceOf<T>),
		/// Randomly get identity authentication service(IAS) provider.\[kyc_index,exchange_key\]
		GetIAS(KYCIndex, ExchangeKey),
		/// Randomly get a sword holder  provider. \[kyc_index, exchange_key\]
		GetSwordHolder(KYCIndex, ExchangeKey),
		/// A judgement was asked from a registrar. \[who, kyc_index\]
		JudgementRequested(T::AccountId, KYCIndex),
		/// A judgement request was retracted. \[who, kyc_index\]
		JudgementUnrequested(T::AccountId, KYCIndex),
		/// A judgement was given by a registrar. \[target, kyc_index\]
		JudgementGiven(T::AccountId, KYCIndex),
		/// A authentication was given by a registrar. \[target, kyc_index\]
		AuthenticationGiven(T::AccountId, KYCIndex),
		/// A judgement was asked from a registrar. \[who, kyc_index\]
		IASJudgementRequested(T::AccountId, KYCIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Nonce has overflowed past u64 limits
		NonceOverflow,
		/// Count Overflow
		CountOverflow,
		/// Account already exists
		AccountExists,
		/// Insufficient permissions
		InsufficientPermissions,
		/// out of bounds
		OutofBounds,
		/// Fee Non Negative
		FeeNonNegative,
		/// Maximum amount of IAS/SwordHolder reached. Cannot add any more.
		TooManyAccount,
		/// Account isn't found.
		NotFound,
		/// NO IAS
		NoIAS,
		/// No IAS OrS wordHolder
		NoIASOrSwordHolder,
		/// Fee is not enough.
		FeeNotEnough,
		/// Fee is changed.
		FeeChanged,
		/// No KYC found.
		NoKYC,
		/// not Unique identification code
		NotUniqueID,
		/// not contains Unique identification code
		NotContainsUniqueID,
		/// No application
		NoApplication,
		/// has been blacklisted
		Blacklisted,
		/// Repeat application
		RepeatApplication,
		/// IAS Judgement given.
		JudgementGiven,
		///  SwordHolder Authentication given
		AuthenticationGiven,
		/// Invalid judgement.
		InvalidJudgement,
		/// Invalid fee
		InvalidFee,
		/// Sticky judgement.
		StickyJudgement,
		/// The authentication's is pending.
		PendingAuthentication,
		/// Sticky judgement.
		EmptyIndex,
		/// The index is invalid.
		InvalidIndex,
		/// The target is invalid.
		InvalidTarget,
		/// The kyc field is invalid.
		InvalidKYCField,
		/// The kyc field is contained in list.
		KYCFieldFound,
		/// The kyc of account is contained in list.
		KYCFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///  Add a identity authentication service(IAS)  provider to the system
		///
		/// Selection through Congress
		///
		/// - `ias_index`: KYCIndex.
		/// - `ias_info`: IASInfo<BalanceOf<T>, T::AccountId>.
		///
		/// Emits `IASAdded` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::add_ias(
			T::MaxIAS::get().into()
		))]
		pub fn add_ias(
			origin: OriginFor<T>,
			ias_index: KYCIndex,
			ias_info: IASInfo<BalanceOf<T>, T::AccountId>,
		) -> DispatchResultWithPostInfo {
			T::IASOrigin::ensure_origin(origin)?;
			Self::add_kyc_service(ias_index, ias_info, true)?;
			Self::deposit_event(Event::<T>::IASAdded(ias_index));
			Ok(().into())
		}

		///  Add a sword holder  provider to the system
		///
		/// Selection through Congress
		///
		/// - `ias_index`: KYCIndex.
		/// - `ias_info`: IASInfo<BalanceOf<T>, T::AccountId>.
		/// - `reserve_value`: Balance, The amount that each service provider needs to reserve
		///   balance, and return it after exiting.
		///
		/// Emits `IASAdded` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::add_sword_holder(
			T::MaxSwordHolder::get().into()
		))]
		pub fn add_sword_holder(
			origin: OriginFor<T>,
			sword_index: KYCIndex,
			sword_info: IASInfo<BalanceOf<T>, T::AccountId>,
		) -> DispatchResultWithPostInfo {
			T::IASOrigin::ensure_origin(origin)?;
			Self::add_kyc_service(sword_index, sword_info, false)?;
			Self::deposit_event(Event::<T>::IASAdded(sword_index));
			Ok(().into())
		}

		/// ForceOrigin delete a kyc of accounts and add blacklist
		///
		/// This requires Congress to operate
		///
		/// - `target`: The deleted user.
		/// - `black`: The reason for joining blacklist
		///
		/// Emits `KYCRemove` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::remove_kyc(
			T::MaxSwordHolder::get().into(), // R
		))]
		pub fn remove_kyc(
			origin: OriginFor<T>,
			target: T::AccountId,
			black: Black<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;
			ensure!(!<BlackListOf<T>>::contains_key(&target), Error::<T>::Blacklisted);
			let _registration = <KYCOf<T>>::get(&target).ok_or(Error::<T>::NoKYC)?;
			let black_info = match <BlackListOf<T>>::get(&target) {
				Some(mut b) => {
					b.info.push(black);
					b
				}
				None => BlackInfo { info: vec![black] },
			};

			let _ = <ApplicationFormList<T>>::take(&target);

			let basic_deposit = T::BasicDeposit::get();

			T::Currency::unreserve(&target, basic_deposit);
			T::Currency::withdraw(
				&target,
				basic_deposit,
				WithdrawReasons::FEE,
				ExistenceRequirement::KeepAlive,
			);

			<BlackListOf<T>>::insert(&target, black_info);

			Self::deposit_event(Event::<T>::KYCRemove(target));

			Ok(Some(T::WeightInfo::remove_kyc(
				T::MaxSwordHolder::get().into(), // R
			))
			.into())
		}

		/// User setting KYC
		///
		/// If the user is added to the blacklist, cannot be set
		///
		/// - `info`: KYCInfo.
		///
		/// Emits `KYCSet` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::set_kyc(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn set_kyc(origin: OriginFor<T>, info: KYCInfo) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(!<BlackListOf<T>>::contains_key(&sender), Error::<T>::Blacklisted);
			ensure!(!<KYCOf<T>>::contains_key(&sender), Error::<T>::KYCFound);
			let mut reg = match <KYCOf<T>>::get(&sender) {
				Some(mut reg) => {
					// Only keep paid judgements.
					reg.judgements.retain(|j| j.2.is_paid());
					reg.info = info;
					reg
				}
				None => Registration {
					info,
					judgements: Vec::new(),
					deposit: Zero::zero(),
				},
			};

			let old_deposit = reg.deposit;
			reg.deposit = T::BasicDeposit::get();
			if reg.deposit > old_deposit {
				T::Currency::reserve(&sender, reg.deposit - old_deposit)?;
			}
			if old_deposit > reg.deposit {
				let _ = T::Currency::unreserve(&sender, old_deposit - reg.deposit);
			}

			<KYCOf<T>>::insert(&sender, reg);
			Self::deposit_event(Event::<T>::KYCSet(sender));

			Ok(Some(T::WeightInfo::set_kyc(
				T::MaxIAS::get().into(),         // R
				T::MaxSwordHolder::get().into(), // X
			))
			.into())
		}

		/// Users clean up their own KYC
		///
		/// The `ApplicationFormList` will also be cleaned up while cleaning up
		///
		/// Emits `KYCCleared` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::clear_kyc(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn clear_kyc(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let reg = <KYCOf<T>>::take(&sender).ok_or(Error::<T>::NotFound)?;
			let deposit = reg.total_deposit();

			let _ = T::Currency::unreserve(&sender, deposit.clone());

			let _ = <ApplicationFormList<T>>::take(&sender);

			Self::deposit_event(Event::<T>::KYCCleared(sender, deposit));

			Ok(Some(T::WeightInfo::clear_kyc(
				T::MaxIAS::get().into(),         // R
				T::MaxSwordHolder::get().into(), // X
			))
			.into())
		}

		/// The user applies for verification by ias
		///
		/// The user must have submitted KYC and have not been added to the blacklist
		///
		/// - `kyc_fields`: KYCFields.
		/// - `max_fee`: BalanceOf<T>.
		///
		/// Emits `ApplyCertification` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::apply_certification(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn apply_certification(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			max_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(!<BlackListOf<T>>::contains_key(&sender), Error::<T>::Blacklisted);

			let _registration = <KYCOf<T>>::get(&sender).ok_or(Error::<T>::NoKYC)?;

			let mut app_list = <ApplicationFormList<T>>::get(&sender);

			ensure!(
				app_list
					.iter()
					.all(|item| matches!(item, Some(item) if !item.is_repeat(&kyc_fields))),
				Error::<T>::RepeatApplication
			);

			let search_fee = max_fee / 2_u32.saturated_into::<BalanceOf<T>>();

			let app_form = ApplicationForm {
				ias: Self::random_admin(&kyc_fields, &search_fee, true)?,
				supervisor: Self::random_admin(&kyc_fields, &search_fee, false)?,
				progress: Progress::Pending,
			};

			if app_list.is_empty() {
				app_list = vec![Some(app_form)];
			} else {
				app_list.push(Some(app_form));
			}

			<ApplicationFormList<T>>::insert(&sender, app_list);

			Self::deposit_event(Event::<T>::ApplyCertification(sender));
			Ok(().into())
		}

		/// IAS reviews the KYC submitted by the user, and then draws a conclusion on the
		/// information submitted by the user
		///
		/// some tips.
		///
		/// - `kyc_fields`: KYCFields.
		/// - `ias_index`: KYCIndex.
		/// - `message`: Message.
		///
		/// Emits `JudgementRequested` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::request_judgement(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn request_judgement(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			ias_index: KYCIndex,
			message: Message,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::do_request_judgement(&sender, kyc_fields, ias_index, message)?;
			Self::deposit_event(Event::<T>::JudgementRequested(sender, ias_index));
			Ok(Some(T::WeightInfo::request_judgement(
				T::MaxIAS::get().into(),         // R
				T::MaxSwordHolder::get().into(), // X
			))
			.into())
		}

		/// IAS sets its own service fees
		///
		/// - `kyc_fields`: KYCFields.
		/// - `fee`: Balance.
		///
		/// Emits `SetFee` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::ias_set_fee(
			T::MaxIAS::get().into()
		))]
		pub fn ias_set_fee(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::verify_permissions(&sender, &kyc_fields, true)?;
			Self::do_set_fee(&sender, &kyc_fields, &fee, true)?;
			Self::deposit_event(Event::<T>::SetFee(fee));

			Ok(Some(T::WeightInfo::ias_set_fee(T::MaxIAS::get().into())).into()) // R
		}

		/// IAS provide judgement
		///
		/// Only one passport can be provided, so when the passport is used for the second time,
		/// the authentication fails
		///
		/// - `kyc_fields`: KYCFields.
		/// - `kyc_index`: KYCIndex.
		/// - `target`: AccountId.
		/// - `judgement`: Judgement.
		/// - `id`: Data. User's passport ID
		/// - `message`: Message. Information sent to sword holder
		///
		/// Emits `JudgementGiven` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::ias_provide_judgement(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn ias_provide_judgement(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			kyc_index: KYCIndex,
			target: T::AccountId,
			judgement: Judgement<BalanceOf<T>>,
			id: Data,
			message: Message,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::verify_permissions(&sender, &kyc_fields, true)?;
			let mut unique_id_list = <UniqueIdOf<T>>::get(&kyc_fields);
			ensure!(!unique_id_list.contains(&id), Error::<T>::NotUniqueID);

			Self::ias_do_provide_judgement(&sender, kyc_fields.clone(), kyc_index, &target, judgement, id, message)?;

			Self::deposit_event(Event::<T>::JudgementGiven(target, kyc_index));
			Ok(Some(T::WeightInfo::ias_provide_judgement(
				T::MaxIAS::get().into(),         // R
				T::MaxSwordHolder::get().into(), // X
			))
			.into())
		}

		/// sword holder set fee
		///
		/// - `kyc_fields`: KYCFields.
		/// - `KYCFields`: Balance.
		///
		/// Emits `SetFee` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::sword_holder_set_fee(
			T::MaxSwordHolder::get().into()
		))]
		pub fn sword_holder_set_fee(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::verify_permissions(&sender, &kyc_fields, false)?;
			Self::do_set_fee(&sender, &kyc_fields, &fee, false)?;
			Self::deposit_event(Event::<T>::SetFee(fee));
			Ok(Some(T::WeightInfo::sword_holder_set_fee(T::MaxSwordHolder::get().into())).into()) // R
		}

		/// sword_holder make review  on the information provided by IAS
		///
		/// - `kyc_fields`: KYCFields.
		/// - `kyc_index`: KYCIndex.
		/// - `target`: T::AccountId.
		/// - `auth`: Authentication.
		/// - `id`: Data.
		///
		/// Emits `AuthenticationGiven` if successful.
		///
		/// # <weight>
		/// - TODO
		/// # </weight>
		#[pallet::weight(T::WeightInfo::sword_holder_provide_judgement(
			T::MaxIAS::get().into(), // R
			T::MaxSwordHolder::get().into(), // X
		))]
		pub fn sword_holder_provide_judgement(
			origin: OriginFor<T>,
			kyc_fields: KYCFields,
			kyc_index: KYCIndex,
			target: T::AccountId,
			auth: Authentication,
			id: Data,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::verify_permissions(&sender, &kyc_fields, false)?;
			ensure!(!auth.is_pending(), Error::<T>::PendingAuthentication);
			Self::sword_do_provide_judgement(&sender, kyc_fields, kyc_index, &target, &auth, &id)?;
			Self::deposit_event(Event::<T>::AuthenticationGiven(target, kyc_index));
			Ok(Some(T::WeightInfo::sword_holder_provide_judgement(
				T::MaxIAS::get().into(),         // R
				T::MaxSwordHolder::get().into(), // X
			))
			.into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Safely increment the nonce, with error on overflow
		fn increment_nonce() -> DispatchResult {
			<Nonce<T>>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or(Error::<T>::NonceOverflow)?;
				*nonce = next;
				Ok(().into())
			})
		}

		fn increment_area_count(area: &AreaCode) -> DispatchResult {
			<AreaData<T>>::try_mutate(area, |count| {
				let next = count.checked_add(1).ok_or(Error::<T>::CountOverflow)?;
				*count = next;
				Ok(().into())
			})
		}

		/// generate a random number for get random value
		fn generate_random_number(seed: u64) -> u64 {
			let (random_seed, _block_number) = T::Randomness::random(&(T::PalletId::get(), seed).encode()[..]);
			let random_number =
				<u64>::decode(&mut random_seed.as_ref()).expect("secure hashes should always be bigger than u64; qed");
			random_number
		}

		/// generate random  IAS/SwordHolder,
		/// Every time it is randomized, Nonce storage value  will be added by 1
		fn random_admin(
			kyc_fields: &KYCFields,
			max_fee: &BalanceOf<T>,
			is_ias: bool,
		) -> Result<(u32, IASInfo<BalanceOf<T>, T::AccountId>), DispatchError> {
			let nonce = <Nonce<T>>::get();
			let random_number = Self::generate_random_number(nonce) as usize;

			let mut res_list: Vec<(u32, IASInfo<BalanceOf<T>, T::AccountId>)> = Vec::new();

			if is_ias {
				<IASListOf<T>>::get(kyc_fields)
					.into_iter()
					.enumerate()
					.for_each(|(i, input)| {
						if let Some(x) = input {
							if x.fee <= *max_fee && x.fields == *kyc_fields {
								res_list.push((i as u32, x))
							}
						}
					});
			} else {
				<SwordHolderOf<T>>::get(kyc_fields)
					.into_iter()
					.enumerate()
					.for_each(|(i, input)| {
						if let Some(x) = input {
							if x.fee <= *max_fee && x.fields == *kyc_fields {
								res_list.push((i as u32, x))
							}
						}
					});
			}

			let ias_size = &res_list.len();

			ensure!(ias_size > &(0 as usize), Error::<T>::NoIASOrSwordHolder);

			let random_index = random_number % ias_size;
			Self::increment_nonce()?;
			Ok(res_list[random_index].clone())
		}

		/// the  function for add admin(ias/sword holder)
		pub(crate) fn add_kyc_service(
			kyc_index: KYCIndex,
			ias_info: IASInfo<BalanceOf<T>, T::AccountId>,
			is_ias: bool,
		) -> sp_std::result::Result<(), DispatchError> {
			let ias_info_cp = ias_info.clone();
			let kyc_fields = &ias_info_cp.fields;

			let mut ias_list: Vec<Option<IASInfo<BalanceOf<T>, T::AccountId>>> = Vec::new();
			if is_ias {
				ias_list = <IASListOf<T>>::get(&kyc_fields);
			} else {
				ias_list = <SwordHolderOf<T>>::get(&kyc_fields);
			}

			ensure!(
				ias_list
					.iter()
					.all(|ias| matches!(ias, Some(i) if &i.account != &ias_info_cp.account)),
				Error::<T>::AccountExists
			);

			let ias_list_length = ias_list.len() as u32;
			ensure!(ias_list_length + 1 > kyc_index, Error::<T>::OutofBounds);
			ensure!(ias_list_length < T::MaxIAS::get(), Error::<T>::TooManyAccount);

			if ias_list_length == kyc_index {
				ias_list.push(Some(ias_info))
			} else {
				if let Some(ias) = ias_list.get_mut(kyc_index as usize) {
					ias.as_mut().map(|s| {
						*s = ias_info;
						s
					});
				}
			}

			let service_deposit = T::ServiceDeposit::get();
			T::Currency::reserve(&ias_info_cp.account, service_deposit)?;

			if is_ias {
				<IASListOf<T>>::insert(&kyc_fields, ias_list);
			} else {
				<SwordHolderOf<T>>::insert(&kyc_fields, ias_list);
			}
			Ok(())
		}

		/// When the user has completed the KYC application, the next step: ias review
		///
		/// The premise is that it must have submitted KYC information，
		/// At the same time, user must encrypt some data for verification into Message and store it
		/// on the chain. ias gets the Message, decrypts it, and then conducts verification and
		/// review
		pub(crate) fn do_request_judgement(
			who: &T::AccountId,
			kyc_fields: KYCFields,
			kyc_index: KYCIndex,
			message: Message,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut app_list: Vec<Option<ApplicationForm<BalanceOf<T>, T::AccountId>>> =
				<ApplicationFormList<T>>::get(who);

			if let Some(ApplicationForm { ias, supervisor, progress }) = app_list
				.iter()
				.filter(|item| matches!(item, Some(item) if item.ias.1.fields == kyc_fields && item.ias.0 == kyc_index && item.progress == Progress::Pending))
				.next()
				.ok_or(Error::<T>::NoApplication)? {
				let mut registration = <KYCOf<T>>::get(&who).ok_or(Error::<T>::NoKYC)?;

				let pay_fee = ias.1.fee
					.checked_add(&supervisor.1.fee)
					.ok_or_else(|| DispatchError::from(Error::<T>::InvalidFee))?;

				let item = (
					kyc_fields.clone(),
					kyc_index,
					Judgement::FeePaid(pay_fee.clone()),
					Authentication::Pending,
				);

				registration.judgements.push(item);

				T::Currency::reserve(&who, pay_fee)?;

				<KYCOf<T>>::insert(&who, registration);

				let record = Record {
					account: ias.1.account.clone(),
					progress: Progress::Pending,
					fields: kyc_fields,
				};
				Self::update_application_form(&who, ias.1.account.clone(), ias.0,
											  supervisor.1.account.clone(), supervisor.0,
											  ias.1.fields, Progress::IasDoing)?;

				// add record to ias records
				Self::add_record_list(&who, record)?;
				Self::add_message_list(&who, &ias.1.account, message)?;
			}
			Ok(())
		}

		/// After the ias review, and passed, give to the sword holder review
		pub(crate) fn ias_do_provide_judgement(
			who: &T::AccountId,
			kyc_fields: KYCFields,
			kyc_index: KYCIndex,
			target: &T::AccountId,
			judgement: Judgement<BalanceOf<T>>,
			id: Data,
			message: Message,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut app_list: Vec<Option<ApplicationForm<BalanceOf<T>, T::AccountId>>> =
				<ApplicationFormList<T>>::get(target);
			let mut app_list_cp = app_list.clone();

			if let Some(ApplicationForm { ias, supervisor, progress }) = app_list_cp
				.iter()
				.filter(|item| matches!(item, Some(item) if item.ias.1.fields == kyc_fields && item.ias.1.account == who.clone() && item.ias.0 == kyc_index && item.progress == Progress::IasDoing))
				.next()
				.ok_or(Error::<T>::NoApplication)? {

				let mut registration = <KYCOf<T>>::get(target).ok_or(Error::<T>::NoKYC)?;


				for element in registration.judgements.iter_mut() {
					let (field, index, _judgement, _auth) = &element;
					if field.clone() == kyc_fields && index.clone() == kyc_index {
						*element = (
							kyc_fields.clone(),
							kyc_index.clone(),
							judgement.clone(),
							Authentication::Pending,
						)
					}
				}

				let mut record_list: Vec<Record<T::AccountId>> = <RecordsOf<T>>::get(who);

				for record in record_list.iter_mut() {
					if record.account == who.clone() && record.fields == kyc_fields.clone() {
						record.progress = Progress::IasDone;
						Self::add_record_list(&supervisor.1.account, record.clone());
					}
				}
				<RecordsOf<T>>::insert(who, record_list);

				Self::add_unique_id_list(&kyc_fields, id);
				Self::update_application_form(target, ias.1.account.clone(), ias.0,
											  supervisor.1.account.clone(), supervisor.0,
											  ias.1.fields, Progress::IasDone)?;
				Self::add_message_list(who, &supervisor.1.account, message)?;
			}

			Ok(())
		}

		/// sword provide judgement logic function
		pub(crate) fn sword_do_provide_judgement(
			who: &T::AccountId,
			kyc_fields: KYCFields,
			kyc_index: KYCIndex,
			target: &T::AccountId,
			authentication: &Authentication,
			id: &Data,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut unique_id_list = <UniqueIdOf<T>>::get(&kyc_fields);
			ensure!(unique_id_list.contains(&id), Error::<T>::NotContainsUniqueID);

			let mut app_list: Vec<Option<ApplicationForm<BalanceOf<T>, T::AccountId>>> =
				<ApplicationFormList<T>>::get(target);
			let mut app_list_cp = app_list.clone();

			if let Some(ApplicationForm { ias, supervisor, progress }) = app_list_cp
				.iter()
				.filter(|item| matches!(item, Some(item) if item.supervisor.1.fields == kyc_fields && item.supervisor.1.account == who.clone() && item.supervisor.0 == kyc_index && item.progress == Progress::IasDone))
				.next()
				.ok_or(Error::<T>::NoApplication)? {
				if authentication.has_failure() {
					let pay_fee = ias.1.fee
						.checked_add(&supervisor.1.fee)
						.ok_or_else(|| DispatchError::from(Error::<T>::InvalidFee))?;
					Self::update_record_list(&ias.1.account, target, &kyc_fields, Progress::Failure);
					Self::update_record_list(&supervisor.1.account, target, &kyc_fields, Progress::Failure);
					Self::update_application_form(target, ias.1.account.clone(), ias.0,
												  supervisor.1.account.clone(), supervisor.0,
												  ias.1.fields, Progress::Failure)?;
					Self::update_kyc_auth(target,&kyc_fields,ias.0,authentication);
					T::Currency::unreserve(target, pay_fee);

				} else {
					for application in app_list.iter_mut() {
						if matches!(application, Some(app)  if app.supervisor.1.fields == kyc_fields && app.supervisor.0 == kyc_index && app.supervisor.1.account == who.clone() && app.progress == Progress::IasDone)
						{
							let mut registration = <KYCOf<T>>::get(target).ok_or(Error::<T>::InvalidTarget)?;
							for element in registration.judgements.iter_mut() {
								let (field, index, judgement, auth) = &element;
								if field == &kyc_fields && index == &kyc_index && auth.is_pending() {
									if let Judgement::FeePaid(fee) = judgement {
										let _ = T::Currency::repatriate_reserved(
											target,
											&ias.1.account,
											ias.1.fee,
											BalanceStatus::Free,
										);
										let _ = T::Currency::repatriate_reserved(
											&target,
											&supervisor.1.account,
											supervisor.1.fee,
											BalanceStatus::Free,
										);
									}
									element.3 = *authentication;
								}
							}

							if kyc_fields == KYCFields::Area {
								Self::increment_area_count(&registration.info.area)?;
							}
							<KYCOf<T>>::insert(&target, registration);

							application.as_mut().map(|i| {
								i.set_progress(Progress::Success);
								i
							});
						}
					}


					<ApplicationFormList<T>>::insert(target, app_list);
					Self::update_record_list(&ias.1.account, target, &kyc_fields, Progress::Success);
					Self::update_record_list(&supervisor.1.account, target, &kyc_fields, Progress::Success);
				}
			}
			Ok(())
		}

		/// add record
		pub(crate) fn add_record_list(
			who: &T::AccountId,
			record: Record<T::AccountId>,
		) -> sp_std::result::Result<(), DispatchError> {
			// add record to record_list
			let record_list = match <RecordsOf<T>>::try_get(who) {
				Ok(mut record_list) => {
					record_list.push(record);
					record_list
				}
				Err(_) => vec![record],
			};
			<RecordsOf<T>>::insert(who, record_list);
			Ok(())
		}

		/// update record
		pub(crate) fn update_record_list(
			who: &T::AccountId,
			target: &T::AccountId,
			kyc_fields: &KYCFields,
			progress: Progress,
		) -> sp_std::result::Result<(), DispatchError> {
			// update record to record_list
			let mut record_list: Vec<Record<T::AccountId>> = <RecordsOf<T>>::get(who);
			for record in record_list.iter_mut() {
				if record.account == target.clone() && record.fields == kyc_fields.clone() {
					record.progress = progress;
				}
			}
			<RecordsOf<T>>::insert(who, record_list);
			Ok(())
		}

		/// update application list
		pub(crate) fn update_application_form(
			who: &T::AccountId,
			ias_account: T::AccountId,
			ias_index: KYCIndex,
			supervisor_account: T::AccountId,
			supervisor_index: KYCIndex,
			kyc_fields: KYCFields,
			progress: Progress,
		) -> sp_std::result::Result<(), DispatchError> {
			// update record to record_list
			let mut app_list: Vec<Option<ApplicationForm<BalanceOf<T>, T::AccountId>>> =
				<ApplicationFormList<T>>::get(who);
			for app in app_list.iter_mut() {
				if matches!(app, Some(a) if a.ias.0 == ias_index && a.ias.1.account == ias_account
				&& a.ias.1.fields == kyc_fields && a.supervisor.0 == supervisor_index
				&& a.supervisor.1.account == supervisor_account && a.progress != Progress::Failure)
				{
					app.as_mut().map(|i| {
						i.set_progress(progress);
						i
					});
				}
			}
			<ApplicationFormList<T>>::insert(who, app_list);
			Ok(())
		}

		/// add messages
		pub(crate) fn add_message_list(
			who: &T::AccountId,
			target: &T::AccountId,
			message: Message,
		) -> sp_std::result::Result<(), DispatchError> {
			// add message to message_list
			let message_list = match <MessageList<T>>::try_get(&who, target) {
				Ok(mut message_list) => {
					message_list.push(message);
					message_list
				}
				Err(_) => vec![message],
			};
			<MessageList<T>>::insert(who, target, message_list);
			Ok(())
		}

		/// add unique id
		pub(crate) fn add_unique_id_list(
			kyc_fields: &KYCFields,
			id: Data,
		) -> sp_std::result::Result<(), DispatchError> {
			// add add_unique_id_list
			let unique_list = match <UniqueIdOf<T>>::try_get(kyc_fields) {
				Ok(mut unique_list) => {
					unique_list.push(id);
					unique_list
				}
				Err(_) => vec![id],
			};
			<UniqueIdOf<T>>::insert(kyc_fields, unique_list);
			Ok(())
		}

		/// set fees
		pub(crate) fn do_set_fee(
			who: &T::AccountId,
			kyc_fields: &KYCFields,
			fee: &BalanceOf<T>,
			is_ias: bool,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut ias_list: Vec<Option<IASInfo<BalanceOf<T>, T::AccountId>>> = Vec::new();
			if is_ias {
				ias_list = <IASListOf<T>>::get(&kyc_fields);
			} else {
				ias_list = <SwordHolderOf<T>>::get(&kyc_fields);
			}

			for ias in ias_list.iter_mut() {
				if matches!(ias, Some(i) if i.account == who.clone()) {
					ias.as_mut().map(|i| {
						i.set_fee(fee.clone());
						i
					});
				}
			}
			if is_ias {
				<IASListOf<T>>::insert(kyc_fields, ias_list);
			} else {
				<SwordHolderOf<T>>::insert(kyc_fields, ias_list);
			}
			Ok(())
		}

		/// verify permissions
		pub(crate) fn verify_permissions(
			who: &T::AccountId,
			kyc_fields: &KYCFields,
			is_ias: bool,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut ias_list: Vec<Option<IASInfo<BalanceOf<T>, T::AccountId>>> = Vec::new();
			if is_ias {
				ias_list = <IASListOf<T>>::get(kyc_fields);
			} else {
				ias_list = <SwordHolderOf<T>>::get(kyc_fields);
			}

			ensure!(
				ias_list
					.iter()
					.any(|ias| matches!(ias, Some(i) if i.account == who.clone())),
				Error::<T>::InsufficientPermissions
			);
			Ok(())
		}

		/// update kyc authentication
		pub(crate) fn update_kyc_auth(
			who: &T::AccountId,
			kyc_fields: &KYCFields,
			kyc_index: KYCIndex,
			auth: &Authentication,
		) -> sp_std::result::Result<(), DispatchError> {
			let mut registration = <KYCOf<T>>::get(who).ok_or(Error::<T>::InvalidTarget)?;
			for element in registration.judgements.iter_mut() {
				let (field, index, judgement, _) = &element;
				if field == kyc_fields && index == &kyc_index {
					element.3 = *auth;
				}
			}
			<KYCOf<T>>::insert(who, registration);
			Ok(())
		}
	}
}

impl<T: Config> KycHandler<T::AccountId, AreaCode> for Pallet<T> {
	fn get_user_area(user: &T::AccountId) -> Option<AreaCode> {
		match <KYCOf<T>>::get(user) {
			Some(info) => {
				return Some(info.info.area);
			},
			None => None,
		}
	}
}
