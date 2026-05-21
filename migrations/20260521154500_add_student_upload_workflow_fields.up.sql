alter table projects
add column upload_deadline timestamptz;

alter table student_uploads
add column upload_count integer not null default 0;

alter table student_uploads
add constraint student_uploads_selection_unique unique (student_deliverable_selection_id);
