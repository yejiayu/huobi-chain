import { retry } from '@mutadev/client';
// eslint-disable-next-line no-unused-vars
import { Account } from '@mutadev/account';
// eslint-disable-next-line no-unused-vars
import { Hash } from '@mutadev/types';
import {
  client,
  admin,
  feeAssetID,
  // eslint-disable-next-line
} from "./utils";

export async function transfer(
  txSender: Account,
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
  const receipt = await retry(() => client.getReceipt(hash));

  return receipt;
}

export async function addFeeTokenToAccounts(accounts: Array<Hash>) {
  await Promise.all(
    accounts.map((account) => transfer(admin, feeAssetID, account, 10000)),
  );
}

export async function getBalance(assetID: string, user: string) {
  const res = await client.queryService({
    serviceName: 'asset',
    method: 'get_balance',
    payload: JSON.stringify({
      asset_id: assetID,
      user,
    }),
  });

  return res;
}
