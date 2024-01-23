use super::utils::serialized;

use cairo_level_tests::contracts::erc20::{erc_20, IERC20LibraryDispatcher, IERC20DispatcherTrait};
use starknet::testing::set_caller_address;

use starknet::class_hash::{class_hash_const, ClassHash};
use starknet::contract_address::contract_address_const;

#[test]
fn test_erc20_transfer() {
    // TODO(yg): revert + add new code as a profiling test.
    // let class_hash: ClassHash = erc_20::TEST_CLASS_HASH.try_into().unwrap();

    // set_caller_address(contract_address_const::<2_felt252>());
    // let contract_address = contract_address_const::<1_felt252>();
    // IERC20LibraryDispatcher { class_hash }.transfer(contract_address, 0_u256);
    set_caller_address(contract_address_const::<2_felt252>());
    erc_20::__external::transfer(array![1, 0, 0].span());
}
