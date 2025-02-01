# Staking Component Specification

## Purpose
Manages the staking system for validators and delegators, including stake management, rewards distribution, and slashing mechanisms.

## Dependencies
```toml
[dependencies]
frame_support = { workspace = true }
frame_system = { workspace = true }
sp-std = { workspace = true }
sp-runtime = { workspace = true }
parity-scale-codec = { workspace = true }
scale-info = { workspace = true }
crate::types = { path = "../types" }
crate::events = { path = "../events" }
crate::validation = { path = "../validation" }
```

## Key Structures
```rust
pub struct ValidatorStake<AccountId, Balance> {
    validator: AccountId,
    total_stake: Balance,
    self_stake: Balance,
    delegated_stake: Balance,
    delegators: BoundedVec<(AccountId, Balance), MaxDelegatorsPerValidator>,
    commission_rate: Percent,
}

pub struct UnbondingInfo<AccountId, Balance, BlockNumber> {
    delegator: AccountId,
    validator: AccountId,
    amount: Balance,
    started_at: BlockNumber,
    completion_block: BlockNumber,
}
```

## Storage Items
```rust
#[pallet::storage]
pub type ValidatorStakes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorStake<T::AccountId, BalanceOf<T>>>;

#[pallet::storage]
pub type UnbondingRequests<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,
    Blake2_128Concat, T::AccountId,
    UnbondingInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>
>;

#[pallet::storage]
pub type DelegatorHistory<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(T::AccountId, T::BlockNumber)>>;
```

## Key Functions
1. `bond`: Stake tokens as a validator
2. `delegate`: Delegate tokens to a validator
3. `unbond`: Start unbonding process
4. `withdraw_unbonded`: Complete unbonding
5. `claim_rewards`: Claim staking rewards
6. `slash`: Apply slashing penalties

## Events
- Bonded
- Delegated
- UnbondingStarted
- UnbondingCompleted
- RewardsClaimed
- Slashed

## Error Handling
- InsufficientBalance
- AlreadyDelegating
- NotDelegating
- TooManyDelegators
- InvalidCommissionRate
- UnbondingInProgress

## Integration Points
1. Validation system for validator management
2. Core module for stake requirements
3. Event system for status updates
4. Governance for parameter management

## Economic Parameters
1. Validator Requirements
   - Minimum self-stake: 5000 COM
   - Commission rate range: 5% - 50%
   - Unlimited delegations allowed

2. Delegator Rules
   - Single validator per delegator
   - 7-day unbonding period
   - No rewards during unbonding
   - Subject to slashing during unbonding

3. Reward Distribution
   - Based on total stake
   - Commission applied to rewards
   - Daily distribution cycle
   - Performance multipliers
