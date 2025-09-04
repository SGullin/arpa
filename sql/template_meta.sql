create table if not exists template_meta (
    id serial primary key,
    pulsar_id int references pulsar_meta,
    checksum UUID,
    file_path text
);