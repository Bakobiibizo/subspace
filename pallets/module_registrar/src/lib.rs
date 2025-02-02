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
        traits::{Currency, LockableCurrency},
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
    }

    #[pallet::storage]
    pub type ValidatorStake<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        types::ValidatorStake<T::AccountId, BalanceOf<T>>,
        OptionQuery
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
    }
}
