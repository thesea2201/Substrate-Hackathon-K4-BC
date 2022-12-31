#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type Id = u32;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use codec::{Encode, MaxEncodedLen, Decode};
use frame_support::pallet_prelude::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::traits::Hash;
use sp_runtime::ArithmeticError;
use frame_support::traits::Randomness;
use frame_support::dispatch::Vec;

#[frame_support::pallet]
pub mod pallet {
	
	pub use super::*;
	#[derive(Encode, Decode, TypeInfo, Clone, MaxEncodedLen, PartialEq, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	
	pub struct Reviewer<T:Config> {
		pub id: T::Hash,
		pub name: Name,
		pub star: Option<u32>, 
		pub owner: T::AccountId,

	}


	#[derive(Encode,Decode, TypeInfo, Clone,PartialEq, RuntimeDebug)]
	pub struct Name(Vec<u8>);
	impl MaxEncodedLen for Name {
		fn max_encoded_len() -> usize {
			20
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type IdRandom: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxAccount: Get<u32>;
	}

	
	#[pallet::storage]
	#[pallet::getter(fn count_reviewers)]
	pub type CountReviewers<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reviewers)]
	pub type Reviewers<T:Config> = StorageMap<_, Blake2_128Concat, T::Hash, Reviewer<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reviewers_owned)]
	pub type ReviewersAccountOwned<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::MaxAccount> , ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateReviewer {reviewer: T::Hash, who: T::AccountId},
		UpdateReviewer {reviewer: T::Hash}
	}

	#[pallet::error]
	pub enum Error<T> {
		ReviewerNotFound,
		NotOwnedAccountReviewer,
		DuplicateReviewer,
		TooManyOwned,
	}

	#[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}


	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_reviewer(origin: OriginFor<T>, name: Name, star: Option<u32>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let id = Self::id_random();

			let reviewer = Reviewer::<T> {
				id: id.clone(),
				name: name,
				star: star,
				owner: who.clone()
			};

			ensure!(!Reviewers::<T>::contains_key(&reviewer.id), Error::<T>::DuplicateReviewer);

			let reviewer_total = Self::count_reviewers();
            let new_reviewer_total = reviewer_total.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			CountReviewers::<T>::put(new_reviewer_total);

			ReviewersAccountOwned::<T>::try_mutate(&who, |list_reviewer| {
				list_reviewer.try_push(id.clone())
			}).map_err(|_| <Error<T>>::TooManyOwned)?;

			Reviewers::<T>::insert(reviewer.id.clone(), reviewer);

			Self::deposit_event(Event::CreateReviewer { reviewer: id.clone(), who: who.clone() });
			
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn update_reviewer(origin: OriginFor<T>, id: T::Hash, name: Name, star: Option<u32>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let reviewer = Reviewers::<T>::get(id.clone()).ok_or(Error::<T>::ReviewerNotFound)?;

			ensure!(reviewer.owner.clone() == who, Error::<T>::NotOwnedAccountReviewer);
			
			Reviewers::<T>::try_mutate(&id, |change| {
				if let Some(reviewer) = change {
					reviewer.name = name;
					reviewer.star = star;
					return Ok(());
				}
				Err(())
			}).map_err(|_| Error::<T>::ReviewerNotFound)?;
			
			Self::deposit_event(Event::UpdateReviewer { reviewer: id.clone()});
			
			Ok(())
		}
	}

	
}

impl<T: Config> Pallet<T> {
	fn id_random() -> T::Hash {
		let block_number = <frame_system::Pallet<T>>::block_number();
		let (seed,_) = T::IdRandom::random_seed();
		T::Hashing::hash_of(&(seed,block_number))

	}
}
