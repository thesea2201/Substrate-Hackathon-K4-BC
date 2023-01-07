#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_apps;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Time;
	use frame_system::{pallet_prelude::*};
	use frame_support::inherent::Vec;


	use frame_support::sp_runtime::{traits::{AtLeast32Bit, Scale, CheckedAdd, Hash}, SaturatedConversion};


	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Review<T: Config> {
		pub app_id: T::Hash,
		pub star: Option<<T as pallet_apps::Config>::Star>,
		pub title: BoundedVec<u8, T::ContentLimit>,
		pub cons: BoundedVec<u8, T::ContentLimit>,
		pub pros: BoundedVec<u8, T::ContentLimit>,
		pub owner: T::AccountId,
		pub created_date: u64,
	}
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config  + pallet_apps::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type ContentLimit: Get<u32>;

		// type Star: Parameter
		// 	+ Default
		// 	+ MaxEncodedLen
		// 	+ MaybeSerializeDeserialize
		// 	+ Copy
		// 	+ From<u32>
		// 	+ PartialOrd;

		// #[pallet::constant]
		// type StarLimit: Get<u32>;

		#[pallet::constant]
		type ReviewOwnerLimit: Get<u32>;

		#[pallet::constant]
		type ReviewOwnerByAppLimit: Get<u32>;

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
			+ Scale<Self::BlockNumber, Output = <Self as pallet_apps::Config>::Moment>
			+ Copy
			+ MaxEncodedLen
			+ scale_info::StaticTypeInfo
			+ MaybeSerializeDeserialize
			+ Into<u64>;


		type ReviewTime: Time;
	
	}

	#[pallet::storage]
	#[pallet::getter(fn review_owner)]
	pub(super) type ReviewOnwer<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::ReviewOwnerLimit>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn reviews)]
	pub(super) type Reviews<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Review<T>>;



	#[pallet::storage]
	#[pallet::getter(fn reviews_owned_by_app)]
	/// Keeps track of what accounts own what Kitty.
	pub(super) type ReviewsOwnedByApp<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, BoundedVec<T::Hash, T::ReviewOwnerByAppLimit>, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ReviewCreated { who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		StarLimited,

		OwnerCanNotReview,

		AppNotExist,

		ReviewNotExist,

		ReviewOwnerLimit,

		ReviewOwnerByAppLimit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100)]
		pub fn create_review(
			origin: OriginFor<T>,
			star: <T as pallet_apps::Config>::Star,
			app_id: T::Hash,
			title: Vec<u8>,
			cons: Vec<u8>,
			pros: Vec<u8>,
		) -> DispatchResult {
			
			let who = ensure_signed(origin)?;

			let mut app = <pallet_apps::Module<T>>::apps(&app_id).ok_or(<Error<T>>::AppNotExist)?;

			// Limit review of each user
			let review_owned = <ReviewOnwer<T>>::get(&who);
			ensure!((review_owned.len() as u32) < T::ReviewOwnerLimit::get(), <Error<T>>::ReviewOwnerLimit);

			// App owner can not create review
			ensure!(who != app.owner(), <Error<T>>::OwnerCanNotReview);

			let bounded_title: BoundedVec<u8, T::ContentLimit> =
				title.clone().try_into().expect("title is too long");
			let bounded_cons: BoundedVec<u8, T::ContentLimit> =
				cons.clone().try_into().expect("content is too long");
			let bounded_pros: BoundedVec<u8, T::ContentLimit> =
				pros.clone().try_into().expect("content is too long");

			let star_limit = <T as pallet_apps::Config>::StarLimit::get();

			// ensure star less than star limit
			ensure!(star <= star_limit.into(), <Error<T>>::StarLimited);

			let now = T::ReviewTime::now().saturated_into();

			let review: Review<T> = Review::<T> {
				app_id: app_id,
				star: Some(star),
				title: bounded_title,
				cons: bounded_cons,
				pros: bounded_pros,
				owner: who.clone(),
				created_date: now
			};

			let review_id = T::Hashing::hash_of(&review);

			<Reviews<T>>::insert(&review_id, review);

			ReviewsOwnedByApp::<T>::try_mutate(&app_id, |review_vec| {
				review_vec.try_push(review_id)
			}).map_err(|_| <Error<T>>::ReviewOwnerByAppLimit)?;

			Self::deposit_event(Event::ReviewCreated { who });

			Ok(())
		}


		// #[pallet::weight(100)]
		// pub fn get_reviews(
		// 	origin: OriginFor<T>,
		// 	app_id: T::Hash,
		// ) -> Result<Review<T>, Error<T>>  {
		// 	let sender = ensure_signed(origin)?;
		// 	let apps = T::PalletApps::apps();

		// 	// Ensure the kitty exists and is called by the kitty owner
		// 	ensure!(Self::is_kitty_owner(&app_id, &sender)?, <Error<T>>::NotAppOwner);

		// 	let app = <pallet_apps::Module<T>>::apps(&app_id).ok_or(<Error<T>>::AppNotExist)?;

		// 	let reviews: Review<T> = Self::reviews_owned_by_app(&app_id).ok_or(<Error<T>>::ReviewNotExist)?;

		// 	Self::deposit_event(Event::ReviewCreated { who });
			

		// 	Ok(reviews)
		// }
	}
}
