/* eslint-disable */

import React, { useEffect, useState } from "react";
import { CopyToClipboard } from "react-copy-to-clipboard";
import toast, { Toaster } from "react-hot-toast";
import { Link } from "react-router-dom";
import { Button, Dropdown, Grid, Label, Table } from "semantic-ui-react";

const Dashboard = ({ state }) => {
  let [currentAccount, setCurrentAccount] = useState(state.accounts[0]);
  let [transactions, setTransactions] = useState([]);
  let [mutated, setMutated] = useState(true);

  useEffect(() => {
    // Update the current currentAccount
    const fetchAccount = async () => {
      console.log("Ws is ready to use", state.oracleRpc.isConnected);

      const bankAccount = await state.oracleRpc.send(
        "pcidss_get_bank_account",
        [currentAccount.card_number]
      );

      const transactions = await state.oracleRpc.send(
        "pcidss_get_transactions",
        [currentAccount.card_number]
      );

      setCurrentAccount(bankAccount);
      setTransactions(transactions);
    };
    fetchAccount();
    setMutated(false);
  }, [mutated]);

  const DEV_MODE = process.env.MODE === "dev";

  const onReverse = async (hash, amount) => {
    fetch(`${DEV_MODE ? "" : "http://0.0.0.0:3000"}/reverse`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        cardNumber: currentAccount.card_number,
        cardExpiration:
          currentAccount.card_expiration_date.slice(5, 7) +
          "/" +
          currentAccount.card_expiration_date.slice(2, 4),
        cvv: currentAccount.card_cvv,
        amount: amount.toString(),
        txHash: hash,
      }),
    })
      .then((res) => res.json())
      .then((data) => {
        console.log("data", data);

        if (data.status) {
          toast.success("Transaction reversed successfully");
        } else {
          toast.error("Transaction reversal failed: " + data.message);
        }
      });
  };

  return (
    <div className="sr-content">
      <Toaster position="top-center" reverseOrder={false} />
      <Grid columns={1}>
        <Grid columns={4}>
          <Grid.Column></Grid.Column>
          <Grid.Column></Grid.Column>
          <Grid.Column></Grid.Column>
          <Grid.Column>
            <Dropdown
              placeholder="Select Account"
              fluid
              selection
              value={currentAccount?.card_holder_first_name}
              options={state.accounts.map((account) => ({
                key: account.card_number,
                text: account?.card_holder_first_name,
                value: account?.card_holder_first_name,
              }))}
              onChange={(e, { value }) => {
                const account = state.accounts.find(
                  (account) => account?.card_holder_first_name === value
                );
                setCurrentAccount(account);
                setMutated(true);
              }}
            />
          </Grid.Column>
        </Grid>
        <Grid.Row>
          <Grid.Column>
            <h1>Balances</h1>
            {!currentAccount ? (
              <Label basic color="yellow">
                No accounts to be shown
              </Label>
            ) : (
              <Table celled striped size="small">
                <Table.Body>
                  <Table.Row>
                    <Table.Cell width={3} textAlign="right">
                      <strong>Name</strong>
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <strong>Card Number</strong>
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <strong>CVV</strong>
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <strong>Expiration Date</strong>
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <strong>Balance</strong>
                    </Table.Cell>
                    <Table.Cell width={10}>
                      <strong></strong>
                    </Table.Cell>
                  </Table.Row>
                  <Table.Row key={currentAccount.card_number}>
                    <Table.Cell width={3} textAlign="right">
                      {currentAccount.card_holder_first_name}
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <span
                        style={{
                          display: "inline-block",
                          minWidth: "20em",
                        }}
                      >
                        {currentAccount.card_number}
                      </span>
                      <CopyToClipboard text={currentAccount.card_number}>
                        <Button
                          basic
                          circular
                          compact
                          size="mini"
                          color="blue"
                          icon="copy outline"
                        />
                      </CopyToClipboard>
                    </Table.Cell>
                    <Table.Cell width={3}>{currentAccount.card_cvv}</Table.Cell>
                    <Table.Cell width={3}>
                      {currentAccount?.card_expiration_date?.slice(0, 7)}
                    </Table.Cell>
                    <Table.Cell width={3}>${currentAccount.balance}</Table.Cell>
                    <Table.Cell width={10}>
                      <span
                        style={{
                          display: "inline-block",
                          minWidth: "13em",
                        }}
                      >
                        <Link to="/checkout-demo">
                          <Button color="blue">Go to checkout demo</Button>
                        </Link>
                      </span>
                    </Table.Cell>
                  </Table.Row>
                </Table.Body>
              </Table>
            )}
          </Grid.Column>
        </Grid.Row>
        <Grid.Row>
          <Grid.Column>
            <h1>Transactions</h1>
            {transactions.length === 0 ? (
              <Label basic color="yellow">
                No Transactions to be shown
              </Label>
            ) : (
              <Table celled striped size="small">
                <Table.Body>
                  {transactions.map((transaction) => (
                    <>
                      <Table.Row key={transaction.hash}>
                        <Table.Cell width={3}>
                          <strong>Hash</strong>
                        </Table.Cell>
                        <Table.Cell width={3}>
                          <strong>To</strong>
                        </Table.Cell>
                        <Table.Cell width={3}>
                          <strong>Reversed</strong>
                        </Table.Cell>
                        <Table.Cell width={3}>
                          <strong>Amount</strong>
                        </Table.Cell>
                        <Table.Cell width={3} textAlign="right">
                          <strong>Transaction Type</strong>
                        </Table.Cell>
                        <Table.Cell width={10}>
                          <strong></strong>
                        </Table.Cell>
                      </Table.Row>
                      <Table.Row key={transaction.id}>
                        <Table.Cell width={3} textAlign="right">
                          <span
                            style={{
                              display: "inline-block",
                              minWidth: "15em",
                            }}
                          >
                            {transaction.hash}
                          </span>
                          <CopyToClipboard text={transaction.hash}>
                            <Button
                              basic
                              circular
                              compact
                              size="mini"
                              color="blue"
                              icon="copy outline"
                            />
                          </CopyToClipboard>
                        </Table.Cell>
                        <Table.Cell width={3}>{transaction?.to}</Table.Cell>
                        <Table.Cell width={3}>
                          {transaction.reversed.toString()}
                        </Table.Cell>
                        <Table.Cell width={3}>${transaction.amount}</Table.Cell>
                        <Table.Cell width={3}>
                          {currentAccount.id === transaction.from
                            ? "Credit"
                            : "Debit"}
                        </Table.Cell>
                        <Table.Cell width={10}>
                          <span
                            style={{
                              display: "inline-block",
                              minWidth: "8em",
                            }}
                          >
                            <Button
                              color="blue"
                              onClick={() =>
                                onReverse(transaction.hash, transaction.amount)
                              }
                            >
                              Reverse
                            </Button>
                          </span>
                        </Table.Cell>
                      </Table.Row>
                    </>
                  ))}
                </Table.Body>
              </Table>
            )}
          </Grid.Column>
        </Grid.Row>
      </Grid>
    </div>
  );
};

export default Dashboard;
