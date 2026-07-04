create table device_id(
    id integer primary key not null check (id < 1),
    device blob not null
);

create table paired_devices(
    hostname text primary key not null,
    identifier text not null
);
