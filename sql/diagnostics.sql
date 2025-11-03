create table if not exists diag_floats (
    id serial primary key,
    process integer not null,
    diagnostic text not null,
    result float4 not null
);
create table if not exists diag_plots (
    id serial primary key,
    process integer not null,
    diagnostic text not null,
    filepath text not null
);