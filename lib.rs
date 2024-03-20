#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Dutch Auction Contract
///
/// This Ink! smart contract implements a Dutch auction. A Dutch auction is a type of auction where the price
/// of an asset starts high and gradually decreases over time until a buyer is found or a minimum price is reached.
///
/// Contract Storage:
/// - auction_owner: The account ID of the auction owner.
/// - asset_token: The account ID of the token representing the asset being auctioned.
/// - payment_token: The account ID of the token used for payment.
/// - start_price: The starting price of the auction.
/// - min_price: The minimum price of the auction.
/// - start_time: The block number at which the auction starts.
/// - end_time: The block number at which the auction ends.
///
/// Contract Events:
/// - AssetBought: Emitted when an asset is bought.
///
/// Error Types:
/// - PSP22TokenCall: An error occurred while interacting with the PSP22 token contract.
/// - MaxPriceExceeded: The current price is higher than the limit set buy the payer.
/// - InsufficientSupplyToken: The contract does not have enough tokens to fulfill the request.
/// - NotAuctionOwner: The caller is not the auction owner.
///
/// Messages:
/// - end_time: Returns the block number at which the auction ends.
/// - start_block: Returns the block number at which the auction starts.
/// - price: Returns the current price of the asset.
/// - available_asset: Returns the number of available asset tokens.
/// - min_price: Returns the minimum price of the auction.
/// - set_min_price: Updates the minimum price of the auction. Only the auction owner can call this message.
/// - set_end_time: Updates the end time of the auction. Only the auction owner can call this message.
/// - buy: Buys a specified amount of asset tokens at the current price. The caller must provide approval
///        for the `payment_token` before calling this message.
/// - terminate: Terminates the contract. Only the auction owner can call this message.
///
/// Additional Functions:
/// - current_price: Calculates the current price of the asset based on the starting price, minimum price,
///        start time, end time, and current block number.
/// - take_payment: Takes payment from the caller for the specified amount.
/// - give_asset: Transfers the specified amount of asset tokens to the caller.
/// - asset_balance: Gets the balance of the asset token held by the contract.
/// - linear_decrease: Calculates the linear interpolation between two points.
/// - check_owner: Checks if the caller is the auction owner.

#[ink::contract]
mod dutch_auction {
    use ink::{contract_ref, prelude::vec};
    use psp22::{PSP22Error, PSP22};

    #[ink(storage)]
    pub struct DutchAuction {
        auction_owner: AccountId,
        asset_token: contract_ref!(PSP22),
        payment_token: contract_ref!(PSP22),
        start_price: u128,
        min_price: u128,
        start_time: BlockNumber,
        end_time: BlockNumber,
    }

    #[derive(Eq, PartialEq, Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        PSP22TokenCall(PSP22Error),
        MaxPriceExceeded,
        InsufficientSupplyToken,
        NotAuctionOwner,
    }

    #[ink(event)]
    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct AssetBought {
        #[ink(topic)]
        pub by: AccountId,
        pub price: u128,
        pub amount: u128,
    }

    impl From<PSP22Error> for Error {
        fn from(inner: PSP22Error) -> Self {
            Error::PSP22TokenCall(inner)
        }
    }

    impl DutchAuction {
        /// Constructor that initializes the contract storrage.
        ///
        /// Caller would be the auction_owner
        #[ink(constructor)]
        pub fn new(
            asset_token: AccountId,
            payment_token: AccountId,
            start_price: u128,
            min_price: u128,
            end_time: BlockNumber,
        ) -> Self {
            Self {
                auction_owner: Self::env().caller(),
                asset_token: asset_token.into(),
                payment_token: payment_token.into(),
                start_price,
                min_price,
                start_time: Self::env().block_number(),
                end_time,
            }
        }

        /// The block after which the price will no longer decrease.
        ///
        /// The contract will decrease the price linearly from start_price()
        /// to `min_price()` over the period from `start_time()` to 'end_time()`.
        /// The auction doesn't end after the period elapses -
        /// the asset remains available for purchase at `min_price()`.
        #[ink(message)]
        pub fn end_time(&self) -> BlockNumber {
            self.end_time
        }

        /// The block at which the auction starts
        #[ink(message)]
        pub fn start_block(&self) -> BlockNumber {
            self.start_time
        }

        /// The price the contract would charge when buying at the current block.
        #[ink(message)]
        pub fn price(&self) -> u128 {
            self.current_price()
        }

        /// Amount of tokens available for sale.
        #[ink(message)]
        pub fn available_asset(&self) -> u128 {
            self.asset_balance()
        }

        /// The minimal price the contract allows.
        #[ink(message)]
        pub fn min_price(&self) -> u128 {
            self.min_price
        }

        /// Update the minimal price.
        ///
        /// Requires auction_owner to execute.
        #[ink(message)]
        pub fn set_min_price(&mut self, value: u128) -> Result<(), Error> {
            self.check_owner(self.env().caller())?;
            self.min_price = value;

            Ok(())
        }

        /// Update the length of the auction.
        ///
        /// Requires auction_owner to execute.
        #[ink(message)]
        pub fn set_end_time(&mut self, end_time: BlockNumber) -> Result<(), Error> {
            self.check_owner(self.env().caller())?;
            self.end_time = end_time;

            Ok(())
        }

        /// Buy `asset_tokens` at the `current_price`.
        ///
        /// The caller should provide a positive `amount` of 'asset_tokens' to purchase.
        ///
        /// The caller should make an approval for at least `price()*amount` reward tokens to make sure the
        /// call will succeed. The caller can specify a `max_price` - the call will fail if the
        /// current price is greater than that.
        #[ink(message)]
        pub fn buy(&mut self, amount: u128, max_price: Option<Balance>) -> Result<(), Error> {
            if self.available_asset() < amount || amount < 1 {
                return Err(Error::InsufficientSupplyToken);
            }

            let price = self.current_price().saturating_mul(amount);
            if let Some(max_price) = max_price {
                if price > max_price {
                    return Err(Error::MaxPriceExceeded);
                }
            }

            let caller = self.env().caller();

            self.take_payment(caller, price)?;
            self.give_asset(caller, amount)?;

            self.env().emit_event(AssetBought {
                price,
                by: caller,
                amount,
            });

            Ok(())
        }

        /// Terminates the contract
        ///
        /// Requires auction_owner to execute.
        #[ink(message)]
        pub fn terminate(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();
            self.check_owner(caller)?;
            self.env().terminate_contract(caller)
        }

        fn current_price(&self) -> u128 {
            Self::linear_decrease(
                self.start_time.into(),
                self.start_price,
                self.end_time.into(),
                self.min_price,
                self.env().block_number().into(),
            )
            .max(self.min_price)
        }

        fn take_payment(&mut self, from: AccountId, amount: u128) -> Result<(), Error> {
            let call = self
                .payment_token
                .transfer_from(from, self.auction_owner, amount, vec![]);

            match call {
                Err(psp22_err) => Err(Error::from(psp22_err)),
                Ok(()) => Ok(()),
            }
        }

        fn give_asset(&mut self, to: AccountId, amount: u128) -> Result<(), Error> {
            let call = self.asset_token.transfer(to, amount, vec![]);

            match call {
                Err(psp22_err) => Err(Error::from(psp22_err)),
                Ok(()) => Ok(()),
            }
        }

        fn asset_balance(&self) -> u128 {
            self.asset_token.balance_of(self.auction_owner)
        }

        /// Returns (an approximation of) the linear function passing through `(x_start, y_start)` and `(x_end, y_end)` at
        /// `x`. If `x` is outside the range of `x_start` and `x_end`, the value of `y` at the closest endpoint is returned.
        fn linear_decrease(
            x_start: u128,
            y_start: u128,
            x_end: u128,
            y_end: u128,
            x: u128,
        ) -> u128 {
            let steps = x.saturating_sub(x_start);
            let x_span = x_end.saturating_sub(x_start);
            let y_span = y_start.saturating_sub(y_end);

            if x >= x_end {
                y_end
            } else if x <= x_start {
                y_start
            } else if y_span > x_span {
                let y_per_x = y_span.saturating_div(x_span);
                y_start.saturating_sub(steps.saturating_mul(y_per_x))
            } else {
                let x_per_y = x_span.saturating_div(y_span);
                y_start.saturating_sub(steps.saturating_div(x_per_y))
            }
        }

        fn check_owner(&self, account: AccountId) -> Result<(), Error> {
            if account != self.auction_owner {
                return Err(Error::NotAuctionOwner);
            }

            Ok(())
        }
    }
}
