use support::{
    decl_storage, decl_module, decl_event, ensure, StorageValue, StorageMap, 
    dispatch::Result, Parameter, traits::Currency
};
use runtime_primitives::traits::{SimpleArithmetic, Bounded, One, Member};
use parity_codec::{Encode, Decode, Input, Output};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;
use crate::linked_item::{LinkedList, LinkedItem};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type KittyIndex: Parameter + Member + Bounded + SimpleArithmetic + Default + Copy;
    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

// #[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);


impl Decode for Kitty
{
    fn decode<I: Input>(input: &mut I) -> Option<Self> {
        Some(Kitty(Decode::decode(input)?))
    }
}

impl Encode for Kitty
{
    fn encode_to<T: Output>(&self, dest: &mut T) {
        dest.write(&Encode::encode(&self.0));
    }
}

// #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
// #[derive(Encode, Decode)]
// pub struct KittyLinkedItem<T: Trait> {
//     pub prev: Option<T::KittyIndex>,
//     pub next: Option<T::KittyIndex>,
// }
type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnedKittiesList<T> = LinkedList<OwnedKitties<T>, <T as system::Trait>::AccountId, <T as Trait>::KittyIndex>;

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(kitty): map T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(kitties_count): T::KittyIndex;
        // pub OwnedKitties get(owned_kitties): map (T::AccountId, T::KittyIndex) => T::KittyIndex;
        // pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => T::KittyIndex;
        pub OwnedKitties get(owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

        pub KittyOwners get(kitty_owner): map T::KittyIndex => Option<T::AccountId>;

        pub KittyPrices get(kitty_price): map T::KittyIndex => Option<BalanceOf<T>>;
    }
}

decl_event! (
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::KittyIndex,
        Balance = BalanceOf<T>,
    {   
        // A kitty is created. (owner, kitty_id)
        Created(AccountId, KittyIndex),
        // A kitty is available for sale. (from, to, kitty_id)
        Transferred(AccountId, AccountId, KittyIndex),
        /// A kitty is available for sale. (owner, kitty_id, price)
        Ask(AccountId, KittyIndex, Option<Balance>),
        // A kitty is sold. (from, to, kitty_id, price)
        Sold(AccountId, AccountId, KittyIndex, Balance),

    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);

            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }

        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            ensure!(<OwnedKitties<T>>::exists(&(sender.clone(), Some(kitty_id))), "Only owner can transfer kitty");
            Self::do_transfer(&sender, &to, kitty_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
        }


        pub fn ask(origin, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(<OwnedKitties<T>>::exists(&(sender.clone(), Some(kitty_id))), "Only owner can set price for kitty");

            if let Some(price) = price {
                <KittyPrices<T>>::insert(kitty_id, price);
            } else {
                <KittyPrices<T>>::remove(kitty_id);
            }

            Self::deposit_event(RawEvent::Ask(sender, kitty_id, price));
        }

        pub fn buy(origin, kitty_id: T::KittyIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id);
            ensure!(owner.is_some(), "Kitty does not exist");
            let owner = owner.unwrap();

            let kitty_price = Self::kitty_price(kitty_id);
            ensure!(kitty_price.is_some(), "Kitty not for sale");

            let kitty_price = kitty_price.unwrap();
            ensure!(price >= kitty_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, kitty_price)?;

            <KittyPrices<T>>::remove(kitty_id);

            Self::do_transfer(&owner, &sender, kitty_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, kitty_id, kitty_price));

        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    ((selector & dna1) | (!selector & dna2))
}

// impl<T: Trait> OwnedKitties<T> {
//     fn read_head(account: &T::AccountId) -> KittyLinkedItem<T> {
//         Self::read(account, None)
//     }

//     fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>) {
//         Self::write(account, None, item);
//     }

//     fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T> {
//         <OwnedKitties<T>>::get(&(account.clone(), key)).unwrap_or_else(|| KittyLinkedItem {
//             prev: None,
//             next: None,
//         })
//     }
    
//     fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>) {
//         <OwnedKitties<T>>::insert(&(account.clone(), key), item);
//     }

//     pub fn append (account: &T::AccountId, kitty_id: T::KittyIndex) {
//         let head = Self::read_head(account);
//         let new_head = KittyLinkedItem {
//             prev: Some(kitty_id),
//             next: head.next,
//         };

//         Self::write_head(account, new_head);

//         let prev = Self::read(account, head.prev);
//         let new_prev = KittyLinkedItem {
//             prev: prev.prev,
//             next: Some(kitty_id),
//         };

//         Self::write(account, head.prev, new_prev);

//         let item = KittyLinkedItem {
//             prev: head.prev,
//             next: None,
//         };
//         Self::write(account, Some(kitty_id), item);
//     }

//     pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex) {
//         if let Some(item) = <OwnedKitties<T>>::take(&(account.clone(), Some(kitty_id))) {
//             let prev = Self::read(account, item.prev);
//             let new_prev = KittyLinkedItem {
//                 prev: prev.prev,
//                 next: item.next,
//             };

//             Self::write(account, item.prev, new_prev);

//             let next = Self::read(account, item.next);
//             let new_next = KittyLinkedItem {
//                 prev: item.prev,
//                 next: next.next,
//             };

//             Self::write(account, item.next, new_next);
//         }
//     }
// }

impl<T: Trait> Module<T> {
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            <system::Module<T>>::random_seed(),
            sender.clone(),
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number()
        );
        payload.using_encoded(blake2_128)
    }

    fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value(){
            return Err("Kitties count overflow");
        }
        Ok(kitty_id)
    }

    fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
        // <OwnedKitties<T>>::append(owner, kitty_id);
        <OwnedKittiesList<T>>::append(owner, kitty_id);
    }

    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        <Kitties<T>>::insert(kitty_id, kitty);
        <KittiesCount<T>>::put(kitty_id + One::one());
        <KittyOwners<T>>::insert(kitty_id, owner);

        // let user_kitties_id = Self::owned_kitties_count(owner.clone());
        // <OwnedKitties<T>>::insert((owner.clone(), user_kitties_id), kitty_id);
        // <OwnedKittiesCount<T>>::insert(owner, user_kitties_id + One::one());
        Self::insert_owned_kitty(owner, kitty_id);
    }

    fn do_transfer(from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex) {
        // <OwnedKitties<T>>::remove(&from, kitty_id);
        // <OwnedKitties<T>>::append(&to, kitty_id);
        <OwnedKittiesList<T>>::remove(&from, kitty_id);
		<OwnedKittiesList<T>>::append(&to, kitty_id);
        <KittyOwners<T>>::insert(kitty_id, to);
    }

    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> result::Result<T::KittyIndex, &'static str> {
        let kitty1 = Self::kitty(kitty_id_1);
        let kitty2 = Self::kitty(kitty_id_2);

        ensure!(kitty1.is_some(), "Invalid kitty_id_1");
        ensure!(kitty2.is_some(), "Invalid kitty_id_2");
        ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");
        ensure!(Self::kitty_owner(&kitty_id_1).map(|owner| owner == *sender).unwrap_or(false), "Not onwer of kitty1");
        ensure!(Self::kitty_owner(&kitty_id_2).map(|owner| owner == *sender).unwrap_or(false), "Not onwer of kitty2");

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.unwrap().0;
        let kitty2_dna = kitty2.unwrap().0;

        let selector = Self::random_value(&sender);

        let mut new_dna = [0u8; 16];
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
        
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

        Ok((kitty_id))
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq, Debug)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
    impl balances::Trait for Test {
        type Balance = u32;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();

        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
    }
	impl Trait for Test {
		type KittyIndex = u32;
        type Currency = balances::Module<Test>;
	}
	type KittyModule = Module<Test>;
    type OwnedKittiesTest = OwnedKitties<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			OwnedKittiesTest::append(&0, 1);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(1),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            OwnedKittiesTest::append(&0, 2);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(2),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(2),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
                prev: Some(1),
                next: None,
            }));

            OwnedKittiesTest::append(&0, 3);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(2),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
                prev: Some(1),
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: Some(2),
                next: None,
            }));

		});
	}

    #[test]
    fn owned_kitties_can_remove_values() {
        with_externalities(&mut new_test_ext(), || {
            OwnedKittiesTest::append(&0, 1);
            OwnedKittiesTest::append(&0, 2);
            OwnedKittiesTest::append(&0, 3);

            OwnedKittiesTest::remove(&0, 2);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: Some(1),
                next: None,
            }));

            OwnedKittiesTest::remove(&0, 1);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            OwnedKittiesTest::remove(&0, 3);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), None);
        });
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq, Debug)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type KittyIndex = u32;
	}
	type KittyModule = Module<Test>;
    type OwnedKittiesTest = OwnedKitties<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			OwnedKittiesTest::append(&0, 1);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(1),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            OwnedKittiesTest::append(&0, 2);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(2),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(2),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
                prev: Some(1),
                next: None,
            }));

            OwnedKittiesTest::append(&0, 3);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(2),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
                prev: Some(1),
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: Some(2),
                next: None,
            }));

		});
	}

    #[test]
    fn owned_kitties_can_remove_values() {
        with_externalities(&mut new_test_ext(), || {
            OwnedKittiesTest::append(&0, 1);
            OwnedKittiesTest::append(&0, 2);
            OwnedKittiesTest::append(&0, 3);

            OwnedKittiesTest::remove(&0, 2);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(1),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
                prev: None,
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: Some(1),
                next: None,
            }));

            OwnedKittiesTest::remove(&0, 1);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: Some(3),
                next: Some(3),
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            OwnedKittiesTest::remove(&0, 3);

            assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
                prev: None,
                next: None,
            }));

            assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

            assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), None);
        });
    }
}