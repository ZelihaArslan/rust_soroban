//! This contract implements trading of one token pair between one seller and
//! multiple buyer.
//! It demonstrates one of the ways of how trading might be implemented.
#![no_std]

use soroban_sdk::{
contract, contractimpl, contracttype, token, unwrap::UnwrapOptimized, Address, Env

};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Offer,
}
// Represents an offer managed by the SingleOffer contract.
// If a seller wants to sell 1000 XLM for 100 USDC the `sell_price` would be 1000
// and `buy_price` would be 100 (or 100 and 10, or any other pair of integers
// in 10:1 ratio).
#[derive(Clone)]
#[contracttype]
pub struct Offer {
     // Owner of this offer. Sells sell_token to get buy_token.
    pub seller : Address,
    pub sell_token: Address,
    pub buy_token: Address,
    // Seller-defined price of the sell token in arbitrary units.
    pub sell_price: u32,
    // Seller-defined price of the buy token in arbitrary units.
    pub buy_price: u32,
}

#[contract]
pub struct SingleOffer;

#[contractimpl]
impl SingleOffer{
    pub fn create(e: Env, seller:Address, sell_token:Address, buy_token:Address, sell_price:u32, buy_price:u32,){
        if e.storage().instance().has(&DataKey::Offer){
            panic!("Offer is already created");
        }
        if buy_price == 0 || sell_price == 0{
            panic!("Zero price is not allowed");
        }
        seller.require_auth();
        write_offer(
            &e,
            &Offer {
                seller,
                sell_token,
                buy_token,
                sell_price,
                buy_price,
            },
        );
    }

    pub fn trade(e: Env, buyer:Address, buy_token_amount: i128, min_sell_token_amount: i128){
        buyer.require_auth();
        let offer = load_offer(&e);
        let sell_token_client = token::Client::new(&e, &offer.sell_token);
        let buy_token_client = token::Client::new(&e, &offer.buy_token);

        let sell_token_amount = buy_token_amount
            .checked_mul(offer.sell_price as i128)
            .unwrap_optimized()
            / offer.buy_price as i128;
        
        if sell_token_amount <min_sell_token_amount {
            panic!("Price is too low");
        }

        let contract = e.current_contract_address();

        buy_token_client.transfer(&buyer, &contract, &buy_token_amount);
        sell_token_client.transfer(&contract,&buyer,&sell_token_amount);
        buy_token_client.transfer(&contract,&offer.seller, &buy_token_amount);
    }
    pub fn withdraw(e:Env, token: Address, amount: i128){
        let offer= load_offer(&e);
        offer.seller.require_auth();
        token:: Client::new(&e, &token).transfer(
            &e.current_contract_address(),
            &offer.seller,
            &amount,
        );
    }
    pub fn updt_price(e:Env, sell_price:u32, buy_price:u32){
        if buy_price == 0 || sell_price  == 0 {
            panic!("zero price is not allowed");
        }
        let mut offer = load_offer(&e);
        offer.seller.require_auth();
        offer.sell_price = sell_price;
        offer.buy_price = buy_price;
        write_offer(&e,&offer);

   }
   pub fn get_offer(e: Env) ->Offer{
    load_offer(&e)
   }
}

fn load_offer(e: &Env) ->Offer{
    e.storage().instance().get(&DataKey::Offer).unwrap()
}

fn write_offer(e: &Env, offer: &Offer){
    e.storage().instance().set(&DataKey::Offer, offer)
}

mod test;