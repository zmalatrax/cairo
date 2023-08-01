use starknet::ContractAddress;

#[contract_interface]
trait IERC20<TCS> {
    fn balance(ref self: TCS, a: ContractAddress) -> u128;
}
#[contract_interface]
trait IOwnable<TCS> {
    fn is_owner(self: @TCS, user: ContractAddress) -> bool;
}
