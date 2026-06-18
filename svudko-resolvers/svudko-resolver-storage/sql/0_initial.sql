create table trusted_hosts(
    hostname text primary key not null,
    signature text not null
);
