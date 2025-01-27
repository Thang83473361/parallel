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

//! Mocks for the prices module.

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, FixedPointNumber};

pub type AccountId = u128;
pub type BlockNumber = u64;

mod prices {
    pub use super::super::*;
}

pub const DOT: AssetId = 10;
#[allow(non_upper_case_globals)]
pub const xDOT: AssetId = 11;
pub const KSM: AssetId = 20;
#[allow(non_upper_case_globals)]
pub const xKSM: AssetId = 21;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, Moment>;
pub struct MockDataProvider;
impl DataProvider<AssetId, TimeStampedPrice> for MockDataProvider {
    fn get(asset_id: &AssetId) -> Option<TimeStampedPrice> {
        match *asset_id {
            DOT => Some(TimeStampedPrice {
                value: Price::saturating_from_integer(100),
                timestamp: 0,
            }),
            KSM => Some(TimeStampedPrice {
                value: Price::saturating_from_integer(500),
                timestamp: 0,
            }),
            _ => None,
        }
    }
}

impl DataProviderExtended<AssetId, TimeStampedPrice> for MockDataProvider {
    fn get_no_op(_key: &AssetId) -> Option<TimeStampedPrice> {
        None
    }

    fn get_all_values() -> Vec<(AssetId, Option<TimeStampedPrice>)> {
        vec![]
    }
}

pub struct LiquidStakingExchangeRateProvider;
impl ExchangeRateProvider for LiquidStakingExchangeRateProvider {
    fn get_exchange_rate() -> Rate {
        Rate::saturating_from_rational(150, 100)
    }
}

ord_parameter_types! {
    pub const One: AccountId = 1;
    pub const StakingCurrency: AssetId = KSM;
    pub const LiquidCurrency: AssetId = xKSM;
}
pub struct Decimal;
#[allow(non_upper_case_globals)]
impl DecimalProvider for Decimal {
    fn get_decimal(asset_id: &AssetId) -> u8 {
        match *asset_id {
            DOT | xDOT => 10,
            KSM | xKSM => 12,
            _ => 0,
        }
    }
}

pub struct LiquidStaking;
impl LiquidStakingCurrenciesProvider<AssetId> for LiquidStaking {
    fn get_staking_currency() -> Option<AssetId> {
        Some(KSM)
    }
    fn get_liquid_currency() -> Option<AssetId> {
        Some(xKSM)
    }
}

impl Config for Runtime {
    type Event = Event;
    type Source = MockDataProvider;
    type FeederOrigin = EnsureSignedBy<One, AccountId>;
    type LiquidStakingCurrenciesProvider = LiquidStaking;
    type LiquidStakingExchangeRateProvider = LiquidStakingExchangeRateProvider;
    type Decimal = Decimal;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Prices: prices::{Pallet, Storage, Call, Event<T>},
    }
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        t.into()
    }
}
