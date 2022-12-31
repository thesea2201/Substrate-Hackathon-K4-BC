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
// use crate::Config;
// use frame_support::Blake2_128Concat;
use sp_runtime::traits::Hash;
use sp_runtime::ArithmeticError;
// use sp_std::vec::Vec;
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

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
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


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateReviewer {reviewer: T::Hash, who: T::AccountId}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ReviewerNotFound,
		NotOwnedAccount,
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
			};

			ensure!(!Reviewers::<T>::contains_key(&reviewer.id), Error::<T>::DuplicateReviewer);

			let reviewer_total = Self::count_reviewers();
            let new_reviewer_total = reviewer_total.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			CountReviewers::<T>::put(new_reviewer_total);

			ReviewersAccountOwned::<T>::try_mutate(&who, |list_reviewer| {
				list_reviewer.try_push(id.clone())
			}).map_err(|_| <Error<T>>::TooManyOwned)?;

			// ReviewersAccountOwned::<T>::append(&who, reviewer.id.clone());

			Reviewers::<T>::insert(reviewer.id.clone(), reviewer);

			Self::deposit_event(Event::CreateReviewer { reviewer: id.clone(), who: who.clone() });
			
			Ok(())
		}

		// An example dispatchable that may throw a custom error.
		// #[pallet::call_index(1)]
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;

		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => return Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
	}

	
}

impl<T: Config> Pallet<T> {
	fn id_random() -> T::Hash {
		let block_number = <frame_system::Pallet<T>>::block_number();
		let (seed,_) = T::IdRandom::random_seed();
		T::Hashing::hash_of(&(seed,block_number))

	}
}
