# Events Component Specification

## Purpose
Defines and manages all events emitted by the Module Registrar Pallet, providing a standardized way to notify external systems of state changes.

## Dependencies
```toml
[dependencies]
frame_support = { workspace = true }
frame_system = { workspace = true }
sp-std = { workspace = true }
sp-runtime = { workspace = true }
crate::types = { path = "../types" }
```

## Event Definitions
```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    // Module Management
    ModuleRegistered {
        module_id: ModuleId,
        owner: T::AccountId,
        stake: BalanceOf<T>,
    },
    ModuleUpdated {
        module_id: ModuleId,
        metadata: ModuleMetadata,
    },
    ModuleStateChanged {
        module_id: ModuleId,
        old_state: ModuleState,
        new_state: ModuleState,
    },

    // Validation Events
    ValidationStarted {
        module_id: ModuleId,
        validator: T::AccountId,
    },
    ValidationCompleted {
        module_id: ModuleId,
        validator: T::AccountId,
        result: ValidationResult,
    },
    ValidationFailed {
        module_id: ModuleId,
        validator: T::AccountId,
        error: ValidationError,
    },

    // Staking Events
    StakeAdded {
        staker: T::AccountId,
        module_id: ModuleId,
        amount: BalanceOf<T>,
    },
    StakeRemoved {
        staker: T::AccountId,
        module_id: ModuleId,
        amount: BalanceOf<T>,
    },
    DelegationChanged {
        delegator: T::AccountId,
        validator: T::AccountId,
        amount: BalanceOf<T>,
    },
    UnbondingStarted {
        delegator: T::AccountId,
        validator: T::AccountId,
        amount: BalanceOf<T>,
        completion_block: T::BlockNumber,
    },

    // Validator Events
    ValidatorAdded {
        validator: T::AccountId,
        stake: BalanceOf<T>,
    },
    ValidatorRemoved {
        validator: T::AccountId,
    },
    ValidatorSlashed {
        validator: T::AccountId,
        amount: BalanceOf<T>,
        reason: SlashReason,
    },

    // IPFS Events
    ContentStored {
        hash: H256,
        size: u64,
        content_type: ContentType,
    },
    ContentRetrieved {
        hash: H256,
        requester: T::AccountId,
    },
    ContentRemoved {
        hash: H256,
        reason: RemovalReason,
    },
}
```

## Event Metadata
```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum SlashReason {
    InvalidValidation,
    MissedValidation,
    MaliciousBehavior,
    GovernanceDecision,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum RemovalReason {
    Expired,
    Deprecated,
    GarbageCollection,
    GovernanceDecision,
}
```

## Event Handling
1. Event Emission
   - Proper type conversion
   - Consistent parameter ordering
   - Clear documentation

2. Event Filtering
   - Module-specific filters
   - Account-specific filters
   - Time-based filters

3. Event Storage
   - System event queue
   - Historical event access
   - Event pruning

## Integration Points
1. Core Module
   - Module lifecycle events
   - State change notifications

2. Validation System
   - Validation process events
   - Result notifications

3. Staking System
   - Stake management events
   - Delegation updates

4. IPFS System
   - Content management events
   - Storage operations

## Usage Guidelines
1. Event Documentation
   - Clear descriptions
   - Parameter explanations
   - Example usage

2. Event Best Practices
   - Atomic events
   - Consistent naming
   - Complete information
   - Proper ordering

3. Performance Considerations
   - Event size optimization
   - Storage impact
   - Indexing requirements
