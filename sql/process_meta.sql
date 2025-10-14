create table if not exists process_meta (
    id serial primary key,
    par_id integer references par_meta,
    template_id integer,
    n_channels smallint,
    n_subints smallint,
    method text,
    user_id integer, -- references users,
    started_at timestamptz default (now())
);