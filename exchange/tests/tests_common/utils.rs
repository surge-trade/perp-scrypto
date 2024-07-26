use scrypto_test::prelude::*;
use radix_engine::system::system_db_reader::SystemDatabaseWriter;

pub fn set_time(time: Instant, ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>) {
    let substate_db = ledger.substate_db_mut();
    let substate = ProposerMilliTimestampSubstate {
        epoch_milli: time.seconds_since_unix_epoch * 1000,
    };

    let mut writer = SystemDatabaseWriter::new(substate_db);
    writer
        .write_typed_object_field(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMilliTimestamp.field_index(),
            ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(substate),
        )
        .unwrap();
}

pub fn check_error_msg(err: &RuntimeError, expected: &str) -> bool {
    match err {
        RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) => {
            msg.contains(expected)
        },
        _ => false,
    }
}

pub fn parse_added_nft_ids(
    result: &CommitResult,
    resource: ResourceAddress,
) -> BTreeSet<NonFungibleLocalId> {
    result.vault_balance_changes()
        .into_iter().find_map(|(_, (r, c))| {
            if *r == resource {
                Some(c.clone().added_non_fungibles().clone())
            } else {
                None
            }
        }).unwrap()
}
