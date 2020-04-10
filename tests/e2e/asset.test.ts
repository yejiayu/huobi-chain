import {
  muta,
  CHAIN_CONFIG,
  delay,
  client,
  accounts,
  admin,
  fee_asset_id,
  fee_account
} from "./utils";

async function createAsset(txSender, name, symbol, supply, precision) {
  const payload = {
    name,
    symbol,
    supply,
    precision
  };
  const tx = await client.composeTransaction({
    method: "create_asset",
    payload,
    serviceName: "asset"
  });
  const signed_tx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

async function getAsset(assetID) {
  const res = await client.queryService({
    serviceName: "asset",
    method: "get_asset",
    payload: JSON.stringify({
      id: assetID
    })
  });
  return res;
}

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

async function getBalance(assetID, user) {
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

async function approve(txSender, assetID, to, value) {
  const payload = {
    asset_id: assetID,
    to,
    value
  };

  const tx = await client.composeTransaction({
    method: "approve",
    payload,
    serviceName: "asset"
  });
  const signed_tx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

async function getAllowance(assetID, grantor, grantee) {
  const res = await client.queryService({
    serviceName: "asset",
    method: "get_allowance",
    payload: JSON.stringify({
      asset_id: assetID,
      grantor,
      grantee
    })
  });
  return res;
}

async function transferFrom(txSender, assetID, sender, recipient, value) {
  const payload = {
    asset_id: assetID,
    sender,
    recipient,
    value
  };

  const tx = await client.composeTransaction({
    method: "transfer_from",
    payload,
    serviceName: "asset"
  });
  const signed_tx = txSender.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

describe("asset service API test via muta-sdk-js", () => {
  test("test normal process", async () => {
    // fee not enough
    let caReceipt = await createAsset(
      accounts[0],
      "Test Token",
      "TT",
      8888,
      10000
    );
    expect(caReceipt.response.isError).toBe(true);
    expect(caReceipt.response.ret).toBe(
      "[ProtocolError] Kind: Service Error: FeeNotEnough"
    );
    // add fee token to accounts
    await Promise.all(
      accounts.map(account =>
        transfer(admin, fee_asset_id, account.address, 10000)
      )
    );

    // Create asset
    const fee_account_balance_before = await getBalance(
      fee_asset_id,
      fee_account
    );
    caReceipt = await createAsset(accounts[0], "Test Token", "TT", 8888, 10000);
    expect(caReceipt.response.isError).toBe(false);
    const fee_account_balance_after = await getBalance(
      fee_asset_id,
      fee_account
    );
    const caRet = JSON.parse(caReceipt.response.ret);
    const assetID = caRet.id;

    // check fee account balance
    expect(
      JSON.parse(fee_account_balance_before.ret).balance <
        JSON.parse(fee_account_balance_after.ret).balance
    ).toBe(true);

    // Get asset
    const gaRes = await getAsset(assetID);
    const gaRet = JSON.parse(gaRes.ret);
    expect(gaRet.id).toBe(assetID);
    expect(gaRet.name).toBe("Test Token");
    expect(gaRet.symbol).toBe("TT");
    expect(gaRet.supply).toBe(8888);
    expect(gaRet.precision).toBe(10000);
    expect(gaRet.issuer).toBe(accounts[0].address);

    // Transfer
    const tranReceipt = await transfer(
      accounts[0],
      assetID,
      accounts[1].address,
      88
    );
    // console.log("transfer receipt: ", tranReceipt);
    expect(tranReceipt.response.isError).toBe(false);

    // Check balance
    const issuerBalanceRes = await getBalance(assetID, accounts[0].address);
    // console.log("balance res:", issuerBalanceRes);
    const issuerBalance = JSON.parse(issuerBalanceRes.ret).balance;
    let recipientBalanceRes = await getBalance(assetID, accounts[1].address);
    let recipientBalance = JSON.parse(recipientBalanceRes.ret).balance;
    expect(issuerBalance).toBe(8800);
    expect(recipientBalance).toBe(88);

    // Approve
    const apprReceipt = await approve(
      accounts[1],
      assetID,
      accounts[2].address,
      8
    );
    expect(apprReceipt.response.isError).toBe(false);

    // Check allowance
    let alloRes = await getAllowance(
      assetID,
      accounts[1].address,
      accounts[2].address
    );
    let allowance = JSON.parse(alloRes.ret).value;
    expect(allowance).toBe(8);

    // Transfer from
    const tfReceipt = await transferFrom(
      accounts[2],
      assetID,
      accounts[1].address,
      accounts[2].address,
      8
    );
    expect(tfReceipt.response.isError).toBe(false);

    // Check balance and allowance
    const senderBalanceRes = await getBalance(assetID, accounts[1].address);
    const senderBalance = JSON.parse(senderBalanceRes.ret).balance;
    recipientBalanceRes = await getBalance(assetID, accounts[2].address);
    recipientBalance = JSON.parse(recipientBalanceRes.ret).balance;
    expect(senderBalance).toBe(80);
    expect(recipientBalance).toBe(8);
    alloRes = await getAllowance(
      assetID,
      accounts[1].address,
      accounts[2].address
    );
    allowance = JSON.parse(alloRes.ret).value;
    expect(allowance).toBe(0);
  });
});
