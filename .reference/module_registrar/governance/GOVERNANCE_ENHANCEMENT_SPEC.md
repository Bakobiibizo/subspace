# Governance Enhancement Specification

## Overview
This specification outlines the enhancements to the governance pallet to support the module registrar system and improved validator security measures. The design leverages the existing whitelist functionality for module registration while adding new capabilities for validator oversight.

## Core Components

### 1. Module Registration via Whitelist

#### Integration Points
```rust
// Existing whitelist functionality to be used:
pub fn add_to_whitelist(origin: OriginFor<T>, module_key: T::AccountId)
pub fn remove_from_whitelist(origin: OriginFor<T>, module_key: T::AccountId)
pub fn add_dao_application(origin: OriginFor<T>, application_key: T::AccountId, data: Vec<u8>)
```

#### Enhanced Module Application
```rust
pub struct ModuleApplication {
    // Existing CuratorApplication fields
    module_type: ModuleType,
    resource_requirements: ResourceRequirements,
    performance_guarantees: PerformanceGuarantees,
    security_deposit: Balance,
}

pub enum ModuleType {
    Inference,
    Storage,
    Validation,
    Custom(Vec<u8>),
}
```

### 2. Validator Control Mechanisms

#### Emission Control
```rust
pub struct EmissionAdjustment {
    validator: AccountId,
    adjustment_type: AdjustmentType,
    reason: Vec<u8>,
    duration: BlockNumber,
}

pub enum AdjustmentType {
    Percentage(Percent),  // Adjust by percentage
    Fixed(Balance),       // Fixed amount adjustment
    Suspension,           // Temporary suspension
    Permanent,           // Permanent suspension
}

// New extrinsics
impl<T: Config> Pallet<T> {
    pub fn propose_emission_adjustment(
        origin: OriginFor<T>,
        adjustment: EmissionAdjustment,
    ) -> DispatchResult;
    
    pub fn execute_emission_adjustment(
        origin: OriginFor<T>,
        proposal_id: ProposalId,
    ) -> DispatchResult;
}
```

#### Slashing Mechanism
```rust
pub struct SlashingProposal {
    target: AccountId,
    severity: SlashingSeverity,
    evidence: Vec<BehaviorProof>,
    recovery_conditions: Option<RecoveryPath>,
}

pub enum SlashingSeverity {
    Warning { penalty: Percent },      // 0-1%
    Moderate { penalty: Percent },     // 1-10%
    Severe { penalty: Percent },       // 10-50%
    Terminal { penalty: Percent },     // 50-100%
}

// New extrinsics
impl<T: Config> Pallet<T> {
    pub fn propose_validator_slash(
        origin: OriginFor<T>,
        proposal: SlashingProposal,
    ) -> DispatchResult;
    
    pub fn execute_slash(
        origin: OriginFor<T>,
        proposal_id: ProposalId,
    ) -> DispatchResult;
}
```

### 3. Behavioral Monitoring

#### Performance Metrics
```rust
pub struct ValidatorMetrics {
    module_performance: BTreeMap<ModuleId, ModuleMetrics>,
    network_behavior: NetworkMetrics,
    economic_activity: EconomicMetrics,
}

pub struct ModuleMetrics {
    success_rate: Percent,
    response_time: u32,
    resource_usage: ResourceMetrics,
    user_feedback: Vec<Feedback>,
}

pub struct NetworkMetrics {
    peer_connections: Vec<(AccountId, ConnectionStrength)>,
    message_patterns: MessagePatternAnalysis,
    voting_correlation: BTreeMap<AccountId, Percent>,
}

// New extrinsics
impl<T: Config> Pallet<T> {
    pub fn submit_validator_metrics(
        origin: OriginFor<T>,
        metrics: ValidatorMetrics,
    ) -> DispatchResult;
    
    pub fn flag_suspicious_behavior(
        origin: OriginFor<T>,
        target: AccountId,
        evidence: Vec<BehaviorProof>,
    ) -> DispatchResult;
}
```

### 4. Emergency Powers

```rust
pub enum EmergencyAction {
    FreezeValidator {
        target: AccountId,
        duration: BlockNumber,
    },
    PauseModule {
        module_id: ModuleId,
        duration: BlockNumber,
    },
    EmergencyShutdown {
        scope: ShutdownScope,
        duration: BlockNumber,
    },
}

pub enum ShutdownScope {
    Validator(AccountId),
    Module(ModuleId),
    Network,
}

// New extrinsics
impl<T: Config> Pallet<T> {
    pub fn execute_emergency_action(
        origin: OriginFor<T>,
        action: EmergencyAction,
    ) -> DispatchResult;
}
```

## Storage Additions

```rust
#[pallet::storage]
pub type ValidatorMetricsHistory<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<ValidatorMetrics<T>, T::MaxMetricsHistory>,
    ValueQuery,
>;

#[pallet::storage]
pub type EmissionAdjustments<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Vec<(EmissionAdjustment<T>, BlockNumber)>,
    ValueQuery,
>;

#[pallet::storage]
pub type BehaviorFlags<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Vec<(BehaviorProof, BlockNumber)>,
    ValueQuery,
>;
```

## Events
```rust
#[pallet::event]
pub enum Event<T: Config> {
    // Module Registration
    ModuleApplicationSubmitted(T::AccountId, ModuleType),
    ModuleWhitelisted(T::AccountId),
    ModuleRemoved(T::AccountId),
    
    // Validator Control
    EmissionAdjustmentProposed(ProposalId, T::AccountId),
    EmissionAdjustmentExecuted(T::AccountId, AdjustmentType),
    SlashingProposed(ProposalId, T::AccountId),
    SlashingExecuted(T::AccountId, SlashingSeverity),
    
    // Monitoring
    SuspiciousBehaviorFlagged(T::AccountId, Vec<BehaviorProof>),
    EmergencyActionExecuted(EmergencyAction),
}
```
