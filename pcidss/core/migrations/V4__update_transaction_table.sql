ALTER TABLE bank_transaction ADD COLUMN on_chain_id VARCHAR(64);
CREATE UNIQUE INDEX on_chain_id_unique_index ON bank_transaction (on_chain_id) WHERE on_chain_id IS NOT NULL;