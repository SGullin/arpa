create table if not exists pulsar_meta (
    id serial primary key,
	alias text not null unique,
	j_name text,
	b_name text,
	j2000_ra text,
	j2000_dec text,
	master_parfile_id integer
);