ALTER TABLE transactions
    DROP CONSTRAINT IF EXISTS transactions_unique_purchase,
    DROP COLUMN IF EXISTS group_deliverable_component_id;

ALTER TABLE fairs
    DROP CONSTRAINT IF EXISTS fairs_project_id_unique,
    DROP COLUMN IF EXISTS min_purchases;
