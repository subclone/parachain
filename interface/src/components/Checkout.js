import React, { useEffect, useState } from "react";
import Tab from "react-bootstrap/Tab";
import Tabs from "react-bootstrap/Tabs";
import Cards from "react-credit-cards";

import "react-credit-cards/es/styles-compiled.css";
import {
  formatCreditCardNumber,
  formatCVC,
  formatExpirationDate,
  formatPrice,
} from "../utils";

const Checkout = () => {
  const [quantity, setQuantity] = useState(1);
  const [amount, setAmount] = useState(0);

  const [cardDetails, setCardDetails] = useState({
    cvc: "",
    expiry: "",
    focus: "",
    name: "",
    number: "",
  });

  useEffect(() => {
    setAmount(100 * quantity);
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
  };

  return (
    <div className="sr-root">
      <div className="sr-main">
        <section className="container">
          <div>
            <h1>Single photo</h1>
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

            <button role="link" id="submit" type="submit">
              Buy {formatPrice({ amount, quantity })}
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
                      <button className="btn btn-primary btn-block">PAY</button>
                    </div>
                  </form>
                </div>
              </div>
            </Tab>

            <Tab
              eventKey="Crypto"
              title="Crypto"
              disabled
              itemRef="stuff"
            ></Tab>
          </Tabs>
        </section>
      </div>
    </div>
  );
};

export default Checkout;