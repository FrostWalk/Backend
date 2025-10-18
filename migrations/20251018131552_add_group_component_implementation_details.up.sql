-- Create group_component_implementation_details table
CREATE TABLE group_component_implementation_details (
    id SERIAL PRIMARY KEY,
    group_deliverable_selection_id INTEGER NOT NULL REFERENCES group_deliverable_selections(group_deliverable_selection_id) ON DELETE CASCADE,
    group_deliverable_component_id INTEGER NOT NULL REFERENCES group_deliverable_components(group_deliverable_component_id) ON DELETE CASCADE,
    markdown_description TEXT NOT NULL,
    repository_link TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (group_deliverable_selection_id, group_deliverable_component_id)
);

-- Remove old columns from group_deliverable_selections
ALTER TABLE group_deliverable_selections 
DROP COLUMN link,
DROP COLUMN markdown_text;