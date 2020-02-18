import { readFileSync } from "fs";
import { Muta, Client } from "muta-sdk";
const toml = require("toml");

export const CHAIN_CONFIG = toml.parse(readFileSync("./chain.toml", "utf-8"));
export const GENESIS = toml.parse(readFileSync("./genesis.toml", "utf-8"));

export const muta = Muta.createDefaultMutaInstance();
// export const client = muta.client('0xffffffff', '0x1');

export const endpoint = process.env.ENDPOINT || "http://127.0.0.1:8000/graphql";
export const client = new Client({
  chainId: "0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
  defaultCyclesLimit: "0xffffffff",
  defaultCyclesPrice: "0x1",
  endpoint,
  maxTimeout: 50000
});

export function makeid(length: number) {
  var result = "";
  var characters = "abcdef0123456789";
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

export function getNonce() {
  return makeid(64);
}

export function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

import * as _ from "lodash";
const mnemonic = Muta.hdWallet.generateMnemonic();
export const wallet = new Muta.hdWallet(mnemonic);
export const accounts = _.range(20).map(i => wallet.deriveAccount(i));
export const admin = Muta.accountFromPrivateKey(
  "0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2"
);

export function str2hex(s) {
  return Buffer.from(s, "utf8").toString("hex");
}
