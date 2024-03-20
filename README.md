
# Dutch Auction Marketplace Smart Contract (Ink!)
This document provides an overview of the Dutch auction marketplace smart contract written in Ink!.
A Dutch auction is a type of auction where the price of an asset starts high and gradually decreases over time until a buyer is found or a minimum price is reached.

The smart contract relies on [PSP22 implementation](https://github.com/Cardinal-Cryptography/PSP22).


### Prerequisites
    - Rust programming language with tools installed ([Rust compiler, Cargo package manager](https://doc.rust-lang.org/cargo/getting-started/installation.html))
    - Cargo contract (https://github.com/paritytech/cargo-contract)
    - Familiarity with smart contract development principles

### Clone the Repository

```Bash
git clone https://github.com/inkdevhub/dutch_auction.git
```
### Build

```Bash
cd adutch_auction
cargo contract build --release
```

### Test

```Bash
cargo test
```

## Tokens:
This smart contract facilitates a Dutch auction for two fungible tokens:

- Asset token: Represents the auctioned assets.
- Payment token: Used by buyers to purchase asset tokens.


## Key Features:
- Declining Price: The initial asset price gradually decreases over time until a buyer purchases them, the minimum price is reached or the owner terminates the contract.
- Minimum Price: Serves as a safety net, ending the auction if the decreasing price reaches this point.
- Asset Purchase: Users can dynamically purchase assets with the payment token at the current price.

## Components:
A brief description of the key components.

### Storrage
Keeps data important for the contract functioning, like current price, minimum price, auction duration, and token addresses.

### Events
Emit details to track auction activities, such as ticket purchases, for monitoring and auditing purposes.

### Functions:
- buy_ticket: Allows users to purchase tickets with the ticket token.
- price: Returns the current price of an asset.
- linear_decrease: Takes part in calculating the current asset price.

## Usage:
1. Deploy the smart contract, specifying the `asset_token` ,and `payment_token` contracts' on chain `account_id`, asset `start_price` and `min_price` and the `end_time` of the auction.
2. Users can participate in the auction by calling the buy_ticket function, providing the desired amount of tickets.
3. The contract automatically calculates the price based on the current auction state and transfers the corresponding reward tokens to the buyer upon successful purchase.

## Further Development:
This code serves as a foundational example and requires further enhancements for real-world usage. Consider:

- Security best practices: Implement thorough balance checks, transfer verifications, and access control mechanisms.
- Completeness: Develop functionalities like auction reset and reward token distribution logic.
- Testing and auditing: Rigorously test the contract to identify and address potential vulnerabilities.

### Disclaimer:
This code example is for educational purposes only and should not be used in production environments without proper testing, security audits, and legal considerations.