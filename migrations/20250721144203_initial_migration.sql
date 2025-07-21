TRUNCATE TABLE _sqlx_migrations;
DROP TABLE IF EXISTS student_uploads;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS group_part_selections;
DROP TABLE IF EXISTS student_part_selections;
DROP TABLE IF EXISTS student_parts_components;
DROP TABLE IF EXISTS group_parts_components;
DROP TABLE IF EXISTS complaints;
DROP TABLE IF EXISTS group_members;
DROP TABLE IF EXISTS security_codes;
DROP TABLE IF EXISTS fairs;
DROP TABLE IF EXISTS student_parts;
DROP TABLE IF EXISTS students_components;
DROP TABLE IF EXISTS group_components;
DROP TABLE IF EXISTS group_parts;
DROP TABLE IF EXISTS admins;
DROP TABLE IF EXISTS students;
DROP TABLE IF EXISTS groups;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS admin_roles;
DROP TABLE IF EXISTS student_roles;
DROP TABLE IF EXISTS blacklist;

create table projects (
    project_id serial primary key,
    name varchar not null,
    year integer not null,
    max_student_uploads integer not null,
    max_group_size integer not null,
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
    student_role_id integer not null references student_roles on delete cascade,
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
    unique (project_id, name)
);

create table group_members (
    group_member_id serial primary key,
    group_id integer not null references groups on delete cascade,
    student_id integer not null references students on delete cascade,
    student_role_id integer not null references student_roles on delete cascade,
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

create table group_parts (
    group_part_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table group_components (
    group_component_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table group_parts_components (
    group_component_id integer not null references group_components on delete cascade,
    group_part_id integer not null references group_parts on delete cascade,
    quantity integer not null,
    primary key (group_component_id, group_part_id)
);

create table group_part_selections (
    group_part_selection_id serial primary key,
    group_id integer not null references groups on delete cascade,
    link text not null unique,
    markdown_text text not null
);

create table transactions (
    transaction_id serial primary key,
    buyer_group_id integer not null references groups on delete cascade,
    group_part_selection_id integer not null references group_part_selections on delete cascade,
    fair_id integer not null references fairs on delete cascade,
    timestamp timestamp not null
);

create table student_parts (
    student_part_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table students_components (
    students_component_id serial primary key,
    project_id integer not null references projects on delete cascade,
    name varchar not null,
    unique (project_id, name)
);

create table student_parts_components (
    student_part_id integer not null references student_parts on delete cascade,
    students_component_id integer not null references students_components on delete cascade,
    quantity integer not null,
    primary key (student_part_id, students_component_id)
);

create table student_part_selections (
    student_part_selection_id serial primary key,
    student_id integer not null references students on delete cascade,
    student_part_id integer not null references student_parts on delete cascade
);

create table student_uploads (
    upload_id serial primary key,
    student_part_selection_id integer not null references student_part_selections on delete cascade,
    path varchar not null unique,
    timestamp timestamp not null
);
