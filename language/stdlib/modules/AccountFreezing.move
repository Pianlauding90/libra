address 0x1 {
module AccountFreezing {
    use 0x1::Event::{Self, EventHandle};
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Roles;

    resource struct FreezingBit {
        /// If `is_frozen` is set true, the account cannot be used to send transactions or receive funds
        is_frozen: bool,
    }

    resource struct FreezeEventsHolder {
        freeze_event_handle: EventHandle<FreezeAccountEvent>,
        unfreeze_event_handle: EventHandle<UnfreezeAccountEvent>,
    }

    /// Message for freeze account events
    struct FreezeAccountEvent {
        /// The address that initiated freeze txn
        initiator_address: address,
        /// The address that was frozen
        frozen_address: address,
    }

    /// Message for unfreeze account events
    struct UnfreezeAccountEvent {
        /// The address that initiated unfreeze txn
        initiator_address: address,
        /// The address that was unfrozen
        unfrozen_address: address,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ENOT_ABLE_TO_FREEZE: u64 = 2;
    const ECANNOT_FREEZE_LIBRA_ROOT: u64 = 3;
    const ECANNOT_FREEZE_TC: u64 = 4;
    const ENOT_ABLE_TO_UNFREEZE: u64 = 5;

    public fun initialize(lr_account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(
            Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );
        move_to(lr_account, FreezeEventsHolder {
            freeze_event_handle: Event::new_event_handle(lr_account),
            unfreeze_event_handle: Event::new_event_handle(lr_account),
        });
    }

    public fun create(account: &signer) {
        move_to(account, FreezingBit { is_frozen: false })
    }

    /// Freeze the account at `addr`.
    public fun freeze_account(
        account: &signer,
        frozen_address: address,
    )
    acquires FreezingBit, FreezeEventsHolder {
        assert(Roles::has_treasury_compliance_role(account), ENOT_ABLE_TO_FREEZE);
        let initiator_address = Signer::address_of(account);
        // The libra root account and TC cannot be frozen
        assert(frozen_address != CoreAddresses::LIBRA_ROOT_ADDRESS(), ECANNOT_FREEZE_LIBRA_ROOT);
        assert(frozen_address != CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), ECANNOT_FREEZE_TC);
        borrow_global_mut<FreezingBit>(frozen_address).is_frozen = true;
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address,
                frozen_address
            },
        );
    }
    spec fun freeze_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Unfreeze the account at `addr`.
    public fun unfreeze_account(
        account: &signer,
        unfrozen_address: address,
    )
    acquires FreezingBit, FreezeEventsHolder {
        assert(Roles::has_treasury_compliance_role(account), ENOT_ABLE_TO_UNFREEZE);
        let initiator_address = Signer::address_of(account);
        borrow_global_mut<FreezingBit>(unfrozen_address).is_frozen = false;
        Event::emit_event<UnfreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).unfreeze_event_handle,
            UnfreezeAccountEvent {
                initiator_address,
                unfrozen_address
            },
        );
    }
    spec fun unfreeze_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Returns if the account at `addr` is frozen.
    public fun account_is_frozen(addr: address): bool
    acquires FreezingBit {
        borrow_global<FreezingBit>(addr).is_frozen
     }

    spec module {
        pragma verify = true;
    }
}
}
