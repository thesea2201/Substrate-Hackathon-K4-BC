#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use codec::MaxEncodedLen;
	use frame_support::{pallet_prelude::{*, MaybeSerializeDeserialize, ValueQuery, OptionQuery}, traits::Time, Parameter, StorageMap, BoundedVec};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32Bit, Scale};
	pub use types::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type AppId: Member 
			+ Parameter
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;
			
		type Moment: Parameter
			+ Default
			+ AtLeast32Bit
			+ Scale<Self::BlockNumber, Output = Self::Moment>
			+ Copy
			+ MaxEncodedLen
			+ scale_info::StaticTypeInfo
			+ MaybeSerializeDeserialize;

		type AppTime: Time<Moment = Self::Moment>;

		type Star: Parameter
			+ Default
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ Copy;

		type MaxStar: Get<u8>;
	}

	#[pallet::storage]
	#[pallet::getter(fn apps_count)]
	pub type AppsCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn apps)]
	pub type Apps<T> = StorageMap<_, T::Hash, App<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn apps_owner)]
	pub type AppsOwner<T> = StorageMap<_, T::AccountId, BoundedVec<T::Hash, u32>, OptionQuery>;

	

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored { something: u32, who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn xyz(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Ok(())
		}
	}
}
