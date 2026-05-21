alter table student_uploads
drop constraint if exists student_uploads_selection_unique;

alter table student_uploads
drop column if exists upload_count;

alter table projects
drop column if exists upload_deadline;
