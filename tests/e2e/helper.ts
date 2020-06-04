import {
  client, admin, feeAssetID,
  // eslint-disable-next-line
} from './utils';

export async function transfer(txSender, assetID, to, value) {
  const payload = {
    asset_id: assetID,
    to,
    value,
  };

  const tx = await client.composeTransaction({
    method: 'transfer',
    payload,
    serviceName: 'asset',
  });

  const signedTx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signedTx);
  const receipt = await client.getReceipt(hash);

  return receipt;
}

export async function addFeeTokenToAccounts(accounts) {
  await Promise.all(
    accounts.map((account) => transfer(admin, feeAssetID, account, 10000)),
  );
}

export async function getBalance(assetID, user) {
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
