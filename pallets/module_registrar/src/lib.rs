#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
// Re-export types but not Error to avoid ambiguity
pub use types::{ModuleId, ModuleInfo, ModuleMetadata, ModuleState, ValidatorStake, ValidatorRequirements, ValidatorWeights, UnbondingInfo, ResourceUsage, ValidationResult};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Zero;
use crate::weights::WeightInfo;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type WeightInfo: weights::WeightInfo;
        type BlockNumber: Member + Parameter + Default + Copy + MaxEncodedLen + From<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn modules)]
    pub type Modules<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ModuleId,
        ModuleInfo<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_stakes)]
    pub type ValidatorStakes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorStake<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn unbonding_info)]
    pub type UnbondingInfos<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        UnbondingInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ModuleRegistered {
            module_id: ModuleId,
            owner: T::AccountId,
        },
        ModuleUpdated {
            module_id: ModuleId,
        },
        ModuleStateChanged {
            module_id: ModuleId,
            new_state: ModuleState,
        },
        ValidatorAdded {
            module_id: ModuleId,
            validator: T::AccountId,
        },
        ValidatorRemoved {
            module_id: ModuleId,
            validator: T::AccountId,
        },
        StakeAdded {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        StakeRemoved {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ModuleNotFound,
        ModuleAlreadyExists,
        InvalidModuleId,
        InvalidModuleState,
        InvalidStakeAmount,
        InsufficientBalance,
        NotAuthorized,
        TooManyValidators,
        ValidatorNotFound,
        ValidatorAlreadyExists,
        UnbondingInProgress,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_module())]
        pub fn register_module(
            origin: OriginFor<T>,
            module_id: ModuleId,
            metadata: ModuleMetadata,
            stake: BalanceOf<T>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            // Validate module ID and check it doesn't exist
            ensure!(!Modules::<T>::contains_key(&module_id), Error::<T>::ModuleAlreadyExists);
            ensure!(!stake.is_zero(), Error::<T>::InvalidStakeAmount);

            // Reserve the stake amount
            T::Currency::reserve(&owner, stake)?;

            // Create new module info
            let module_info = ModuleInfo {
                owner: owner.clone(),
                metadata,
                state: ModuleState::Pending,
                stake,
                validators: BoundedVec::default(),
                trust_score: 0,
            };

            // Store module info
            Modules::<T>::insert(&module_id, module_info);

            // Emit event
            Self::deposit_event(Event::ModuleRegistered { 
                module_id, 
                owner 
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_module())]
        pub fn update_module(
            origin: OriginFor<T>,
            module_id: ModuleId,
            metadata: ModuleMetadata,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            // Get and validate module
            Modules::<T>::try_mutate(&module_id, |maybe_module| -> DispatchResult {
                let module = maybe_module.as_mut().ok_or(Error::<T>::ModuleNotFound)?;
                ensure!(module.owner == caller, Error::<T>::NotAuthorized);

                // Update metadata
                module.metadata = metadata;

                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ModuleUpdated { module_id });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::change_module_state())]
        pub fn change_module_state(
            origin: OriginFor<T>,
            module_id: ModuleId,
            new_state: ModuleState,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            // Get and validate module
            Modules::<T>::try_mutate(&module_id, |maybe_module| -> DispatchResult {
                let module = maybe_module.as_mut().ok_or(Error::<T>::ModuleNotFound)?;
                ensure!(module.owner == caller, Error::<T>::NotAuthorized);

                // Validate state transition
                Self::validate_state_transition(&module.state, &new_state)?;

                // Update state
                module.state = new_state.clone();

                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ModuleStateChanged { 
                module_id, 
                new_state 
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Helper function to validate state transitions
        fn validate_state_transition(
            current_state: &ModuleState,
            new_state: &ModuleState,
        ) -> Result<(), Error<T>> {
            match (current_state, new_state) {
                // Allow transition from Pending to Active or Suspended
                (ModuleState::Pending, ModuleState::Active) |
                (ModuleState::Pending, ModuleState::Suspended) => Ok(()),
                
                // Allow transition from Active to Suspended or Deprecated
                (ModuleState::Active, ModuleState::Suspended) |
                (ModuleState::Active, ModuleState::Deprecated) => Ok(()),
                
                // Allow transition from Suspended to Active or Deprecated
                (ModuleState::Suspended, ModuleState::Active) |
                (ModuleState::Suspended, ModuleState::Deprecated) => Ok(()),
                
                // All other transitions are invalid
                _ => Err(Error::<T>::InvalidModuleState),
            }
        }
    }
}
