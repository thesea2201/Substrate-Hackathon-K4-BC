#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use codec::MaxEncodedLen;
	use frame_support::pallet_prelude::*;
	use frame_support::pallet_prelude::BoundedVec;
	use frame_support::{Parameter, Blake2_128Concat};
	use frame_support::traits::Time;
	use frame_support::inherent::Vec;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32Bit, Scale, CheckedAdd, Hash};
	pub use crate::types::*;

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
			+ MaxEncodedLen
			+ CheckedAdd
			+ Default
			+ From<u32>
			+ Into<u32>;
			
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

		#[pallet::constant]
		type NumberLimit: Get<u32>;

		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn apps_count)]
	pub(super) type AppsCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_app_id)]
	pub(super) type NextAppId<T: Config> = StorageValue<_, T::AppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn apps)]
	pub(super) type Apps<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, App<T>, OptionQuery, >;

	#[pallet::storage]
	#[pallet::getter(fn app_owner)]
	pub(super) type AppsOnwer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::NumberLimit>, ValueQuery, >;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AppCreated { app_hash: T::Hash, app_id: T::AppId, who: T::AccountId },
		
	}

	#[pallet::error]
	pub enum Error<T> {
		AppNumberLimited,
		TotalAppLimited,
		AppOwnerLimited,
		AppNotFound,
		NotAppOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_app(origin: OriginFor<T>, name: Vec<u8>, symbol: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let app_id = Self::next_app_id();

			let bounded_name: BoundedVec<u8, T::StringLimit> = name.clone().try_into().expect("app name is too long");
			let bounded_symbol: BoundedVec<u8, T::StringLimit> = symbol.clone().try_into().expect("app symbol is too long");

			let app = App::create(app_id, who.clone(), bounded_name, bounded_symbol);

			let app_hash = T::Hashing::hash_of(&app);

			// storage app
			<Apps<T>>::insert(app_hash, app);

			// storage app owner
			<AppsOnwer<T>>::try_mutate(&who, |app_vec| {
				app_vec.try_push(app_hash)
			}).map_err(|_| <Error<T>>::AppOwnerLimited)?;
			
			// set next app id
			let next_app_id = Self::next_app_id().checked_add(&1_u32.into()).ok_or(<Error<T>>::AppNumberLimited)?;
			<NextAppId<T>>::put(next_app_id);

			// increase app count
			let app_count = Self::apps_count().checked_add(1_u32).ok_or(<Error<T>>::TotalAppLimited)?;
			<AppsCount<T>>::put(app_count);

			Self::deposit_event(Event::AppCreated{app_hash, app_id, who});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn update(origin: OriginFor<T>, app_hash: T::Hash, name: Vec<u8>, symbol: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let app = <Apps<T>>::get(app_hash.clone()).ok_or(<Error<T>>::AppNotFound)?;
			
			// ensure who is app owner
			ensure!(who == app.owner(), <Error<T>>::NotAppOwner)?;

			// ensure name and symbol valid
			let bounded_name: BoundedVec<u8, T::StringLimit> = name.clone().try_into().expect("app name is too long");
			let bounded_symbol: BoundedVec<u8, T::StringLimit> = symbol.clone().try_into().expect("app symbol is too long");

			// change app info
			<Apps<T>>::try_mutate(&app_hash, |app_option| {
				if let Some(app) = app_option {
					app.set_name(bounded_name);
					app.set_symbol(bounded_symbol);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::AppNotFound)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn vote(origin: OriginFor<T>, app_hash: T::Hash, star: T::Star) -> DispatchResult {
			let who = ensure_signed(origin)?;


			Ok(())
		}
	}

	
}
