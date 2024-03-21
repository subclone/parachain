import { WsProvider } from "@polkadot/api";
import React, { createRef, useEffect, useState } from "react";
import { Route, BrowserRouter as Router, Routes } from "react-router-dom";
import { Dimmer, Grid, Loader, Message, Sticky } from "semantic-ui-react";

import AccountSelector from "./components/AccountSelector";
import Checkout from "./components/Checkout";
import Dashboard from "./components/Dashboard";
import Register from "./components/Register";
import Success from "./components/Success";
import { SubstrateContextProvider, useSubstrateState } from "./substrate-lib";

function Main() {
  let [oracleRpc, setOracleRpc] = useState(null);
  let [isConnected, setIsConnected] = useState(false);
  const loadWs = async () => {
    const ws = new WsProvider("ws://0.0.0.0:3030");
    setOracleRpc(ws);
  };

  oracleRpc?.on("connected", () => {
    console.log("connected");
    setIsConnected(true);
  });

  useEffect(() => {
    loadWs();
  }, []);

  const { apiState, apiError, keyringState } = useSubstrateState();

  const loader = (text) => (
    <Dimmer active>
      <Loader size="small">{text}</Loader>
    </Dimmer>
  );

  const message = (errObj) => (
    <Grid centered columns={2} padded>
      <Grid.Column>
        <Message
          negative
          compact
          floating
          header="Error Connecting to Substrate"
          content={`Connection to websocket '${errObj.target.url}' failed.`}
        />
      </Grid.Column>
    </Grid>
  );

  if (apiState === "ERROR") return message(apiError);
  else if (apiState !== "READY") return loader("Connecting to Substrate");
  if (keyringState !== "READY") {
    return loader(
      "Loading accounts (please review any extension's authorization)"
    );
  }

  const contextRef = createRef();

  if (isConnected) {
    return (
      <div ref={contextRef}>
        <Sticky>
          <AccountSelector />
        </Sticky>
        <Router>
          <Routes>
            <Route path="/success" element={<Success />}></Route>
            <Route
              path="/checkout-demo"
              element={<Checkout state={{ oracleRpc }} />}
            ></Route>
            <Route
              path="/"
              element={<Dashboard state={{ oracleRpc }} />}
            ></Route>
            <Route path="/register" element={<Register />}></Route>
          </Routes>
        </Router>
      </div>
    );
  }
}

export default function App() {
  return (
    <SubstrateContextProvider>
      <Main />
    </SubstrateContextProvider>
  );
}
