#![cfg_attr(not(feature="std"), no_std)]

/// a module for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        (T::AccountId, T::BlockNumber)
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransfered(T::AccountId, T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

            Proofs::<T>::insert(
                &claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number())
            );

            Self::deposit_event(Event::ClaimCreated(sender, claim));
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            ensure!(owner == sender, Error::<T>::NotClaimOwner);

            Proofs::<T>::remove(&claim);

            Self::deposit_event(Event::ClaimRevoked(sender, claim));
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            claim: Vec<u8>,
            receiver: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

            let (owner, block_num) = Proofs::<T>::get(&claim);
            ensure!(owner == sender, Error::<T>::NotClaimOwner);

            Proofs::<T>::mutate(&claim, |val| *val = (receiver.clone(), block_num));

            Self::deposit_event(Event::ClaimTransfered(sender, receiver, claim));

            Ok(().into())
        }
    }
}