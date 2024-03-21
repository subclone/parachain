// Development accounts
export const DEV_ACCOUNTS = [
  // Healthy account
  {
    card_holder_first_name: "Alice",
    card_number: "4169812345678901",
    card_cvv: "123",
    balance: 1000,
  },
  // Zero balance case
  {
    card_holder_first_name: "Bob",
    card_number: "4169812345678902",
    card_cvv: "124",
    balance: 0,
  },
  {
    card_holder_first_name: "Charlie",
    card_number: "4169812345678903",
    card_cvv: "125",
    balance: 12345,
  },
  {
    card_holder_first_name: "Dave",
    card_number: "4169812345678904",
    card_cvv: "126",
    balance: 1000000,
  },
  // Expired card
  {
    card_holder_first_name: "Eve",
    card_number: "4169812345678905",
    card_cvv: "127",
    balance: 1000,
  },
  // Mock acquirer account, i.e merchant
  {
    card_holder_first_name: "Acquirer",
    card_number: "123456",
    card_cvv: "000",
    balance: 1000000000,
  },
  // Alice stash
  {
    card_holder_first_name: "Alice_stash",
    card_number: "4169812345678908",
    card_cvv: "999",
    balance: 0,
  },
  // Bob stash
  {
    card_holder_first_name: "Bob_stash",
    card_number: "4169812345678909",
    card_cvv: "888",
    balance: 0,
  },
];
