ALTER TABLE projects ADD COLUMN oral_exam_enabled BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE oral_exam_notes (
    note_id SERIAL PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES students(student_id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    note_text TEXT NOT NULL DEFAULT '',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by_admin_id INTEGER REFERENCES admins(admin_id) ON DELETE SET NULL,
    UNIQUE (student_id, project_id)
);

CREATE TABLE oral_exam_completions (
    completion_id SERIAL PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES students(student_id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_by_admin_id INTEGER REFERENCES admins(admin_id) ON DELETE SET NULL,
    UNIQUE (student_id, project_id)
);
