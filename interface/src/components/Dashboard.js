/* eslint-disable */

import React, { useEffect, useState } from "react";
import { CopyToClipboard } from "react-copy-to-clipboard";
import toast, { Toaster } from "react-hot-toast";
import { Link, useNavigate } from "react-router-dom";
import { Button, Grid, Label, Table } from "semantic-ui-react";
import { u8aToHexCompact, useSubstrateState } from "../substrate-lib";
import { formatAmount } from "../utils";

const Dashboard = ({ state }) => {
  const { apiState, currentAccount } = useSubstrateState();
  const navigate = useNavigate();

  let [transactions, setTransactions] = useState([]);
  let [mutated, setMutated] = useState(true);
  let [bankAccount, setBankAccount] = useState(null);

  useEffect(() => {
    // Update the current currentAccount
    const fetchAccount = async () => {
      console.log("Ws is ready to use", state.oracleRpc.isConnected);

      let hex_pub_key = u8aToHexCompact(currentAccount.publicKey);

      const bankAccount = await state.oracleRpc.send(
        "pcidss_get_bank_account",
        [hex_pub_key]
      );

      if (!bankAccount) {
        // push to `/register` if no bank account is found
        navigate("/register");
        return;
      }

      const transactions = await state.oracleRpc.send(
        "pcidss_get_transactions",
        [hex_pub_key]
      );

      setBankAccount(bankAccount);
      setTransactions(transactions);
    };
    if (currentAccount && apiState === "READY") fetchAccount();
    setMutated(false);
  }, [mutated, currentAccount, apiState]);

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
        <Grid.Row>
          <Grid.Column>
            <h1>Account</h1>
            {!bankAccount ? (
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
                  <Table.Row key={bankAccount.card_number}>
                    <Table.Cell width={3} textAlign="right">
                      {bankAccount.card_holder_first_name}
                    </Table.Cell>
                    <Table.Cell width={3}>
                      <span
                        style={{
                          display: "inline-block",
                          minWidth: "20em",
                        }}
                      >
                        {bankAccount.card_number}
                      </span>
                      <CopyToClipboard text={bankAccount.card_number}>
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
                    <Table.Cell width={3}>{bankAccount.card_cvv}</Table.Cell>
                    <Table.Cell width={3}>
                      {bankAccount?.card_expiration_date?.slice(0, 7)}
                    </Table.Cell>
                    <Table.Cell width={3}>
                      ${formatAmount(bankAccount.balance)}
                    </Table.Cell>
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
                  <Table.Row key={bankAccount.card_number}>
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
                  {transactions.map((transaction) => (
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
                      <Table.Cell width={3}>
                        ${formatAmount(transaction.amount)}
                      </Table.Cell>
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
                            Revert Transaction
                          </Button>
                        </span>
                      </Table.Cell>
                    </Table.Row>
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
