create table if not exists process_meta (
    id serial primary key,
    raw_id integer,
    par_id integer references par_meta,
    template_id integer,
    n_channels smallint,
    n_subints smallint,
    method text,
    user_id integer references users
);