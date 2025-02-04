#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod validation;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, LockableCurrency, LockIdentifier, WithdrawReasons},
    };

    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + TypeInfo {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
        type WeightInfo: weights::WeightInfo;
        type BlockNumber: Member + Parameter + Default + Copy + MaxEncodedLen + From<u32>;

        #[pallet::constant]
        type MaxValidatorsPerSet: Get<u32>;

        #[pallet::constant]
        type MaxSlashingEvents: Get<u32>;

        #[pallet::constant]
        type MaxModulesPerValidator: Get<u32>;

        #[pallet::constant]
        type MaxModuleIdLen: Get<u32>;

        #[pallet::constant]
        type MaxModuleGaps: Get<u32>;
    }

    const LOCK_ID: LockIdentifier = *b"modulerg";

    #[pallet::storage]
    pub type ValidatorStake<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        types::ValidatorStake<T::AccountId, BalanceOf<T>>,
        OptionQuery
    >;

    #[pallet::storage]
    pub type ModuleInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        types::ModuleId,
        types::ModuleInfo<T::AccountId, BalanceOf<T>>,
        OptionQuery
    >;

    #[pallet::storage]
    pub type ModuleGaps<T: Config> = StorageValue<
        _,
        BoundedVec<types::ModuleId, T::MaxModuleGaps>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::storage_prefix = "ValidatorPerformance"]
    pub type ValidatorPerformanceStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        validation::ValidatorPerformanceMetrics<T>,
        OptionQuery
    >;

    #[pallet::storage]
    pub type ValidatorScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        validation::ValidationScore,
        ValueQuery
    >;

    #[pallet::storage]
    pub type ValidatorTrustScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery
    >;

    #[pallet::storage]
    pub type SlashingEvents<T: Config> = StorageValue<
        _,
        BoundedVec<types::SlashEvent<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>, T::MaxSlashingEvents>,
        ValueQuery
    >;

    #[pallet::storage]
    pub type ActiveValidatorSet<T: Config> = StorageValue<
        _,
        BoundedVec<T::AccountId, T::MaxValidatorsPerSet>,
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new validator has been registered
        ValidatorRegistered(T::AccountId),
        /// A validator has been slashed
        ValidatorSlashed(T::AccountId, BalanceOf<T>),
        /// The validator set has been rotated
        ValidatorSetRotated(Vec<T::AccountId>),
        /// A new module has been registered
        ModuleRegistered(types::ModuleId, T::AccountId),
        /// A module has been updated
        ModuleUpdated(types::ModuleId),
        /// A module has been removed
        ModuleRemoved(types::ModuleId),
        /// A module's state has changed
        ModuleStateChanged(types::ModuleId, types::ModuleState),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Invalid module state transition
        InvalidModuleState,
        /// Validator not found
        ValidatorNotFound,
        /// Validator already exists
        ValidatorAlreadyExists,
        /// Insufficient stake
        InsufficientStake,
        /// Too many validators
        TooManyValidators,
        /// Too many slashing events
        TooManySlashingEvents,
        /// Not authorized
        NotAuthorized,
        /// Module not found
        ModuleNotFound,
        /// Module already exists
        ModuleAlreadyExists,
        /// Too many modules
        TooManyModules,
        /// Invalid module ID
        InvalidModuleId,
        /// Invalid module metadata
        InvalidMetadata,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(20)]
        #[pallet::weight(10_000)]
        pub fn register_validator(
            origin: OriginFor<T>,
            stake: BalanceOf<T>,
            requirements: types::ValidatorRequirements<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_register_validator(&who, stake, requirements)
        }

        #[pallet::call_index(24)]
        #[pallet::weight(10_000)]
        pub fn register_module(
            origin: OriginFor<T>,
            module_id: types::ModuleId,
            metadata: types::ModuleMetadata,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_register_module(&who, module_id, metadata, stake)
        }

        #[pallet::call_index(25)]
        #[pallet::weight(10_000)]
        pub fn update_module(
            origin: OriginFor<T>,
            module_id: types::ModuleId,
            metadata: types::ModuleMetadata,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_update_module(&who, module_id, metadata)
        }

        #[pallet::call_index(26)]
        #[pallet::weight(10_000)]
        pub fn remove_module(
            origin: OriginFor<T>,
            module_id: types::ModuleId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_remove_module(&who, module_id)
        }

        #[pallet::call_index(27)]
        #[pallet::weight(10_000)]
        pub fn change_module_state(
            origin: OriginFor<T>,
            module_id: types::ModuleId,
            new_state: types::ModuleState,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::transition_module_state(&who, module_id, new_state)
        }

        #[pallet::call_index(21)]
        #[pallet::weight(10_000)]
        pub fn report_validator_performance(
            origin: OriginFor<T>,
            validator: T::AccountId,
            success: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_authorized_reporter(&who), Error::<T>::NotAuthorized);
            Self::update_validator_performance(&validator, success)
        }

        #[pallet::call_index(22)]
        #[pallet::weight(10_000)]
        pub fn slash_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_governance(&who), Error::<T>::NotAuthorized);
            Self::do_slash_validator(&validator, amount)
        }

        #[pallet::call_index(23)]
        #[pallet::weight(10_000)]
        pub fn rotate_validator_set(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_governance(&who), Error::<T>::NotAuthorized);
            
            // Update scores first
            Self::calculate_validator_scores()?;
            
            // Select top validators
            let top_validators = Self::select_top_validators(T::MaxValidatorsPerSet::get());
            
            // Update active set
            ActiveValidatorSet::<T>::try_mutate(|set| {
                *set = BoundedVec::try_from(top_validators.clone())
                    .map_err(|_| Error::<T>::TooManyValidators)?;

                // Emit event
                Self::deposit_event(Event::ValidatorSetRotated(top_validators));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Check if an account is authorized to report validator performance
        pub(crate) fn is_authorized_reporter(_who: &T::AccountId) -> bool {
            // TODO: Implement proper authorization check
            true
        }

        /// Check if an account has governance privileges
        pub(crate) fn is_governance(_who: &T::AccountId) -> bool {
            // TODO: Implement proper governance check
            true
        }

        /// Register a new module
        pub fn do_register_module(
            who: &T::AccountId,
            module_id: types::ModuleId,
            metadata: types::ModuleMetadata,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            // Ensure module doesn't already exist
            ensure!(!ModuleInfo::<T>::contains_key(&module_id), Error::<T>::ModuleAlreadyExists);

            // Validate module ID length
            ensure!(module_id.len() <= T::MaxModuleIdLen::get() as usize, Error::<T>::InvalidModuleId);

            // Create module info
            let module_info = types::ModuleInfo {
                owner: who.clone(),
                metadata: metadata.clone(),
                state: types::ModuleState::Pending,
                stake,
                validators: BoundedVec::default(),
                trust_score: 0,
            };

            // Store module info
            ModuleInfo::<T>::insert(&module_id, module_info);

            // Lock stake
            T::Currency::set_lock(
                LOCK_ID,
                who,
                stake,
                WithdrawReasons::all(),
            );

            // Initialize module gaps
            ModuleGaps::<T>::mutate(|gaps| {
                if let Some(pos) = gaps.iter().position(|id| id == &module_id) {
                    gaps.remove(pos);
                }
            });

            // Emit event
            Self::deposit_event(Event::ModuleRegistered(module_id.clone(), who.clone()));

            Ok(())
        }

        /// Update a module's metadata
        pub fn do_update_module(
            who: &T::AccountId,
            module_id: types::ModuleId,
            metadata: types::ModuleMetadata,
        ) -> DispatchResult {
            ModuleInfo::<T>::try_mutate(&module_id, |maybe_info| -> DispatchResult {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ModuleNotFound)?;
                ensure!(info.owner == *who, Error::<T>::NotAuthorized);
                
                info.metadata = metadata;
                Self::deposit_event(Event::ModuleUpdated(module_id.clone()));
                Ok(())
            })
        }

        /// Remove a module
        pub fn do_remove_module(
            who: &T::AccountId,
            module_id: types::ModuleId,
        ) -> DispatchResult {
            let info = ModuleInfo::<T>::get(&module_id).ok_or(Error::<T>::ModuleNotFound)?;
            ensure!(info.owner == *who, Error::<T>::NotAuthorized);

            // Return stake to owner
            T::Currency::remove_lock(LOCK_ID, who);

            // Clean up storage
            ModuleInfo::<T>::remove(&module_id);
            ModuleGaps::<T>::try_mutate(|gaps| -> DispatchResult {
                gaps.try_push(module_id.clone())
                    .map_err(|_| Error::<T>::TooManyModules)?;
                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ModuleRemoved(module_id));

            Ok(())
        }

        /// Change module state
        pub fn transition_module_state(
            who: &T::AccountId,
            module_id: types::ModuleId,
            new_state: types::ModuleState,
        ) -> DispatchResult {
            ModuleInfo::<T>::try_mutate(&module_id, |maybe_info| -> DispatchResult {
                let info = maybe_info.as_mut().ok_or(Error::<T>::ModuleNotFound)?;
                ensure!(info.owner == *who || Self::is_governance(who), Error::<T>::NotAuthorized);

                // Validate state transition
                match (info.state.clone(), new_state.clone()) {
                    (types::ModuleState::Pending, types::ModuleState::Active) => {},
                    (types::ModuleState::Active, types::ModuleState::Suspended) => {},
                    (types::ModuleState::Suspended, types::ModuleState::Active) => {},
                    (_, types::ModuleState::Deprecated) => {},
                    _ => return Err(Error::<T>::InvalidModuleState.into()),
                }

                info.state = new_state.clone();
                Self::deposit_event(Event::ModuleStateChanged(module_id.clone(), new_state));
                Ok(())
            })
        }
    }
}
