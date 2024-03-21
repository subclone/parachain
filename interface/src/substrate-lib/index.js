import { u8aToHex } from "@polkadot/util";
import { decodeAddress } from "@polkadot/util-crypto";
import {
  SubstrateContextProvider,
  useSubstrate,
  useSubstrateState,
} from "./SubstrateContext";
export { SubstrateContextProvider, useSubstrate, useSubstrateState };

const _utils = {
  paramConversion: {
    num: [
      "Compact<Balance>",
      "BalanceOf",
      "u8",
      "u16",
      "u32",
      "u64",
      "u128",
      "i8",
      "i16",
      "i32",
      "i64",
      "i128",
    ],
  },
};

export function u8aToHexCompact(data) {
  return u8aToHex(data).substring(2);
}

export function ss58ToHex(ss58) {
  const pubkeyData = decodeAddress(ss58);
  return u8aToHexCompact(pubkeyData);
}
