import React, { useState } from "react";
import Tab from "react-bootstrap/Tab";
import Tabs from "react-bootstrap/Tabs";
import Cards from "react-credit-cards";
import toast, { Toaster } from "react-hot-toast";

import "react-credit-cards/es/styles-compiled.css";
import { useNavigate } from "react-router-dom";
import { u8aToHexCompact, useSubstrateState } from "../substrate-lib";
import {
  formatCVC,
  formatCreditCardNumber,
  formatExpirationDate,
} from "../utils";

const Register = () => {
  const navigate = useNavigate();

  const { currentAccount } = useSubstrateState();
  const [cardDetails, setCardDetails] = useState({
    cvc: "",
    expiry: "",
    focus: "",
    name: "",
    number: "",
    txHash: null,
  });

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

    fetch(
      `${process.env.MODE === "dev" ? "" : "http://0.0.0.0:3001"}/register/`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          cardNumber: cardDetails.number.replace(/\s+/g, ""),
          cvv: cardDetails.cvc,
          cardExpiration: cardDetails.expiry,
          txHash: cardDetails.txHash,
          amount: 0,
          accountId: u8aToHexCompact(currentAccount.publicKey),
        }),
      }
    )
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

  return (
    <div className="sr-root">
      <Toaster position="top-center" reverseOrder={false} />
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
                        Register
                      </button>
                    </div>
                  </form>
                </div>
              </div>
            </Tab>
          </Tabs>
        </section>
      </div>
    </div>
  );
};

export default Register;
