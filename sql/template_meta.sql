create table if not exists template_meta (
    id serial primary key,
    pulsar_id int not null,
    checksum UUID,
    file_path text
);