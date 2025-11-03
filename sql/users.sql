create table if not exists users (
    id serial primary key,
    username text not null unique, 
    real_name text not null,
    created_at timestamptz default (now()) not null,
    is_admin boolean not null,
    email text not null,
    pass_hash text not null
);