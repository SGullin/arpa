create table if not exists raw_meta (
    id serial primary key,
    file_path text,
    checksum UUID,
    pulsar_id integer references pulsar_meta,
    observer_id integer 
);