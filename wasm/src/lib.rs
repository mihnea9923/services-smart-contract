// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            9
// Async Callback (empty):               1
// Total number of exported functions:  11

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    subscription_contract
    (
        init => init
        deposit => deposit
        withdraw => withdraw
        subscribe => subscribe
        whitelist_token => whitelist_token
        register_service => register_service
        whitelistedTokens => whitelist_storage
        service => service_storage
        getDeposit => balance_storage
        usersubscriptions => subscriptions_storage
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
