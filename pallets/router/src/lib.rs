// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Router for Automatic Market Maker (AMM)
//!
//! Given a supported `route`, executes the indicated trades on all the available AMM(s) pool(s).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::{
        ensure,
        pallet_prelude::DispatchResultWithPostInfo,
        traits::{
            tokens::fungibles::{self, Inspect},
            Get, Hooks, IsType,
        },
        transactional, BoundedVec, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use primitives::{currency::CurrencyId, AssetId, Balance, AMM};
    use sp_runtime::traits::Zero;

    pub type Route<T, I> = BoundedVec<
        (
            // Base asset
            CurrencyId,
            // Quote asset
            CurrencyId,
        ),
        <T as Config<I>>::MaxLengthRoute,
    >;

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        frame_system::Config
        + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
        + pallet_amm::Config
    {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// Router pallet id
        #[pallet::constant]
        type RouterPalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMM: AMM<Self>;

        /// Weight information for extrinsics in this pallet.
        type AMMRouterWeightInfo: WeightInfo;

        /// How many routes we support at most
        #[pallet::constant]
        type MaxLengthRoute: Get<u32>;

        /// Currency type for deposit/withdraw assets to/from amm route
        /// module
        type AMMCurrency: fungibles::Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + fungibles::Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + fungibles::Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Input balance must not be zero
        ZeroBalance,
        /// Must input one route at least
        EmptyRoute,
        /// User hasn't enough tokens for transaction
        InsufficientBalance,
        /// The expiry is smaller than current block number
        TooSmallExpiry,
        /// Exceed the max length of routes we allow
        ExceedMaxLengthRoute,
        /// Input duplicated route
        DuplicatedRoute,
        /// We received less coins than the minimum amount specified
        UnexpectedSlippage,
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Event emitted when swap is successful
        /// [sender, amount_in, route, amount_out]
        TradedSuccessfully(T::AccountId, Balance, Route<T, I>, Balance),
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// According specified route order to execute which pool or AMM instance.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_in`: the amount of trading assets
        /// - `min_amount_out`:
        /// - `expiry`:
        #[pallet::weight(T::AMMRouterWeightInfo::trade())]
        #[transactional]
        pub fn trade(
            origin: OriginFor<T>,
            route: Route<T, I>,
            #[pallet::compact] mut amount_in: Balance,
            #[pallet::compact] min_amount_out: Balance,
            #[pallet::compact] expiry: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // Ensure the length of routes should be >= 1 at least.
            ensure!(!route.is_empty(), Error::<T, I>::EmptyRoute);
            // Ensure user do not input too many routes.
            ensure!(
                route.len() <= T::MaxLengthRoute::get() as usize,
                Error::<T, I>::ExceedMaxLengthRoute
            );

            // Ensure user doesn't input duplicated routes
            let mut _routes = route.clone().into_inner();
            _routes.dedup();
            ensure!(_routes.eq(&*route), Error::<T, I>::DuplicatedRoute);

            // Ensure balances user input is bigger than zero.
            ensure!(
                amount_in > Zero::zero() && min_amount_out >= Zero::zero(),
                Error::<T, I>::ZeroBalance
            );

            // Ensure user iput a valid block number.
            let current_block_num = <frame_system::Pallet<T>>::block_number();
            ensure!(expiry > current_block_num, Error::<T, I>::TooSmallExpiry);

            // Ensure the trader has enough tokens for transaction.
            let (from_currency_id, _) = route[0];
            ensure!(
                <T as Config<I>>::AMMCurrency::balance(from_currency_id, &trader) > amount_in,
                Error::<T, I>::InsufficientBalance
            );

            let original_amount_in = amount_in;
            let mut amount_out: Balance = Zero::zero();
            for sub_route in route.iter() {
                let (from_currency_id, to_currency_id) = sub_route;
                amount_out =
                    T::AMM::trade(&trader, (*from_currency_id, *to_currency_id), amount_in, 1)?;
                amount_in = amount_out;
            }

            ensure!(
                amount_out >= min_amount_out,
                Error::<T, I>::UnexpectedSlippage
            );

            Self::deposit_event(Event::TradedSuccessfully(
                trader,
                original_amount_in,
                route,
                amount_out,
            ));

            Ok(().into())
        }
    }
}
