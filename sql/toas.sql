create table if not exists toas (
    id serial primary key,
    -- Toaster has these ----------------
    process_id integer not null,
    template_id integer not null,
    rawfile_id integer not null,
    -- The data -------------------------
    pulsar_id integer not null,
    observer_id integer not null,
    toa_int integer,
    toa_frac double precision not null,
    toa_err real,
    frequency real not null
);