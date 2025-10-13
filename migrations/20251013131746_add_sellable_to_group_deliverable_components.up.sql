-- Add sellable column to group_deliverable_components table
ALTER TABLE group_deliverable_components
ADD COLUMN sellable BOOLEAN NOT NULL DEFAULT TRUE;
