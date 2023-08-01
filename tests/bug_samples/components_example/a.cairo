// #[starknet::component]
mod a {
    use starknet::ContractAddress;
    use hello_scarb::interfaces::{IERC20, IOwnable};

    // autogen.
    trait HasComponent<TCS> {
        fn get_component(self: @TCS) -> @ComponentState<TCS>;
        fn get_component_mut(ref self: TCS) -> ComponentState<TCS>;
        fn get_contract(self: @ComponentState<TCS>) -> @TCS;
        fn get_contract_mut(ref self: ComponentState<TCS>) -> TCS;
        // TODO: Make this an associated impl of EventEmitter.
        fn emit(ref self: ComponentState<TCS>, event: Event);
    }

    // #[storage]
    struct Storage {
        data: u32
    }
    // autogen.
    // #[derive(Drop)]
    struct ComponentState<TCS> {
        data: DataVar, 
    }
    impl ComponentStateDrop<TCS> of Drop<ComponentState<TCS>> {}
    #[derive(Drop)]
    struct DataVar {}
    #[generate_trait]
    impl DataVarImpl of DataVarTrait {
        fn read(self: @DataVar) -> u32 {
            0
        }
        fn write(ref self: DataVar, val: u32) {}
    }

    #[event]
    #[derive(Drop, starknet::event)]
    enum Event {
        Log: Log, 
    }
    #[derive(Drop, starknet::event)]
    struct Log {}
    // autogen.
    // EventEmitter

    // #[forward(AImpl)]
    #[generate_trait]
    impl AInnerImpl<TCS, impl X: HasComponent<TCS>, impl Y: IOwnable<TCS>> of ATrait<TCS, X, Y> {
        #[external]
        fn foo(self: @ComponentState<TCS>) {
            self.data.read();
        // self.emit(Log {})
        }
        #[external]
        fn bar(ref self: ComponentState<TCS>, user: ContractAddress) {
            if self.get_contract().is_owner(user) {
                self.data.write(5);
            }
        }
    }
    // autogen.
    #[generate_trait]
    impl AImpl<TCS, impl X: HasComponent<TCS>, impl Y: IOwnable<TCS>> of AImplTrait<TCS, X, Y> {
        #[external]
        fn foo(self: @TCS) {
            self.get_component().foo()
        }
        #[external]
        fn bar(ref self: TCS, user: ContractAddress) {
            self.bar(user)
        }
    }

    // #[forward(AERC20Impl)]
    impl AERC20InnerImpl<TCS, impl X: HasComponent<TCS>> of IERC20<ComponentState<TCS>> {
        fn balance(ref self: ComponentState<TCS>, a: ContractAddress) -> u128 {
            10
        }
    }
    // autogen.
    #[generate_trait]
    impl AERC20Impl<
        TCS, impl X: HasComponent<TCS>, impl TCSPDrop: PanicDestruct<TCS>
    > of AERC20ImplTrait<TCS> {
        fn balance(ref self: TCS, a: ContractAddress) -> u128 {
            let mut component = self.get_component_mut();
            component.balance(a)
        }
    }
}

