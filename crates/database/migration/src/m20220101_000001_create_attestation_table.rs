use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Attestation {
    Table,
    Index,
    Root,
    TxHash,
    BlockNumber,
    BlockTimestamp,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Attestation::Table)
                    .if_not_exists()
                    .col(pk_auto(Attestation::Index))
                    .col(binary_len(Attestation::Root, 32).not_null().unique_key())
                    .col(binary_len(Attestation::TxHash, 32).not_null().unique_key())
                    .col(unsigned_null(Attestation::BlockNumber))
                    .col(unsigned_null(Attestation::BlockTimestamp))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attestation_block_number")
                    .table(Attestation::Table)
                    .col(Attestation::BlockNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attestation_block_timestamp")
                    .table(Attestation::Table)
                    .col(Attestation::BlockTimestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Attestation::Table).to_owned())
            .await
    }
}
