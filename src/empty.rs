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
    pub user : ManagedAddress<M>,
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
        subscription.last_payment = self.blockchain().get_block_timestamp();
        subscription.user = self.blockchain().get_caller();
        //TO DO: check the token is whitelisted
        let service = self.service_storage(&subscription.service_id).get();
        self.pay_subscription(&mut subscription, &service);
        self.subscriptions_storage().insert(subscription);
     }

     #[endpoint]
     fn unsubscribe(&self, subscription : Subscription<Self::Api>) {
        let caller = self.blockchain().get_caller();
        self.subscriptions_storage().remove(&subscription);
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
        for mut subscription in self.subscriptions_storage().iter() {
            if subscription.service_id == service_id {
                if self.blockchain().get_block_timestamp() - subscription.last_payment >= service.expires_in {
                    self.pay_subscription(&mut subscription, &service);
                }
            }
        }
    }

   fn pay_subscription(&self, subscription : &mut Subscription<Self::Api>, service : &Service<Self::Api>) {
        let current_timestamp = self.blockchain().get_block_timestamp();
        subscription.last_payment = current_timestamp;
        self.subscriptions_storage().insert(subscription.clone());
        let pair_address = self.whitelist_storage(&subscription.token).get();
        //TO DO: check if the user has enough funds for the next period, if not cancel the subscription
        self.pay_services(&service, &subscription);
   }

    fn pay_services(&self, service: &Service<Self::Api>, subscription : &Subscription<Self::Api>) {
        let mut services = ManagedVec::<Self::Api, Service<Self::Api>>::new();
        services.push(service.clone());

        while services.len() > 0 {
            let head = services.get(0);
            services.remove(0);
            let payment = EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from("USDC-c76f1f")),
                0,
                head.price.into()
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

            self.balance_storage(&subscription.user, &subscription.token).update(|deposit| *deposit -= big_uint_variable.clone());
            self.send().direct(&head.owner, &subscription.token, 0, &big_uint_variable);
            services.extend(&head.depends_on)
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
    fn subscriptions_storage(&self) -> SetMapper<Subscription<Self::Api>>;
    
}
