create table if not exists bank_transaction (
    id uuid primary key,
    hash varchar(256) not null,
    source uuid not null,
    recipient uuid,
    amount int not null,
    transaction_type int not null,
    reversed boolean default false,
    created_at timestamptz default now(),
    updated_at timestamptz default now(),
    foreign key (source) references bank_account(id)
);
