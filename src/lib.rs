pub use codec::Encode;
pub use frame_support::{
    assert_ok, instances::Instance1, pallet_prelude::Weight, sp_io, sp_tracing,
    traits::fungibles::Inspect,
};
pub use integration_tests_common::{
    constants::{
        accounts::{ALICE, BOB},
        kusama::ED as KUSAMA_ED,
        penpal, statemine, PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD, XCM_V3,
    },
    AccountId, Kusama, KusamaPallet, KusamaReceiver, KusamaSender, PenpalKusamaReceiver,
    PenpalKusamaSender, StatemineReceiver, StatemineSender,
};
pub use sp_core::{sr25519, storage::Storage, Get};

pub use parachains_common::{AuraId, Balance, BlockNumber, StatemintAuraId};
pub use polkadot_core_primitives::InboundDownwardMessage;
pub use xcm::{
    prelude::*,
    v3::{Error, NetworkId::Kusama as KusamaId},
};
use xcm_emulator::{
    assert_expected_events, bx, cumulus_pallet_dmp_queue, decl_test_networks, decl_test_parachains,
    helpers::weight_within_threshold, NetworkComponent, Parachain, RelayChain, TestExt,
};

decl_test_parachains! {
    // Kusama
    pub struct Statemine {
        genesis = statemine::genesis(),
        on_init = (),
        runtime = {
            Runtime: statemine_runtime::Runtime,
            RuntimeOrigin: statemine_runtime::RuntimeOrigin,
            RuntimeCall: statemine_runtime::RuntimeCall,
            RuntimeEvent: statemine_runtime::RuntimeEvent,
            XcmpMessageHandler: statemine_runtime::XcmpQueue,
            DmpMessageHandler: statemine_runtime::DmpQueue,
            LocationToAccountId: statemine_runtime::xcm_config::LocationToAccountId,
            System: statemine_runtime::System,
            Balances: statemine_runtime::Balances,
            ParachainSystem: statemine_runtime::ParachainSystem,
            ParachainInfo: statemine_runtime::ParachainInfo,
        },
        pallets_extra = {
            PolkadotXcm: statemine_runtime::PolkadotXcm,
            Assets: statemine_runtime::Assets,
            ForeignAssets: statemine_runtime::Assets,
        }
    },
    pub struct PenpalKusama {
        genesis = penpal::genesis(penpal::PARA_ID),
        on_init = (),
        runtime = {
            Runtime: penpal_runtime::Runtime,
            RuntimeOrigin: penpal_runtime::RuntimeOrigin,
            RuntimeCall: penpal_runtime::RuntimeCall,
            RuntimeEvent: penpal_runtime::RuntimeEvent,
            XcmpMessageHandler: penpal_runtime::XcmpQueue,
            DmpMessageHandler: penpal_runtime::DmpQueue,
            LocationToAccountId: penpal_runtime::xcm_config::LocationToAccountId,
            System: penpal_runtime::System,
            Balances: penpal_runtime::Balances,
            ParachainSystem: penpal_runtime::ParachainSystem,
            ParachainInfo: penpal_runtime::ParachainInfo,
        },
        pallets_extra = {
            PolkadotXcm: penpal_runtime::PolkadotXcm,
            Assets: penpal_runtime::Assets,
        }
    }
}

decl_test_networks! {
    pub struct KusamaMockNet {
        relay_chain = Kusama,
        parachains = vec![
            Statemine,
            PenpalKusama,
        ],
    }
}

#[cfg(test)]
mod tests;
