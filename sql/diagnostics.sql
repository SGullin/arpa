create table if not exists diag_floats (
    id serial primary key,
    process integer,
    diagnostic text,
    result float
);
create table if not exists diag_plots (
    id serial primary key,
    process integer,
    diagnostic text,
    filepath text
);