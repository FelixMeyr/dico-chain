//! # Non Fungible Token
//! The module provides implementations for non-fungible-token.
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! This module provides basic functions to create and manager
//! NFT(non fungible token) such as `create_class`, `transfer`, `mint`, `burn`.

//! ### Module Functions
//!
//! - `create_class` - Create NFT(non fungible token) class
//! - `transfer` - Transfer NFT(non fungible token) to another account.
//! - `mint` - Mint NFT(non fungible token)
//! - `burn` - Burn NFT(non fungible token)
//! - `destroy_class` - Destroy NFT(non fungible token) class

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, MaxEncodedLen, Currency, ExistenceRequirement, WithdrawReasons},
	BoundedVec, Parameter,
};
use pallet_ico::traits::PowerHandler;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero,
	Hash},
	ArithmeticError, DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{convert::TryInto, vec::Vec, collections::btree_map::BTreeMap};
use sp_std::vec;
use sp_runtime::offchain::storage_lock::BlockNumberProvider;
mod mock;
mod tests;

pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug)]
pub struct ClassInfo<TokenId, AccountId, Data, ClassMetadataOf> {
	/// Class metadata
	pub metadata: ClassMetadataOf,
	/// Total issuance for the class
	pub total_issuance: TokenId,
	/// Class owner
	pub issuer: AccountId,
	/// Class Properties
	pub data: Data,
}


/// class data
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug)]
pub struct ClassData<NftLevel, Hash, Balance, TokenId> {
	level: NftLevel,
	power_threshold: Balance,
	claim_payment: Balance,
	images_hash: Option<Hash>,
	maximum_quantity: TokenId,
}


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, Default)]
pub struct TokenData<Hash, AccountId, Attribute, Balance, NftStatus, ClassId> {
	class_id: ClassId,
	hash: Hash,  
	power_threshold: Balance,
	claim_payment: Balance,
	attribute: Attribute,
	image_hash: Hash,
	sell_records: Vec<(AccountId, Balance)>,
	status: NftStatus,
}


/// nft level
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum NftLevel {
	Rookie,
	Angle,
	WallStreetElite,
	UnicornHunter,
	Mafia,
	GrandMaster,
	Other(Vec<u8>),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct  SaleInfo<TokenId, Balance, BlockNumber, AccountId> {
	seller: AccountId,
	token_id: TokenId,
	price: Balance,
	start_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct  NftStatus {
	is_in_sale: bool,
	is_active_image: bool,
	is_claimed: bool,
}

impl Default for NftStatus {
	fn default() -> Self {
		Self {
			is_in_sale: false,
			is_active_image: false,
			is_claimed: false,
		}
	}
}


impl Default for NftLevel {
	fn default() -> Self {
		Self::Rookie
	}
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug)]
pub struct TokenInfo<AccountId, Data, TokenMetadataOf> {
	/// Token metadata
	pub metadata: TokenMetadataOf,
	/// Token owner
	pub owner: Option<AccountId>,
	/// Token Properties
	pub data: Data,
}

pub use module::*;
use sp_runtime::RuntimeString::Owned;

#[frame_support::pallet]
pub mod module {
	use super::*;
	use pallet_ico::system::pallet_prelude::OriginFor;
	use pallet_ico::ensure_signed;
	use crate::Error::TokenNotFound;

	#[pallet::config]
	pub trait Config: frame_system::Config {

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
		type Currency: Currency<Self::AccountId>;
		/// The maximum size of a class's metadata
		type MaxClassMetadata: Get<u32>;
		/// The maximum size of a token's metadata
		type MaxTokenMetadata: Get<u32>;
		/// The maximum size of a token's attribute.
		type MaxTokenAttribute: Get<u32>;
		type PowerHandler: pallet_ico::traits::PowerHandler<Self::AccountId, DispatchResult, BalanceOf<Self>>;

	}

	pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;
	pub type SaleInfoOf<T> = SaleInfo<<T as Config>::TokenId, <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance, <T as frame_system::Config>::BlockNumber, <T as frame_system::Config>::AccountId>;
	pub type TokenMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxTokenMetadata>;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type TokenDataOf<T> = TokenData<<T as frame_system::Config>::Hash, <T as frame_system::Config>::AccountId, Attributes,  BalanceOf<T>, NftStatus, <T as Config>::ClassId>;
	pub type ClassDataOf<T> = ClassData<NftLevel, <T as frame_system::Config>::Hash, BalanceOf<T>, <T as Config>::TokenId>;
	pub type ClassInfoOf<T> = ClassInfo<
		<T as Config>::TokenId,
		<T as frame_system::Config>::AccountId,
		ClassDataOf<T>,
		ClassMetadataOf<T>,
	>;
	pub type TokenInfoOf<T> =
		TokenInfo<<T as frame_system::Config>::AccountId, TokenDataOf<T>, TokenMetadataOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateClass(T::AccountId, T::ClassId),
		Transfer(T::AccountId, T::AccountId, (T::ClassId, T::TokenId)),
		Mint(T::ClassId, T::TokenId),
		Claim(T::AccountId, T::ClassId, T::TokenId),
		Burn(T::AccountId, T::ClassId, T::TokenId),
		OfferTokenForSale(T::ClassId, T::TokenId, BalanceOf<T>),
		WithdrawSale(T::ClassId, T::TokenId),
		BuyToken(T::AccountId, T::ClassId, T::TokenId, BalanceOf<T>),
		DestroyClass(T::AccountId, T::ClassId),
	}

	/// Error for non-fungible-token module.
	#[pallet::error]
	pub enum Error<T> {
		/// No available class ID
		NoAvailableClassId,
		/// No available token ID
		NoAvailableTokenId,
		/// Token(ClassId, TokenId) not found
		TokenNotFound,
		/// Class not found
		ClassNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
		/// Failed because the Maximum amount of metadata was exceeded
		MaxMetadataExceeded,
		///
		LevelInUse,
		///
		InSale,
		NotIssuer,
		OwnerIsExists,
		NotOwner,
		NotInSale,
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextTokenId<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, T::TokenId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn in_sale_tokens)]
	pub type InSaleTokens<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, Vec<SaleInfoOf<T>>, ValueQuery>;

	/// Store class info.
	///
	/// Returns `None` if class info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn classes)]
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

	/// Store token info.
	///
	/// Returns `None` if token info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TokenInfoOf<T>>;


	#[pallet::storage]
	#[pallet::getter(fn issuer_Of)]
	pub type IssuerOf<T: Config> = StorageMap<_, Twox64Concat, NftLevel, (T::AccountId, T::ClassId)>;


	/// the user's all tokens
	#[pallet::storage]
	#[pallet::getter(fn tokens_Of)]
	pub type TokensOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<(T::ClassId, T::TokenId)>, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn no_owner_tokens_of)]
	pub type NoOwnerTokensOf<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, Vec<T::TokenId>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_class(
			origin: OriginFor<T>,
			metadata: Vec<u8>,
			data: ClassDataOf<T>
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			let class_id = Self::do_create_class(&issuer, metadata, data)?;
			Self::deposit_event(Event::<T>::CreateClass(issuer, class_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			token: (T::ClassId, T::TokenId)
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer(&who, &to, token)?;
			Self::deposit_event(Event::<T>::Transfer(who, to, token));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			metadata: Vec<u8>,
			attribute: Attributes,
			image_hash: T::Hash
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			let token_id = Self::do_mint(&issuer, class_id, metadata, attribute, image_hash)?;
			Self::deposit_event(Event::Mint(class_id,token_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim(
			origin: OriginFor<T>,
			token: (T::ClassId, T::TokenId),
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::do_claim(&owner, token.0, token.1)?;
			Self::deposit_event(Event::Claim(owner, token.0, token.1));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			token: (T::ClassId, T::TokenId)
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::do_burn(&owner, token)?;
			Self::deposit_event(Event::<T>::Burn(owner, token.0, token.1));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn offer_token_for_sale(
			origin: OriginFor<T>,
			token: (T::ClassId, T::TokenId),
			price: BalanceOf<T>
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::do_offer_token_for_sale(&owner, token, price)?;
			Self::deposit_event(Event::OfferTokenForSale(token.0, token.1, price));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn withdraw_sale(
			origin: OriginFor<T>,
			token: (T::ClassId, T::TokenId)
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::do_withdraw_sale(&owner, token)?;
			Self::deposit_event(Event::<T>::WithdrawSale(token.0, token.1));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn buy_token(
			origin: OriginFor<T>,
			token: (T::ClassId, T::TokenId)
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			Self::do_buy_token(&buyer, token)?;
			Ok(())
		}



	}
}

impl<T: Config> Pallet<T> {

	/// Create NFT(non fungible token) class
	pub fn do_create_class(
		issuer: &T::AccountId,
		metadata: Vec<u8>,
		data: ClassDataOf<T>,
	) -> Result<T::ClassId, DispatchError> {

		ensure!(IssuerOf::<T>::get(&data.level).is_none(), Error::<T>::LevelInUse);
		let bounded_metadata: BoundedVec<u8, T::MaxClassMetadata> =
			metadata.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;

		let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
			Ok(current_id)
		})?;

		let info = ClassInfo {
			metadata: bounded_metadata,
			total_issuance: Default::default(),
			issuer: issuer.clone(),
			data: data.clone(),
		};

		Classes::<T>::insert(class_id, info);
		IssuerOf::<T>::insert(data.level, (issuer, class_id));
		Ok(class_id)
	}

	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn do_transfer(from: &T::AccountId, to: &T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
		Tokens::<T>::try_mutate(token.0, token.1, |token_info| -> DispatchResult {
			let mut info = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;

			ensure!(!info.data.status.is_in_sale && info.data.status.is_claimed, Error::<T>::InSale);
			ensure!(info.owner == Some(from.clone()), Error::<T>::NoPermission);

			if from == to {
				return Ok(());
			}
			info.owner = Some(to.clone());
			Self::transfer_ownership(&from, &to, token.0, token.1);

			Ok(())
		})
	}

	/// Mint NFT(non fungible token) to `owner`
	pub fn do_mint(
		issuer: &T::AccountId,
		class_id: T::ClassId,
		metadata: Vec<u8>,
		attribute: Attributes,
		image_hash: T::Hash,
	) -> Result<T::TokenId, DispatchError> {
		NextTokenId::<T>::try_mutate(class_id, |id| -> Result<T::TokenId, DispatchError> {

			let bounded_metadata: BoundedVec<u8, T::MaxTokenMetadata> =
				metadata.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;

			let token_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;

			ensure!(Classes::<T>::get(class_id).ok_or(Error::<T>::ClassNotFound)?.issuer == issuer.clone(), Error::<T>::NotIssuer);

			let mut data = TokenDataOf::<T>::default();
			let hash = Self::get_hash(class_id, token_id);
			data.hash = hash;
			data.power_threshold = BalanceOf::<T>::from(0u32);
			data.claim_payment = BalanceOf::<T>::from(0u32);
			data.attribute = attribute;
			data.image_hash = image_hash;
			data.sell_records = vec![];
			data.status = NftStatus::default();

			Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
				let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				let new_total_issuance = info
					.total_issuance
					.checked_add(&One::one())
					.ok_or(ArithmeticError::Overflow)?;

				ensure!(info.data.maximum_quantity >= new_total_issuance, Error::<T>::NoAvailableTokenId);
				info.total_issuance = new_total_issuance;
				Ok(())
			})?;

			let token_info = TokenInfo {
				metadata: bounded_metadata,
				owner: None,
				data,
			};
			Tokens::<T>::insert(class_id, token_id, token_info);
			Self::update_no_owner_tokens_vec(class_id, token_id, false);

			Ok(token_id)
		})
	}


	fn do_claim(owner: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(class_id, token_id, |token_info| -> DispatchResult {
			let mut t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owner == None, Error::<T>::OwnerIsExists);
			let class_info = Classes::<T>::get(class_id).ok_or(Error::<T>::ClassNotFound)?;
			T::Currency::withdraw(&owner, class_info.data.claim_payment, WithdrawReasons::TRANSFER, ExistenceRequirement::KeepAlive)?;
			T::PowerHandler::sub_user_power(&owner, class_info.data.power_threshold)?;
			t.owner = Some(owner.clone());
			t.data.power_threshold = class_info.data.power_threshold;
			t.data.claim_payment = class_info.data.claim_payment;
			t.data.status = NftStatus {
				is_claimed: true,
				is_in_sale: false,
				is_active_image: true,
			};
			Self::update_no_owner_tokens_vec(class_id, token_id, true);
			*token_info = Some(t);
			Ok(())
		})
	}

	/// Burn NFT(non fungible token) from `owner`
	pub fn do_burn(owner: &T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
			let mut t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owner == Some(owner.clone()), Error::<T>::NoPermission);

			Classes::<T>::try_mutate(token.0, |class_info| -> DispatchResult {
				let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				info.total_issuance = info
					.total_issuance
					.checked_sub(&One::one())
					.ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			T::PowerHandler::add_user_power(&owner, t.data.power_threshold)?;
			Self::remove_token_ownership(owner, token.0, token.1);

			t.owner = None;
			t.data.power_threshold = BalanceOf::<T>::from(0u32);
			t.data.claim_payment = BalanceOf::<T>::from(0u32);
			t.data.sell_records = vec![];
			t.data.status = NftStatus::default();
			*token_info = Some(t);
			Self::update_no_owner_tokens_vec(token.0, token.1, false);

			Ok(())
		})
	}


	fn do_offer_token_for_sale(user: &T::AccountId, token:(T::ClassId, T::TokenId), price: BalanceOf<T>) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
			let mut t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owner == Some(user.clone()), Error::<T>::NotOwner);
			ensure!(!Self::is_in_sale(token.0, token.1), Error::<T>::InSale);
			t.data.status.is_in_sale = true;
			Self::insert_token_to_sale_vec(&user,token.0, token.1, price);
			*token_info = Some(t);
			Ok(())
		})
	}

	fn do_withdraw_sale(user: &T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
			let mut t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owner == Some(user.clone()), Error::<T>::NotOwner);
			ensure!(Self::is_in_sale(token.0, token.1), Error::<T>::NotInSale);
			t.data.status.is_in_sale = false;
			Self::remove_token_from_sale_vec(token.0, token.1);
			*token_info = Some(t);
			Ok(())
		})
	}

	fn do_buy_token(buyer: &T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
			let mut t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			// ensure!(Self::is_in_sale(token.0, token.1), Error::<T>::NotInSale);
			let sale_info = Self::get_in_sale_token(token.0, token.1).ok_or(Error::<T>::NotInSale)?;
			T::Currency::transfer(&buyer, &sale_info.seller, sale_info.price, ExistenceRequirement::KeepAlive)?;
			Self::remove_token_from_sale_vec(token.0, token.1);
			t.data.status.is_in_sale = false;
			t.owner = Some(buyer.clone());
			*token_info = Some(t);
			Self::deposit_event(Event::BuyToken(buyer.clone(),token.0, token.1, sale_info.price));
			Ok(())
		})
	}


	/// Destroy NFT(non fungible token) class
	pub fn destroy_class(issuer: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
		Classes::<T>::try_mutate_exists(class_id, |class_info| -> DispatchResult {
			let info = class_info.take().ok_or(Error::<T>::ClassNotFound)?;

			ensure!(info.issuer == *issuer, Error::<T>::NoPermission);
			ensure!(info.total_issuance == Zero::zero(), Error::<T>::CannotDestroyClass);

			IssuerOf::<T>::remove(&info.data.level);
			Classes::<T>::remove(class_id);

			Ok(())
		})
	}


	/// Update initializing tokens
	fn update_no_owner_tokens_vec(class_id: T::ClassId, token_id: T::TokenId, is_remove: bool) {
		let mut tokens = NoOwnerTokensOf::<T>::get(class_id);
		match is_remove {
			true => {
				if let Some(pos) = tokens.iter().position(|h| h == &token_id ) {
					tokens.swap_remove(pos);
				}
			},
			false => {
				if let None = tokens.iter().position(|h| h == &token_id ) {
					tokens.push(token_id);
				}
			},
		}

		NoOwnerTokensOf::<T>::insert(class_id, tokens);
	}

	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		if let Some(info) = Tokens::<T>::get(token.0, token.1) {
			if info.owner == Some(account.clone()) {
				return true
			}
		}
		false
	}

	pub fn is_issuer(account: &T::AccountId, class_id: T::ClassId) -> bool {
		if let Some(info) = Classes::<T>::get(class_id) {
			if &info.issuer == account {
				return true;
			}
		}
		false
	}


	fn transfer_ownership(who: &T::AccountId, des: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) {
		Self::remove_token_ownership(&who, class_id, token_id);
		Self::get_token_ownership(&des, class_id, token_id);
	}


	fn get_token_ownership(who: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) {
		let mut tokens = TokensOf::<T>::get(who);
		if let None = tokens.iter().position(|h| h.0 == class_id && h.1 == token_id) {
			tokens.push((class_id, token_id));
			TokensOf::<T>::insert(who, tokens);
		}
	}

	fn remove_token_ownership(who: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) {
		let mut tokens = TokensOf::<T>::get(who);
		if let Some(pos) = tokens.iter().position(|h| h.0 == class_id && h.1 == token_id) {
			tokens.swap_remove(pos);
			TokensOf::<T>::insert(who, tokens);
		}
	}

	fn get_hash(class_id: T::ClassId, token_id: T::TokenId) -> T::Hash {
		T::Hashing::hash_of(&(class_id, token_id))
	}

	fn is_in_sale(class_id: T::ClassId, token_id: T::TokenId) -> bool {
		if let Some(pos) = InSaleTokens::<T>::get(class_id).iter().position(|h| h.token_id == token_id) {
			return true;
		}
		false
	}

	fn block_num() -> T::BlockNumber {
		frame_system::Pallet::<T>::current_block_number()
	}

	fn insert_token_to_sale_vec(user: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId, price: BalanceOf<T>) {
		let mut tokens = InSaleTokens::<T>::get(class_id);
		if let Some(pos) = tokens.iter().position(|h| h.token_id == token_id) {
			tokens.swap_remove(pos);
		}
		tokens.push(SaleInfo {
			seller: user.clone(),
			start_block: Self::block_num(),
			price: price,
			token_id: token_id,
		});
		InSaleTokens::<T>::insert(class_id, tokens);
	}

	fn remove_token_from_sale_vec(class_id: T::ClassId, token_id: T::TokenId) {
		let mut tokens = InSaleTokens::<T>::get(class_id);
		if let Some(pos) = tokens.iter().position(|h| h.token_id == token_id) {
			tokens.swap_remove(pos);
		}
		InSaleTokens::<T>::insert(class_id, tokens);
	}

	fn get_in_sale_token(class_id: T::ClassId, token_id: T::TokenId) -> Option<SaleInfoOf<T>> {
		let mut tokens = InSaleTokens::<T>::get(class_id);
		if let Some(pos) = tokens.iter().position(|h| h.token_id == token_id) {
			return Some(tokens.swap_remove(pos));
		}
		None
	}

}
