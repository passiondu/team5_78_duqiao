#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet for demo benchmark

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, dispatch,
    traits::{
        Get
    },
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    // Add other types and constants required to configure this pallet.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    // It is important to update your storage name so that your pallet's
    // storage items are isolated from other pallets.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as TemplateModule {
        // Just a dummy storage item.
        // Here we are declaring a StorageValue, `Something` as a Option<u32>
        // `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
        Something get(fn something): Option<u32>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        /// Just a dummy event.
        /// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        /// To emit this event, we call the deposit function, from our runtime functions
        SomethingStored(u32, AccountId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        /// Value reached maximum and cannot be incremented further
        StorageOverflow,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        // this includes information about your errors in the node's metadata.
        // it is needed only if you are using errors in your pallet
        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        /// Just a dummy entry point.
        /// function that can be called by the external world as an extrinsics call
        /// takes a parameter of the type `AccountId`, stores it, and emits an event
        /// # <weight>
        /// - Base Weight: 31.76 µs
        /// - DB Weight: 1 Write
        /// # </weight>
        #[weight = T::DbWeight::get().writes(1) + 29_430_000]
        pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
            // Check it was signed and get the signer. See also: ensure_root and ensure_none
            let who = ensure_signed(origin)?;

            // Code to execute when something calls this.
            // For example: the following line stores the passed in u32 in the storage
            Something::put(something);

            // Here we are raising the Something event
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            Ok(())
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking {
    use super::*;
    use frame_benchmarking::{benchmarks, account};
    use frame_system::RawOrigin;
    use sp_std::prelude::*;

    benchmarks!{
        _ {
            let b in 1 .. 1000 => ();
        }

        do_something {
            let b in ...;
            let caller = account("caller", 0, 0);
        }: _ (RawOrigin::Signed(caller), b.into())
        verify {
            let value = Something::get();
            assert_eq!(value, b.into());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::mock::{new_test_ext, Test};
        use frame_support::assert_ok;

        #[test]
        fn test_benchmarks() {
            new_test_ext().execute_with(|| {
                assert_ok!(test_benchmark_do_something::<Test>());
            });
        }
    }
}
