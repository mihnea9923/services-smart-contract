#![no_std]

multiversx_sc::derive_imports!();
multiversx_sc::imports!();
use multiversx_sc::types::heap::Vec;

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

        //Not sure if we should restrict the user from depositing tokens that are not whitelisted, but considering the goal of
        //this assigment I will asume not :)
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
      
        //TO DO: add a check he has enough money to subscribe for at least 1 unit of time periodicity of that service
        self.subscriptions_storage(&caller).push(&subscription);
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

    #[view(whitelistedTokens)]
    #[storage_mapper("whitelist")]
    fn whitelist_storage(&self, token: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<ManagedAddress>;

    #[view(service)]
    #[storage_mapper("service")]
    fn service_storage(&self, id: &BigUint) -> SingleValueMapper<Service<Self::Api>>;

    #[view(getDeposit)]
    #[storage_mapper("balanceStorage")]
    fn balance_storage(&self, donor: &ManagedAddress, token_identifier: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(usersubscriptions)]
    #[storage_mapper("subscriptions")]
    fn subscriptions_storage(&self, user : &ManagedAddress) -> VecMapper<Subscription<Self::Api>>;

    fn compute_service_payment(&self, user : &ManagedAddress) {
        let end_round = self.blockchain().get_block_timestamp();
        let start_round = end_round - FIVE_MINUTES_AGO;
        let mut subscriptions_vec = ManagedVec::<Self::Api, Subscription<Self::Api>>::new();
        for sub in self.subscriptions_storage(&user).iter() {
            subscriptions_vec.push(sub);
        }
        
        while subscriptions_vec.len() > 0 {
            let current_sub : Subscription<Self::Api> = subscriptions_vec.get(0);
            subscriptions_vec.remove(0);
            let service = self.service_storage(&current_sub.service_id).get();
            let times_since_last_payment = self.blockchain().get_block_timestamp() - subscriptions_vec.last_payment;
            let times_to_pay = times_since_last_payment / 
        }
    }
}
