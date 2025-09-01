create table if not exists telescopes (
    id serial primary key,
    name text not null,
    abbreviation text not null,
    code text not null,
    itrf_x double precision not null,
    itrf_y double precision not null,
    itrf_z double precision not null

    -- long/lat, datum? 
);
insert into telescopes 
(name,          itrf_x,     itrf_y,     itrf_z,         abbreviation,   code) values
('effelsberg',  4033949.5,  486989.4,   4900430.8,      'eff',          'g'),
('jodrell',     3822626.04, -154105.65, 5086486.04,     'jb',           '8'),
('nancay',      4324165.81, 165927.11,  4670132.83,     'ncy',          'f'),
('wsrt',        3828445.659,445223.6,   5064921.5677,   'wsrt',         'i');

create table if not exists obs_systems(
    id serial primary key,
    name text not null,
    telescope_id integer not null,
    frontend text not null,
    backend text not null,
    clock text not null,
    code text not null
);
insert into obs_systems 
(name,       telescope_id,  frontend,   backend,    clock,              code) values
('eff_rfsoc_p217',      1,  'p217',     'rfsoc',    'unknown',           'g'),
-- Old toaster configs
('eff_asterix_7-beam',  1,  '7-beam',   'asterix',  'eff2gps.clk',      'g'),
('eff_asterix_20cm',    1,  '20cm',     'asterix',  'eff2gps.clk',      'g'),
('eff_asterix_11cm',    1,  '11cm',     'asterix',  'eff2gps.clk',      'g'),
('jb_roach_l-band',     2,  'l-band',   'roach',    'jbdfb2gps.clk',    'q'),
('jb_dfb_l-band',       2,  'l_band',   'dfb',      'jbdfb2gps.clk',    'q'),
('ncy_bon512_l-band',   3,  'l-band',   'bon512',   'ncyobs2obspm.clk', 'f'),
('ncy_bon512_s-band',   3,  's-band',   'bon512',   'ncyobs2obspm.clk', 'f'),
('ncy_bon128_l-band',   3,  'l-band',   'bon128',   'ncy2gps.clk',      'f'),
('ncy_bon128_s-band',   3,  's-band',   'bon128',   'ncy2gps.clk',      'f'),
('wsrt_puma2',          4,  'mffe',     'puma2',    'wsrt2gps.clk',     'i');