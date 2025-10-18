-- Add back old columns to group_deliverable_selections
ALTER TABLE group_deliverable_selections 
ADD COLUMN link TEXT,
ADD COLUMN markdown_text TEXT;

-- Drop group_component_implementation_details table
DROP TABLE group_component_implementation_details;