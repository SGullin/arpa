create table if not exists users (
    id serial primary key,
    username text not null unique, 
    real_name text not null,
    created_at timestamptz default (now()),
    is_admin boolean,
    email text,
    pass_hash text
);