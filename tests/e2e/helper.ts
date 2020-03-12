import {
  client,
  accounts,
  admin,
  fee_asset_id,
} from "./utils";

async function transfer(txSender, assetID, to, value) {
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
  const res = await Promise.all(accounts_address.map(address => transfer(admin, fee_asset_id, address, 10000)));
  // console.log({accounts_address, res});
}
