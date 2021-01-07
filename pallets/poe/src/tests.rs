use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn creat_claim_works() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(execute:||{
			let claim:Vec<i32>=vec![0,1]；
			assert_ok!(PoeModule::create_claim(Origin::signed(1),claim.clone()));
			assert_eq!(Proofs::<Test>::get(&claim),(1,frame_system::Module::<Test>::block_number()));
		})
	});
}

#[test]
fn creat_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(execute:||{
			let claim:Vec<i32>=vec![0,1]；
			let _=PoeModule::create_claim(Origin::signed(1),claim.clone());

			assert_noop!(
				PoeModule::create_claim(Origin::signed(1),claim.clone()),
				Error::<Test>::ProofAlreadyExist
			);
		})
	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(execute:||{
			let claim:Vec<i32>=vec![0,1]；
			let _=PoeModule::create_claim(Origin::signed(1),claim.clone());

			assert_ok!(PoeModule::create_claim(Origin::signed(1),claim.clone()));
		})
	});
}


#[test]
fn creat_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(execute:||{
			let claim:Vec<i32>=vec![0,1]；

			assert_noop!(
				PoeModule::create_claim(Origin::signed(1),claim.clone()),
				Error::<Test>::ClaimNotExist
			);
		})
	});
}