-- # Entity schema.

-- Create `users` table.
create table if not exists users (
    id integer primary key autoincrement,
    username text not null unique,
    password text not null
);


-- Create `avatar` table.
create table if not exists avatars (
    id integer primary key autoincrement,
    img_data blob not null,
    user_id integer references users(id)
);

-- Creating the Articles table
CREATE TABLE articles (
    id integer primary key autoincrement,
    user_id integer references users(id),
    title text not null,
    editor_content json not null,
    content text not null,
);

-- Creating the Posts table
CREATE TABLE posts (
    id integer primary key autoincrement,
    thread_id integer references threads(id),
    user_id integer references users(id),
    title text not null,
    content text not null
);

-- Creating the Categories table
CREATE TABLE categories (
    id integer primary key autoincrement,
    title text not null,
    content text
);

-- Creating the Threads table
CREATE TABLE threads (
    id integer primary key autoincrement,
    category_id integer references categories(id),
    user_id integer references users(id),
    title text not null
);

-- Create `groups` table.
create table if not exists groups (
    id integer primary key autoincrement,
    name text not null unique
);

-- Create `permissions` table.
create table if not exists permissions (
    id integer primary key autoincrement,
    name text not null unique
);


-- # Join tables.

-- Create `users_groups` table for many-to-many relationships between users and groups.
create table if not exists users_groups (
    user_id integer references users(id),
    group_id integer references groups(id),
    primary key (user_id, group_id)
);

-- Create `groups_permissions` table for many-to-many relationships between groups and permissions.
create table if not exists groups_permissions (
    group_id integer references groups(id),
    permission_id integer references permissions(id),
    primary key (group_id, permission_id)
);


-- # Fixture hydration.

-- Insert "ferris" user.
insert into users (username, password)
values (
    'ferris',
    '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw'
);

-- Insert "admin" user.
insert into users (username, password)
values (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw'
);

-- Insert "users" and "superusers" groups.
insert into groups (name) values ('users');
insert into groups (name) values ('superusers');

-- Insert individual permissions.
insert into permissions (name) values ('protected.read');
insert into permissions (name) values ('restricted.read');

-- Insert group permissions.
insert into groups_permissions (group_id, permission_id)
values (
    (select id from groups where name = 'users'),
    (select id from permissions where name = 'protected.read')
), (
    (select id from groups where name = 'superusers'),
    (select id from permissions where name = 'restricted.read')
);

-- Assign edit and delete permissions to the 'users' group (for their own posts)
INSERT INTO groups_permissions (group_id, permission_id)
VALUES
    ((SELECT id FROM groups WHERE name = 'users'), (SELECT id FROM permissions WHERE name = 'edit_own_post')),
    ((SELECT id FROM groups WHERE name = 'users'), (SELECT id FROM permissions WHERE name = 'delete_own_post'));

-- Assign edit and delete any post permissions to the 'superusers' group
INSERT INTO groups_permissions (group_id, permission_id)
VALUES
    ((SELECT id FROM groups WHERE name = 'superusers'), (SELECT id FROM permissions WHERE name = 'edit_any_post')),
    ((SELECT id FROM groups WHERE name = 'superusers'), (SELECT id FROM permissions WHERE name = 'delete_any_post'));

-- Insert users into groups.
insert into users_groups (user_id, group_id)
values (
    (select id from users where username = 'ferris'),
    (select id from groups where name = 'users')
), (
    (select id from users where username = 'admin'),
    (select id from groups where name = 'users')
), (
    (select id from users where username = 'admin'),
    (select id from groups where name = 'superusers')
);