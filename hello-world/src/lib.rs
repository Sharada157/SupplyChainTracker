#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Structure to track product information in the supply chain
#[contracttype]
#[derive(Clone)]
pub struct Product {
    pub product_id: u64,
    pub name: String,
    pub qr_code: String,
    pub current_location: String,
    pub timestamp: u64,
    pub status: String,  // "manufactured", "in_transit", "delivered"
}

// Structure to track supply chain statistics
#[contracttype]
#[derive(Clone)]
pub struct ChainStats {
    pub total_products: u64,
    pub in_transit: u64,
    pub delivered: u64,
}

// Mapping product_id to Product details
#[contracttype]
pub enum ProductBook {
    Product(u64)
}

// Constants for storage
const PRODUCT_COUNT: Symbol = symbol_short!("P_COUNT");
const CHAIN_STATS: Symbol = symbol_short!("CH_STATS");

#[contract]
pub struct SupplyChainContract;

#[contractimpl]
impl SupplyChainContract {

    // Function to register a new product in the supply chain
    pub fn register_product(env: Env, name: String, qr_code: String, location: String) -> u64 {
        let mut count: u64 = env.storage().instance().get(&PRODUCT_COUNT).unwrap_or(0);
        count += 1;

        let time = env.ledger().timestamp();
        
        // Create new product instance
        let new_product = Product {
            product_id: count,
            name: name.clone(),
            qr_code: qr_code.clone(),
            current_location: location,
            timestamp: time,
            status: String::from_str(&env, "manufactured"),
        };

        // Update chain statistics
        let mut stats = Self::view_chain_stats(env.clone());
        stats.total_products += 1;

        // Store product data
        env.storage().instance().set(&ProductBook::Product(count), &new_product);
        env.storage().instance().set(&CHAIN_STATS, &stats);
        env.storage().instance().set(&PRODUCT_COUNT, &count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Product registered with ID: {}", count);
        count
    }

    // Function to update product location when scanned at checkpoints
    pub fn update_location(env: Env, product_id: u64, new_location: String, status: String) {
        let mut product = Self::view_product(env.clone(), product_id);

        if product.product_id == 0 {
            log!(&env, "Product not found!");
            panic!("Product not found!");
        }

        let time = env.ledger().timestamp();
        let old_status = product.status.clone();

        // Update product details
        product.current_location = new_location;
        product.timestamp = time;
        product.status = status.clone();

        // Update statistics if status changed
        let mut stats = Self::view_chain_stats(env.clone());
        
        if old_status != status {
            // Update counters based on status transitions
            if status == String::from_str(&env, "in_transit") {
                stats.in_transit += 1;
            } else if status == String::from_str(&env, "delivered") {
                if old_status == String::from_str(&env, "in_transit") {
                    stats.in_transit -= 1;
                }
                stats.delivered += 1;
            }
            env.storage().instance().set(&CHAIN_STATS, &stats);
        }

        // Store updated product
        env.storage().instance().set(&ProductBook::Product(product_id), &product);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Product ID: {} updated to location: {}", product_id, product.current_location);
    }

    // Function to view product details by product_id
    pub fn view_product(env: Env, product_id: u64) -> Product {
        let key = ProductBook::Product(product_id);
        
        env.storage().instance().get(&key).unwrap_or(Product {
            product_id: 0,
            name: String::from_str(&env, "Not_Found"),
            qr_code: String::from_str(&env, "Not_Found"),
            current_location: String::from_str(&env, "Not_Found"),
            timestamp: 0,
            status: String::from_str(&env, "Not_Found"),
        })
    }

    // Function to view overall supply chain statistics
    pub fn view_chain_stats(env: Env) -> ChainStats {
        env.storage().instance().get(&CHAIN_STATS).unwrap_or(ChainStats {
            total_products: 0,
            in_transit: 0,
            delivered: 0,
        })
    }
}
