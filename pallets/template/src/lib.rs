#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec;

	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct DepositData<AccountId> {
		asset_id: u128,
		amount: u128,
		beneficiary: AccountId
	}

	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct DepositWithSignature<AccountId> {
		deposit_data: DepositData<AccountId>,
		signature: BoundedVec<u8, AccountLimit>
	}

	#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
	pub struct AccountLimit;
	impl Get<u32> for AccountLimit {
		fn get() -> u32 {
			5 // TODO: Arbitrary value
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn get_unverifed_block)]
	pub(super) type UnverifiedSignature<T: Config> =
	StorageMap<_, Blake2_128Concat, T::BlockNumber, BoundedVec<DepositWithSignature<T::AccountId>, AccountLimit>, ValueQuery>;

	// All Relayers
	#[pallet::storage]
	#[pallet::getter(fn get_all_relayers)]
	pub(super) type Relayers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, AccountLimit>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_unverified_signature(origin: OriginFor<T>, deposit_with_signature: DepositWithSignature<T::AccountId>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let caller = ensure_signed(origin)?;
			let relayers = <Relayers<T>>::get();
			ensure!(relayers.contains(&caller), Error::<T>::NoneValue);
			let current_block_number: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
			let mut deposits = <UnverifiedSignature<T>>::get(current_block_number);
			if deposits.is_empty() {
				<UnverifiedSignature<T>>::insert(current_block_number, BoundedVec::try_from(vec![deposit_with_signature]).unwrap());
			} else {
				deposits.try_push(deposit_with_signature);
				 <UnverifiedSignature<T>>::insert(current_block_number, deposits);
			}
			// Emit an event.
			//Self::deposit_event(Event::SomethingStored { something, who });
			Ok(())
		}
	}
}
