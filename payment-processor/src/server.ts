import { WsProvider } from "@polkadot/api";
import cors from "cors";
import express, { Router } from "express";
import helmet from "helmet";
import morgan from "morgan";
import { MTI, ProcessingCode, RequestBody } from "./types";
import { ensurePadded, responseCodeToMessage } from "./utils";
// @ts-ignore
import iso8583 from "iso_8583";

// Custom format for `126` field
let CUSTOM_FORMATS = {
  "4": {
    ContentType: "n",
    Label: "Amount",
    LenType: "fixed",
    MaxLen: 20,
  },
  "126": {
    ContentType: "ans",
    Label: "Private data",
    LenType: "llvar",
    MaxLen: 100,
  },
  "127": {
    ContentType: "ans",
    Label: "Private data 2",
    LenType: "llvar",
    MaxLen: 100,
  },
};

/**
 * Represents the server
 */
export default class Server {
  public app: express.Application;
  public oracle_rpc: WsProvider;
  public env: string;
  public port: number;

  constructor() {
    this.app = express();
    this.env = process.env.NODE_ENV || "development";
    this.port = process.env.PORT ? parseInt(process.env.PORT) : 3001;
    this.oracle_rpc = new WsProvider(
      process.env.ORACLE_RPC_URL || "ws://0.0.0.0:3030",
      1000
    );

    this.initMiddlewares();
  }

  // Starts the server
  public async start() {
    try {
      await this.oracle_rpc.connect();
    } catch (_) {}
    console.log(
      "Connected to oracle at",
      process.env.ORACLE_RPC_URL || "ws://0.0.0.0:3030"
    );

    this.initRoutes();

    this.listen();
  }

  public listen() {
    this.app.listen(this.port, "0.0.0.0", () => {
      console.log(`Server listening on port ${this.port}`);
    });
  }

  /// Initializes routes
  private initRoutes() {
    let router = Router();

    router.get("/", (req: express.Request, res: express.Response) => {
      res.send("Health check");
    });

    router.post("/pos", async (req: express.Request, res: express.Response) =>
      this.submitIso8583(req, res)
    );

    router.post(
      "/reverse",
      async (req: express.Request, res: express.Response) => {
        this.submitIso8583(req, res);
      }
    );

    router.post(
      "/register",
      async (req: express.Request, res: express.Response) => {
        this.submitIso8583(req, res);
      }
    );

    router.post(
      "/balances",
      async (req: express.Request, res: express.Response) => {
        this.fetchAccountsBalances(req, res);
      }
    );

    this.app.use(router);
  }

  // Fetch batch accounts balances from oracle RPC
  private async fetchAccountsBalances(
    req: express.Request,
    res: express.Response
  ) {
    try {
      let accounts = req.body?.accounts || [];
      let signature = req.body?.signature || "";

      if (accounts.length === 0 || signature === "") {
        res.status(400).json({
          status: false,
          message: "Accounts are required",
        });
        return;
      }

      // convert signature to ArrayBuffer, it is string in hex format now
      signature = Array.from(Buffer.from(signature, "hex"));

      let msgResponse = await this.oracle_rpc.send(
        "pcidss_get_batch_balances",
        [signature.slice(1), accounts]
      );

      res.status(200).json(
        msgResponse.map((x: any[]) => {
          return { accountId: x[0], balance: x[1] };
        })
      );
    } catch {
      res.status(500).json({
        status: false,
        message: "Internal server error",
      });
    }
  }

  // POS implementation
  //
  // This function does the following:
  //
  // 1. Extract variables from the request
  // 2. Do some validation
  // 3. Form ISO-8583 message
  // 4. Send the message to the PCIDSS compliant oracle
  // 5. Wait for the response
  // 6. Send the response back to the client
  private async submitIso8583(req: express.Request, res: express.Response) {
    const data = this.formIsoData(req.body);
    const isopack = new iso8583(data, CUSTOM_FORMATS);

    try {
      let msgResponse = await this.oracle_rpc.send("pcidss_submit_iso8583", [
        // slicing the first two bytes, because they are the length of the message
        // RPC doesn't expect it
        Array.from(isopack.getBufferMessage().slice(2)),
      ]);
      await this.processResponse(msgResponse, res);
    } catch {
      res.status(500).json({
        status: false,
        message: "Internal server error",
      });
    }
  }

  // Processes the response from the oracle
  private async processResponse(isoMsg: any[], res: express.Response) {
    // convert length of message to two bytes u16
    let len = isoMsg.length;
    let lenBytes: any[] = [];
    lenBytes[0] = len >> 8;
    lenBytes[1] = len & 0xff;

    let isopack = new iso8583({}, CUSTOM_FORMATS);
    let msg: any = isopack.getIsoJSON(Buffer.from(lenBytes.concat(isoMsg)), {
      lenEncoding: "hex",
      bitmapEncoding: "hex",
      secondaryBitmap: true,
    });

    console.log("Response from oracle", msg);

    const responseCode = msg["39"];

    let message = responseCodeToMessage(responseCode);

    res.status(200).json({
      status: responseCode === "00",
      message,
      result: msg["126"],
    });
  }

  // Forms a custom data to be passed to `ISO8583` pack
  private formIsoData(body: RequestBody): Record<string, string> {
    const isoNow = new Date();
    const now = isoNow.toISOString();

    // Format is `hhmmss`
    const timeDate = [
      now.slice(11, 13),
      now.slice(14, 16),
      now.slice(17, 19),
    ].join("");

    // Format is `MMDD`
    const monthDay = [now.slice(5, 7), now.slice(8, 10)].join("");
    const {
      amount,
      cardNumber,
      cardExpiration,
      cvv,
      txHash,
      accountId,
    }: RequestBody = body;

    let isReversal = !!txHash;
    let registerOnChainAccount = !!accountId;

    console.log("registerOnChainAccount", registerOnChainAccount, accountId);

    // Format is `MMDDhhmmss`
    const transmissionDate = `${monthDay}${timeDate}`;

    const track2 = `${cardNumber}D${cardExpiration.replace("/", "")}C${cvv}`;

    const mti = isReversal
      ? MTI.ReversalRequest
      : registerOnChainAccount
      ? MTI.NetworkManagementRequest
      : MTI.AuthorizationRequest;

    /// Private data is either `txHash` or `accountId`
    const privateData = isReversal ? txHash : accountId;

    return {
      0: mti,
      2: cardNumber,
      3: ProcessingCode.Purchase,
      4: ensurePadded(amount.toString(), 20), // Amount is 12 characters long, check the length of amount and pad it with `0` from the left
      7: transmissionDate,
      12: timeDate,
      32: "123456", // Acquiring institution ID, hard coded, for now
      35: track2, // Track-2 Data
      126: privateData ?? "0".repeat(99), // dummy 100 bytes, will be replaced in the future
      // 127: "0".repeat(99), // dummy 100 bytes, will be replaced in the future
    };
  }

  // Initializes middlewares
  private initMiddlewares() {
    this.app.use(
      morgan("combined", {
        stream: {
          write: (message) =>
            console.log(message.substring(0, message.lastIndexOf("\n"))),
        },
      })
    );
    this.app.use(
      cors({
        origin: ["http://0.0.0.0:3002", "http://localhost:3002"],
        preflightContinue: true,
        credentials: false,
      })
    );
    this.app.use(helmet());
    this.app.use(express.json());
    this.app.use(express.urlencoded({ extended: true }));
  }
}
