create table projects (
    project_id serial primary key,
    name varchar not null,
    year integer not null,
    max_student_uploads integer not null,
    max_group_size integer not null,
    max_groups integer not null default 10,
    deliverable_selection_deadline timestamp,
    active boolean not null
);

create table admin_roles (
    admin_role_id serial primary key,
    name varchar not null unique
);

create table admins (
    admin_id serial primary key,
    first_name varchar not null,
    last_name varchar not null,
    email varchar not null unique,
    password_hash varchar not null,
    admin_role_id integer not null references admin_roles on delete restrict
);

create table student_roles (
    student_role_id serial primary key,
    name varchar not null unique
);

create table students (
    student_id serial primary key,
    first_name varchar not null,
    last_name varchar not null,
    email varchar not null unique,
    university_id integer not null unique,
    password_hash varchar not null,
    is_pending boolean default true not null
);

create table security_codes (
    security_code_id serial primary key,
    project_id integer not null references projects on delete cascade,
    user_role_id integer not null references student_roles on delete cascade,
    code varchar not null unique,
    expiration timestamp not null
);

create table blacklist (
    blacklist_id serial primary key,
    university_id integer not null unique,
    description text not null,
    first_name varchar not null,
    last_name varchar not null,
    banned_at timestamp not null
);

create table groups (
    group_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    created_at timestamp not null default now(),
    unique (project_id, name)
);

create table group_members (
    group_member_id serial primary key,
    group_id integer not null references groups on delete cascade,
    student_id integer not null references students on delete cascade,
    student_role_id integer not null references student_roles on delete cascade,
    joined_at timestamp not null default now(),
    unique (group_id, student_id)
);

create table complaints (
    complaint_id serial primary key,
    from_group_id integer not null references groups on delete cascade,
    to_group_id integer not null references groups on delete cascade,
    text text not null,
    created_at timestamp not null
);

create table fairs (
    fair_id serial primary key,
    project_id integer not null references projects on delete cascade,
    details varchar not null,
    start_date timestamp not null,
    end_date timestamp not null
);

create table group_deliverables (
    group_deliverable_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table group_deliverable_components (
    group_deliverable_component_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table group_deliverables_components (
    id serial primary key,
    group_deliverable_id integer not null references group_deliverables on delete cascade,
    group_deliverable_component_id integer not null references group_deliverable_components on delete cascade,
    quantity integer not null,
    unique (group_deliverable_id, group_deliverable_component_id)
);

create table group_deliverable_selections (
    group_deliverable_selection_id serial primary key,
    group_id integer not null references groups on delete cascade,
    group_deliverable_id integer not null references group_deliverables on delete cascade,
    link text not null unique,
    markdown_text text not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    unique (group_id)
);

create table transactions (
    transaction_id serial primary key,
    buyer_group_id integer not null references groups on delete cascade,
    group_deliverable_selection_id integer not null references group_deliverable_selections on delete cascade,
    fair_id integer not null references fairs on delete cascade,
    timestamp timestamp not null
);

create table student_deliverables (
    student_deliverable_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table student_deliverable_components (
    student_deliverable_component_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table student_deliverables_components (
    id serial primary key,
    student_deliverable_id integer not null references student_deliverables on delete cascade,
    student_deliverable_component_id integer not null references student_deliverable_components on delete cascade,
    quantity integer not null,
    unique (student_deliverable_id, student_deliverable_component_id)
);

create table student_deliverable_selections (
    student_deliverable_selection_id serial primary key,
    student_id integer not null references students on delete cascade,
    student_deliverable_id integer not null references student_deliverables on delete cascade,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    unique (student_id, student_deliverable_id)
);

create table student_uploads (
    upload_id serial primary key,
    student_deliverable_selection_id integer not null references student_deliverable_selections on delete cascade,
    path varchar not null unique,
    timestamp timestamp not null
);

-- Coordinator project assignments table
create table coordinator_projects (
    coordinator_project_id serial primary key,
    admin_id integer not null references admins(admin_id) on delete cascade,
    project_id integer not null references projects(project_id) on delete cascade,
    assigned_at timestamp not null default now(),
    unique (admin_id, project_id)
);
