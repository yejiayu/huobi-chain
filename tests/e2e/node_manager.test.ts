import { muta, admin as ADMIN, delay, client, accounts } from "./utils";

async function setAdmin(admin) {
  const tx = await client.composeTransaction({
    method: "set_admin",
    payload: {
      admin
    },
    serviceName: "node_manager"
  });
  const signed_tx = ADMIN.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

async function getAdmin() {
  const res = await client.queryService({
    serviceName: "node_manager",
    method: "get_admin",
    payload: ""
  });
  return res;
}

async function updateInterval(admin, interval) {
  const tx = await client.composeTransaction({
    method: "update_interval",
    payload: {
      interval
    },
    serviceName: "node_manager"
  });
  const signed_tx = admin.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

async function updateRatio(
  admin,
  propose_ratio,
  prevote_ratio,
  precommit_ratio,
  brake_ratio
) {
  const tx = await client.composeTransaction({
    method: "update_ratio",
    payload: {
      propose_ratio,
      prevote_ratio,
      precommit_ratio,
      brake_ratio
    },
    serviceName: "node_manager"
  });
  const signed_tx = admin.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  return receipt;
}

async function getMetadata() {
  const res = await client.queryService({
    serviceName: "metadata",
    method: "get_metadata",
    payload: ""
  });
  return res;
}

describe("node manager service API test via muta-sdk-js", () => {
  test("test regular progress", async () => {
    // Set admin
    let receipt = await setAdmin(accounts[0].address);
    expect(receipt.response.isError).toBe(false);
    let admin = accounts[0];
    // Get admin
    let res = await getAdmin();
    let ret = JSON.parse(res.ret);
    expect("0x" + ret).toBe(accounts[0].address);

    // Update interval
    receipt = await updateInterval(admin, 666);
    expect(receipt.response.isError).toBe(false);
    res = await getMetadata();
    ret = JSON.parse(res.ret);
    expect(ret.interval).toBe(666);

    // Update ratio
    receipt = await updateRatio(admin, 16, 16, 16, 6);
    expect(receipt.response.isError).toBe(false);
    res = await getMetadata();
    ret = JSON.parse(res.ret);
    expect(ret.propose_ratio).toBe(16);
    expect(ret.prevote_ratio).toBe(16);
    expect(ret.precommit_ratio).toBe(16);
    expect(ret.brake_ratio).toBe(6);
  });
});
