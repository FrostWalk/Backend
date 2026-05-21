-- Add min_purchases to fairs and enforce one fair per project
ALTER TABLE fairs
    ADD COLUMN min_purchases integer NOT NULL DEFAULT 1,
    ADD CONSTRAINT fairs_project_id_unique UNIQUE (project_id);

-- Add component tracking to transactions and enforce uniqueness
ALTER TABLE transactions
    ADD COLUMN group_deliverable_component_id integer NOT NULL REFERENCES group_deliverable_components ON DELETE CASCADE,
    ADD CONSTRAINT transactions_unique_purchase UNIQUE (buyer_group_id, group_deliverable_selection_id, group_deliverable_component_id);
