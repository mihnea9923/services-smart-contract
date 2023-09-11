#![no_std]

multiversx_sc::derive_imports!();
multiversx_sc::imports!();

const FIVE_MINUTES_AGO : u64 = 300; 

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct Service<M : ManagedTypeApi> {
    pub id: BigUint<M>,
    pub expires_in: u64,
    pub price : u64,
    pub owner : ManagedAddress<M>,
    pub depends_on : ManagedVec<M,Service<M>>
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct Subscription<M : ManagedTypeApi> {
    pub service_id: BigUint<M>,
    pub last_payment: u64,
    pub token : EgldOrEsdtTokenIdentifier<M>
}


#[multiversx_sc::contract]
pub trait SubscriptionContractContract {
    #[init]
    fn init(&self) {}

    #[proxy]
    fn pair_proxy(&self) -> pair::Proxy<Self::Api>;

    #[endpoint]
    #[payable("*")]
    fn deposit(&self) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();
        //TO DO: check the token is whitelisted
        let caller = self.blockchain().get_caller();
        self.balance_storage(&caller, &token).update(|deposit| *deposit += payment);
    }

    #[endpoint]
    fn withdraw(&self, amount: &BigUint, token: &EgldOrEsdtTokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let deposit = self.balance_storage(&caller, &token).get();

        if deposit - amount >= 0u32 {
            self.balance_storage(&caller, &token).update(|deposit_val| *deposit_val -= amount);
            self.send().direct(&caller, &token, 0, &amount);
        }
     }

     #[endpoint]
     fn subscribe(&self, mut subscription : Subscription<Self::Api>) {
        let caller = self.blockchain().get_caller();
        subscription.last_payment = self.blockchain().get_block_timestamp();
        //TO DO: check the token is whitelisted
        //TO DO: pay the first period of subscription
        self.subscriptions_storage().insert(caller ,subscription);
     }

    //This method handles both registration and price + periodicity of a service
    #[endpoint]
    fn whitelist_token(&self, token: &EgldOrEsdtTokenIdentifier, pair_address: &ManagedAddress, whitelist : bool) {
        require!(self.blockchain().get_caller() == self.blockchain().get_owner_address(), "only contract owner can whitelist tokens");
        if whitelist {
            self.whitelist_storage(token).set(pair_address)
        }
        else {
            self.whitelist_storage(token).clear();
        }
    }

    #[endpoint]
    fn register_service(&self, mut service : Service<Self::Api>) {
        require!(service.id > 0, "id must be greater than 0");
        require!(service.expires_in > 0, "id must be greater than 0");
        require!(service.price > 0, "id must be greater than 0");
        service.owner = self.blockchain().get_caller();
        //use set_if_empty to ensure no other person can override your service by providing the same id
        self.service_storage(&service.id).set_if_empty(service);
    }

    //Don't think you can schedule a method execute at certain time intervals on blockchain, so I define this endpoint
    //and expect the service owner to have something like a cron job which calls this endpoint
    #[endpoint]
    fn collect_service_fees(&self, service_id : BigUint) {
        let service = self.service_storage(&service_id).get();

        for (caller, mut subscription) in self.subscriptions_storage().iter() {
            if subscription.service_id == service_id {
                let current_timestamp = self.blockchain().get_block_timestamp();
                let time_since_last_payment = current_timestamp - subscription.last_payment;
                subscription.last_payment = current_timestamp;
                self.subscriptions_storage().insert(caller.clone(), subscription.clone());
                //maybe there are 3 periods of time passing without this endpoint beeing called so the user would have to pay for all 3 of them
                let times_to_pay = time_since_last_payment / service.expires_in + if time_since_last_payment % service.expires_in != 0 { 1 } else { 0 };
                let pair_address = self.whitelist_storage(&subscription.token).get();

                let payment = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from("USDC-c76f1f")),
                    0,
                    (times_to_pay * service.price).into()
                );
                // self.pair_proxy().contract(self.blockchain().get_owner_address()).get_safe_price(
                //     pair_address,
                //     current_timestamp,
                //     current_timestamp - FIVE_MINUTES_AGO,
                //     payment
                // ).execute_on_dest_context();

                //this should come as a reponse from the get_safe_price method
                let value: u64 = 100;
                let big_uint_variable: BigUint<Self::Api> = value.into();
                self.balance_storage(&caller, &subscription.token).update(|deposit| *deposit -= big_uint_variable.clone());
                self.send().direct(&service.owner, &subscription.token, 0, &big_uint_variable);
                self.pay_dependent_services(&service, &subscription.token);
                //TO DO: check if the user has enough funds for the next period, if not cancel the subscription
            }
        }
    }

    fn pay_subscription(&self, user: &ManagedAddress, service: &Service<Self::Api>, subscription : &mut Subscription<Self::Api>){
        let current_timestamp = self.blockchain().get_block_timestamp();
        let time_since_last_payment = current_timestamp - subscription.last_payment;
        subscription.last_payment = current_timestamp;
        self.subscriptions_storage().insert(user.clone(), subscription.clone());
        //maybe there are 3 periods of time passing without this endpoint beeing called so the user would have to pay for all 3 of them
        let times_to_pay = time_since_last_payment / service.expires_in + if time_since_last_payment % service.expires_in != 0 { 1 } else { 0 };
        let pair_address = self.whitelist_storage(&subscription.token).get();

        let payment = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from("USDC-c76f1f")),
            0,
            (times_to_pay * service.price).into()
        );
        // self.pair_proxy().contract(self.blockchain().get_owner_address()).get_safe_price(
        //     pair_address,
        //     current_timestamp,
        //     current_timestamp - FIVE_MINUTES_AGO,
        //     payment
        // ).execute_on_dest_context();

        //this should come as a reponse from the get_safe_price method
        let value: u64 = 100;
        let big_uint_variable: BigUint<Self::Api> = value.into();
        self.balance_storage(&user, &subscription.token).update(|deposit| *deposit -= big_uint_variable.clone());
        self.send().direct(&service.owner, &subscription.token, 0, &big_uint_variable);
        self.pay_dependent_services(&service, &subscription.token);
        //TO DO: check if the user has enough funds for the next period, if not cancel the subscription
    }

    fn pay_dependent_services(&self, service: &Service<Self::Api>, token : &EgldOrEsdtTokenIdentifier) {
        let mut dependencies = ManagedVec::<Self::Api, Service<Self::Api>>::new();
        dependencies.push(service.clone());

        while dependencies.len() > 0 {
            let head = dependencies.get(0);
            dependencies.remove(0);
            if head.id != service.id {
                self.send().direct(&head.owner, &token, 0, &head.price.into());
            }
            dependencies.extend(&head.depends_on)
        }
    }

    #[view(whitelistedTokens)]
    #[storage_mapper("whitelist")]
    fn whitelist_storage(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<ManagedAddress>;

    #[view(service)]
    #[storage_mapper("service")]
    fn service_storage(&self, id: &BigUint) -> SingleValueMapper<Service<Self::Api>>;

    #[view(getDeposit)]
    #[storage_mapper("balanceStorage")]
    fn balance_storage(&self, caller: &ManagedAddress, token_identifier: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(usersubscriptions)]
    #[storage_mapper("subscriptions")]
    fn subscriptions_storage(&self) -> MapMapper<ManagedAddress,Subscription<Self::Api>>;
    
}
