import { client, accounts, admin, fee_asset_id } from "./utils";

export async function transfer(txSender, assetID, to, value) {
  const payload = {
    asset_id: assetID,
    to,
    value
  };

  const tx = await client.composeTransaction({
    method: "transfer",
    payload,
    serviceName: "asset"
  });
  const signed_tx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

export async function add_fee_token_to_accounts(accounts_address) {
  const res = await Promise.all(
    accounts_address.map(address =>
      transfer(admin, fee_asset_id, address, 10000)
    )
  );
  // console.log({accounts_address, res});
}

export async function getBalance(assetID, user) {
  const res = await client.queryService({
    serviceName: "asset",
    method: "get_balance",
    payload: JSON.stringify({
      asset_id: assetID,
      user: user
    })
  });
  return res;
}
