create table if not exists bank_account (
    id uuid primary key,
    card_number varchar(63) not null unique,
    card_holder_first_name varchar(63) not null,
    card_holder_last_name varchar(63) not null,
    card_expiration_date timestamptz not null,
    card_cvv varchar(3) not null,
    balance int default 0,
    nonce int default 0,
    account_id char(64),
    created_at timestamptz default now(),
    updated_at timestamptz default now()
);
