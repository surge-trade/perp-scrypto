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