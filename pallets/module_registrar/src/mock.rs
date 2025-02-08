#![cfg(test)]

use crate as pallet_module_registrar;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64, Currency, ExistenceRequirement, WithdrawReasons, SignedImbalance, ConstBool, LockableCurrency},
    PalletId,
};
use sp_runtime::DispatchError;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use pallet_governance::{self, GovernanceApi, GovernanceConfiguration};
use pallet_subspace;
use pallet_subnet_emission_api::{SubnetEmissionApi, SubnetConsensus};

type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
    pub const MaxValidatorsPerSet: u32 = 100;
    pub const MaxSlashingEvents: u32 = 1000;
    pub const MaxModulesPerValidator: u32 = 10;
    pub const MaxModuleIdLen: u32 = 32;
    pub const MaxModuleGaps: u32 = 5;
}

// Mock implementations for required traits
impl GovernanceApi<u64> for Test {
    fn get_dao_treasury_address() -> u64 {
        1
    }

    fn get_global_governance_configuration() -> GovernanceConfiguration {
        GovernanceConfiguration::default()
    }

    fn get_subnet_governance_configuration(_subnet_id: u16) -> GovernanceConfiguration {
        GovernanceConfiguration::default()
    }

    fn update_global_governance_configuration(_config: GovernanceConfiguration) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }

    fn update_subnet_governance_configuration(_subnet_id: u16, _config: GovernanceConfiguration) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }

    fn is_delegating_voting_power(_account_id: &u64) -> bool {
        false
    }

    fn update_delegating_voting_power(_account_id: &u64, _delegating: bool) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }

    fn execute_application(_account_id: &u64) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }

    fn get_general_subnet_application_cost() -> u64 {
        1000
    }

    fn curator_application_exists(_account_id: &u64) -> bool {
        false
    }

    fn whitelisted_keys() -> BTreeSet<u64> {
        BTreeSet::new()
    }

    fn get_curator() -> u64 {
        1
    }

    fn set_curator(_account_id: &u64) {
    }

    fn set_general_subnet_application_cost(_cost: u64) {
    }

    fn clear_subnet_includes(_subnet_id: u16) {
    }
}

impl SubnetEmissionApi<u64> for Test {
    fn set_subnet_emission_storage(_netuid: u16, _emission: u64) {
    }

    fn get_lowest_emission_netuid(_is_mineable: bool) -> Option<u16> {
        Some(0)
    }

    fn create_yuma_subnet(_netuid: u16) {
    }

    fn can_remove_subnet(_netuid: u16) -> bool {
        true
    }

    fn is_mineable_subnet(_netuid: u16) -> bool {
        true
    }

    fn get_consensus_netuid(_consensus_type: SubnetConsensus) -> Option<u16> {
        Some(0)
    }

    fn get_subnet_consensus_type(_netuid: u16) -> Option<SubnetConsensus> {
        Some(SubnetConsensus::default())
    }

    fn set_subnet_consensus_type(_netuid: u16, _consensus_type: Option<SubnetConsensus>) {
    }

    fn get_weights(_netuid: u16, _module_id: u16) -> Option<Vec<(u16, u16)>> {
        Some(vec![])
    }

    fn set_weights(_netuid: u16, _module_id: u16, _weights: Option<Vec<(u16, u16)>>) -> Option<Vec<(u16, u16)>> {
        Some(vec![])
    }

    fn clear_subnet_includes(_netuid: u16) {
    }

    fn clear_module_includes(_netuid: u16, _module_id: u16, _module_idx: u16, _who: &u64, _slash_who: &u64) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }
}

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Governance: pallet_governance::{Pallet, Call, Storage, Event<T>},
        Subspace: pallet_subspace::{Pallet, Call, Storage, Event<T>},
        ModuleRegistrar: pallet_module_registrar::{Pallet, Call, Storage, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const SubspacePalletId: PalletId = PalletId(*b"subspace");
    pub const GovernancePalletId: PalletId = PalletId(*b"govrnace");
    pub const ExistentialDeposit: u64 = 1;
}

pub struct MockCurrency;

impl Currency<u64> for MockCurrency {
    type Balance = u64;
    type PositiveImbalance = ();
    type NegativeImbalance = ();

    fn total_balance(_who: &u64) -> Self::Balance {
        0
    }

    fn can_slash(_who: &u64, _value: Self::Balance) -> bool {
        true
    }

    fn total_issuance() -> Self::Balance {
        0
    }

    fn minimum_balance() -> Self::Balance {
        1
    }

    fn burn(_amount: Self::Balance) -> Self::PositiveImbalance {
        ()
    }

    fn issue(_amount: Self::Balance) -> Self::NegativeImbalance {
        ()
    }

    fn free_balance(_who: &u64) -> Self::Balance {
        0
    }

    fn ensure_can_withdraw(
        _who: &u64,
        _amount: Self::Balance,
        _reasons: WithdrawReasons,
        _new_balance: Self::Balance,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }

    fn transfer(
        _source: &u64,
        _dest: &u64,
        _value: Self::Balance,
        _existence_requirement: ExistenceRequirement,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }

    fn slash(_who: &u64, _value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        ((), 0)
    }

    fn deposit_into_existing(
        _who: &u64,
        _value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, DispatchError> {
        Ok(())
    }

    fn deposit_creating(_who: &u64, _value: Self::Balance) -> Self::PositiveImbalance {
        ()
    }

    fn withdraw(
        _who: &u64,
        _value: Self::Balance,
        _reasons: WithdrawReasons,
        _liveness: ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, DispatchError> {
        Ok(())
    }

    fn make_free_balance_be(
        _who: &u64,
        _balance: Self::Balance,
    ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        SignedImbalance::Positive(())
    }
}

impl frame_support::traits::LockableCurrency<u64> for MockCurrency {
    type Moment = u64;
    type MaxLocks = ConstU32<50>;

    fn set_lock(
        _id: frame_support::traits::LockIdentifier,
        _who: &u64,
        _amount: Self::Balance,
        _reasons: frame_support::traits::WithdrawReasons,
    ) {
        // Mock implementation
    }

    fn extend_lock(
        _id: frame_support::traits::LockIdentifier,
        _who: &u64,
        _amount: Self::Balance,
        _reasons: frame_support::traits::WithdrawReasons,
    ) {
        // Mock implementation
    }

    fn remove_lock(
        _id: frame_support::traits::LockIdentifier,
        _who: &u64,
    ) {
        // Mock implementation
    }
}



impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<50>;
    type RuntimeFreezeReason = ();
    type RuntimeHoldReason = ();
}



impl pallet_subspace::Config for Test {
    type PalletId = SubspacePalletId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type DefaultMaxRegistrationsPerInterval = ConstU16<10>;
    type DefaultMaxSubnetRegistrationsPerInterval = ConstU16<5>;
    type DefaultModuleMinBurn = ConstU64<1000>;
    type DefaultSubnetMinBurn = ConstU64<5000>;
    type DefaultMinValidatorStake = ConstU64<10000>;
    type WeightInfo = ();
    type EnforceWhitelist = ConstBool<false>;
    type DefaultUseWeightsEncryption = ConstBool<false>;
}



pub struct BalancesWrapper<T>(sp_std::marker::PhantomData<T>);

impl frame_support::traits::LockableCurrency<u64> for BalancesWrapper<Test> {
    type Moment = u64;
    type MaxLocks = ConstU32<50>;

    fn set_lock(
        id: frame_support::traits::LockIdentifier,
        who: &u64,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
    ) {
        <Balances as LockableCurrency<u64>>::set_lock(id, who, amount, reasons)
    }

    fn extend_lock(
        id: frame_support::traits::LockIdentifier,
        who: &u64,
        amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
    ) {
        <Balances as LockableCurrency<u64>>::extend_lock(id, who, amount, reasons)
    }

    fn remove_lock(id: frame_support::traits::LockIdentifier, who: &u64) {
        <Balances as LockableCurrency<u64>>::remove_lock(id, who)
    }
}

impl Currency<u64> for BalancesWrapper<Test> {
    type Balance = u64;
    type PositiveImbalance = ();
    type NegativeImbalance = ();

    fn total_balance(who: &u64) -> Self::Balance {
        Balances::total_balance(who)
    }

    fn can_slash(who: &u64, value: Self::Balance) -> bool {
        Balances::can_slash(who, value)
    }

    fn total_issuance() -> Self::Balance {
        Balances::total_issuance()
    }

    fn minimum_balance() -> Self::Balance {
        1
    }

    fn burn(_amount: Self::Balance) -> Self::PositiveImbalance {
        ()
    }

    fn issue(_amount: Self::Balance) -> Self::NegativeImbalance {
        ()
    }

    fn free_balance(who: &u64) -> Self::Balance {
        Balances::free_balance(who)
    }

    fn ensure_can_withdraw(
        who: &u64,
        amount: Self::Balance,
        reasons: WithdrawReasons,
        new_balance: Self::Balance,
    ) -> frame_support::dispatch::DispatchResult {
        Balances::ensure_can_withdraw(who, amount, reasons, new_balance)
    }

    fn transfer(
        source: &u64,
        dest: &u64,
        value: Self::Balance,
        existence_requirement: ExistenceRequirement,
    ) -> frame_support::dispatch::DispatchResult {
        <Balances as Currency<u64>>::transfer(source, dest, value, existence_requirement)
    }

    fn slash(who: &u64, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        let (_, amount) = Balances::slash(who, value);
        ((), amount)
    }

    fn deposit_into_existing(
        who: &u64,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, DispatchError> {
        let _ = Balances::deposit_into_existing(who, value);
        Ok(())
    }

    fn deposit_creating(who: &u64, value: Self::Balance) -> Self::PositiveImbalance {
        let _ = Balances::deposit_creating(who, value);
        ()
    }

    fn withdraw(
        who: &u64,
        value: Self::Balance,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, DispatchError> {
        Balances::withdraw(who, value, reasons, liveness).map(|_| ())
    }

    fn make_free_balance_be(
        who: &u64,
        balance: Self::Balance,
    ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        let _ = <Balances as Currency<u64>>::make_free_balance_be(who, balance);
        SignedImbalance::Positive(())
    }
}

impl pallet_governance::Config for Test {
    type PalletId = GovernancePalletId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = BalancesWrapper<Test>;
    type WeightInfo = ();
}

impl pallet_module_registrar::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxValidatorsPerSet = MaxValidatorsPerSet;
    type MaxSlashingEvents = MaxSlashingEvents;
    type MaxModulesPerValidator = MaxModulesPerValidator;
    type MaxModuleIdLen = MaxModuleIdLen;
    type MaxModuleGaps = MaxModuleGaps;
    type Currency = BalancesWrapper<Test>;
    type BlockNumber = u32;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    t.into()
}

pub fn get_origin(account: u64) -> RuntimeOrigin {
    RuntimeOrigin::signed(account)
}
