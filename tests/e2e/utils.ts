import { readFileSync } from 'fs';
import { Muta, utils } from '@mutadev/muta-sdk';
import { Account } from '@mutadev/account';
import { parse } from 'toml';
import { find, range } from 'lodash';

const { hexToNum } = utils;

const ADMIN_PRIVATE_KEY = '0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2';
const apiUrl = process.env.API_URL || 'http://localhost:8000/graphql';

const genesis = parse(readFileSync('./genesis.toml', 'utf-8'));
const metadata = JSON.parse(
  find(genesis.services, (s) => s.name === 'metadata').payload,
);
const chainId = metadata.chain_id;

const muta = new Muta({
  endpoint: apiUrl,
  chainId,
});
const client = muta.client('0xffffffff', '0x1');

const mnemonic = Muta.hdWallet.generateMnemonic();
// eslint-disable-next-line
const wallet = new Muta.hdWallet(mnemonic);
const accounts = range(20).map((i) => wallet.deriveAccount(i));
const admin: Account = Account.fromPrivateKey(ADMIN_PRIVATE_KEY);

const assetGenesis = JSON.parse(
  find(genesis.services, (o) => o.name === 'asset').payload,
);
const feeAssetID = assetGenesis.id;
const feeAccount = assetGenesis.fee_acocunt;

const randomString = require("randomstring");

export async function transfer(
  txSender: any,
  assetID: any,
  to: any,
  value: any,
) {
  const payload = {
    asset_id: assetID,
    to,
    value,
  };

  const tx = await client.composeTransaction({
    method: 'transfer',
    payload,
    serviceName: 'asset',
    sender: txSender.address,
  });

  const signedTx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signedTx);
  const receipt = await client.getReceipt(hash);

  return receipt;
}

export function genRandomString(prefix: String = 'r', length: number = 12) {
  expect(prefix.length <= length);
  return prefix + randomString.generate(length - prefix.length);
}

export function genRandomStrings(size: number = 3, prefix: String = 't', length: number = 12) {
  const names = new Array(0);

  for(var i = 0; i < size; i++) {
    names.push(genRandomString(prefix, length));
  }

  return names;
}

export function genRandomAccount() {
  const randomPriKey = randomString.generate({
    charset: '0123456789abcdef',
    length: 64,
  });
  return Account.fromPrivateKey('0x' + randomPriKey);
}

export {
  accounts, admin, client, feeAssetID, feeAccount, hexToNum
};
