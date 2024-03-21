import React, { useEffect, useState } from "react";
import Tab from "react-bootstrap/Tab";
import Tabs from "react-bootstrap/Tabs";
import Cards from "react-credit-cards";
import toast, { Toaster } from "react-hot-toast";

import { web3FromSource } from "@polkadot/extension-dapp";
import { hexToU8a } from "@polkadot/util";
import "react-credit-cards/es/styles-compiled.css";
import { useNavigate } from "react-router-dom";
import { Loader } from "semantic-ui-react";
import { ss58ToHex, useSubstrateState } from "../substrate-lib";
import {
  formatCVC,
  formatCreditCardNumber,
  formatExpirationDate,
} from "../utils";

const MERCHANT =
  "0xecd07df8b5fdd6c13e776c4720b325423d5c2449520266ca11dfd1735e28f572";

const Checkout = () => {
  const { api, currentAccount, keyring } = useSubstrateState();
  const navigate = useNavigate();
  const [quantity, setQuantity] = useState(1);
  const [amount, setAmount] = useState(0);
  const [loading, setLoading] = useState(false);

  const [cardDetails, setCardDetails] = useState({
    cvc: "",
    expiry: "",
    focus: "",
    name: "",
    number: "",
    txHash: null,
  });

  useEffect(() => {
    setAmount(20 * quantity);
  }, [quantity]);

  const handleInputChange = ({ target }) => {
    if (target.name === "number") {
      target.value = formatCreditCardNumber(target.value);
    } else if (target.name === "expiry") {
      target.value = formatExpirationDate(target.value);
    } else if (target.name === "cvc") {
      target.value = formatCVC(target.value);
    }
    console.log("name, value", target.name, target.value);

    setCardDetails({
      ...cardDetails,
      [target.name]: target.value,
    });
  };

  const handleInputFocus = ({ target }) => {
    setCardDetails({
      ...cardDetails,
      focus: target.name,
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    // TODO: Call your backend to create the Checkout session.
    console.log("cardDetails", cardDetails);

    fetch(`${process.env.MODE === "dev" ? "" : "http://0.0.0.0:3001"}/pos/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        cardNumber: cardDetails.number.replace(/\s+/g, ""),
        cvv: cardDetails.cvc,
        cardExpiration: cardDetails.expiry,
        amount: amount.toString(),
        txHash: cardDetails.txHash,
      }),
    })
      .then((res) => res.json())
      .then((data) => {
        console.log("Response", data);
        // go back to dashboard if approved
        if (data.status) {
          toast.success("Payment confirmed!");
          navigate("/");
        } else {
          toast.error("Transaction failed: " + data.message);
        }
      });
  };

  const initiateTransfer = async () => {
    const {
      meta: { source, isInjected },
    } = currentAccount;

    const tx = api.tx.iso8583.initiateTransfer(
      currentAccount.publicKey,
      hexToU8a(MERCHANT),
      amount
    );

    if (isInjected) {
      const injector = await web3FromSource(source);

      const hash = await tx.signAndSend(currentAccount.address, {
        signer: injector.signer,
      });
      toast.success("Transfer initiated with hash " + hash.toHex());
    } else {
      // get it from dev accounts
      console.log("account", currentAccount.address);
      let signer = keyring.getPair(currentAccount.address);
      const hash = await tx.signAndSend(signer);
      toast.success("Transfer initiated with hash " + hash.toHex());
    }

    setLoading(true);

    // wait for `ProcessedTransaction` event
    // navigate to dashboard
    api.query.system.events((events) => {
      events.forEach((record) => {
        // Extract the phase, event and the event types
        const { event } = record;

        // Show what we are busy with
        console.log(`\t${event.section}:${event.method}})`);

        if (
          event.section === "iso8583" &&
          event.method === "ProcessedTransaction"
        ) {
          const data = event.data.toJSON();
          if (
            ss58ToHex(data[0].from) === ss58ToHex(currentAccount.address) &&
            ss58ToHex(data[0].to) === MERCHANT.substring(2)
          ) {
            toast.success("Payment confirmed!");
            setLoading(false);
            navigate("/");
          }
        }
      });
    });
  };
  return (
    <>
      <Toaster position="top-left" reverseOrder={false} />
      <div className="sr-root">
        <div className="sr-main">
          <Loader size="huge" active={loading} />
          <section className="container">
            <div>
              <h1>Random photo</h1>
              <h4>Purchase an original photo</h4>
              <div className="pasha-image">
                <img
                  alt="Random asset from Picsum"
                  src="https://picsum.photos/280/320?random=4"
                  width="140"
                  height="160"
                />
              </div>
            </div>
            <form>
              <div className="quantity-setter">
                <button
                  className="increment-btn"
                  disabled={quantity === 1}
                  onClick={() => setQuantity(quantity - 1)}
                  type="button"
                >
                  -
                </button>
                <input
                  type="number"
                  id="quantity-input"
                  min="1"
                  max="10"
                  value={quantity}
                  name="quantity"
                  readOnly
                />
                <button
                  className="increment-btn"
                  disabled={quantity === 10}
                  onClick={() => setQuantity(quantity + 1)}
                  type="button"
                >
                  +
                </button>
              </div>
              <p className="sr-legal-text">Number of copies (max 10)</p>

              <button style={{ pointerEvents: "none" }} id="submit">
                {`Pay $${amount}`}
              </button>
            </form>
          </section>
        </div>
        <div className="sr-content">
          <section className="container">
            <Tabs
              defaultActiveKey="Card"
              id="uncontrolled-tab-example"
              className="mb-3"
            >
              <Tab eventKey="Card" title="Card">
                <div key="Payment">
                  <div className="App-payment">
                    <Cards
                      cvc={cardDetails.cvc}
                      expiry={cardDetails.expiry}
                      focused={cardDetails.focus}
                      name={cardDetails.name}
                      number={cardDetails.number}
                    />

                    <form onSubmit={handleSubmit}>
                      <div className="form-group">
                        <input
                          type="tel"
                          name="number"
                          className="form-control"
                          placeholder="Card Number"
                          pattern="[\d| ]{16,22}"
                          required
                          onChange={handleInputChange}
                          onFocus={handleInputFocus}
                        />
                      </div>
                      <div className="form-group">
                        <input
                          type="text"
                          name="name"
                          className="form-control"
                          placeholder="Name"
                          required
                          onChange={handleInputChange}
                          onFocus={handleInputFocus}
                        />
                      </div>
                      <div className="row">
                        <div className="col-6">
                          <input
                            type="tel"
                            name="expiry"
                            className="form-control"
                            placeholder="Valid Thru"
                            pattern="\d\d/\d\d"
                            required
                            onChange={handleInputChange}
                            onFocus={handleInputFocus}
                          />
                        </div>
                        <div className="col-6">
                          <input
                            type="tel"
                            name="cvc"
                            className="form-control"
                            placeholder="CVC"
                            pattern="\d{3,4}"
                            required
                            onChange={handleInputChange}
                            onFocus={handleInputFocus}
                          />
                        </div>
                      </div>
                      <input type="hidden" name="issuer" />
                      <div className="form-actions">
                        <button className="btn btn-primary btn-block">
                          PAY
                        </button>
                      </div>
                    </form>
                  </div>
                </div>
              </Tab>

              <Tab eventKey="Crypto" title="Crypto">
                <div key="polkadot-js">
                  <div className="form-actions">
                    <button
                      className="btn btn-primary btn-block"
                      onClick={initiateTransfer}
                    >
                      PAY
                    </button>
                  </div>
                </div>
              </Tab>
            </Tabs>
          </section>
        </div>
      </div>
    </>
  );
};

export default Checkout;
