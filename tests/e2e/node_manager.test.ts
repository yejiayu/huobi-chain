/* eslint-env node, jest */
import {
  client, accounts, hexToNum, admin as presetAdmin,
  // eslint-disable-next-line
} from './utils';
// eslint-disable-next-line
import { addFeeTokenToAccounts } from './helper';

async function setAdmin(admin) {
  const tx = await client.composeTransaction({
    method: 'set_admin',
    payload: {
      admin,
    },
    serviceName: 'node_manager',
  });

  const signedTx = presetAdmin.signTransaction(tx);
  const hash = await client.sendTransaction(signedTx);
  const receipt = await client.getReceipt(hash);

  return receipt;
}

async function getAdmin() {
  const res = await client.queryService({
    serviceName: 'node_manager',
    method: 'get_admin',
    payload: '',
  });
  return res;
}

async function updateInterval(admin, interval) {
  const tx = await client.composeTransaction({
    method: 'update_interval',
    payload: {
      interval,
    },
    serviceName: 'node_manager',
  });

  const signedTx = admin.signTransaction(tx);
  const hash = await client.sendTransaction(signedTx);
  const receipt = await client.getReceipt(hash);

  return receipt;
}

async function updateRatio(
  admin,
  proposeRatio,
  prevoteRatio,
  precommitRatio,
  brakeRatio,
) {
  const tx = await client.composeTransaction({
    method: 'update_ratio',
    payload: {
      propose_ratio: proposeRatio,
      prevote_ratio: prevoteRatio,
      precommit_ratio: precommitRatio,
      brake_ratio: brakeRatio,
    },
    serviceName: 'node_manager',
  });

  const signedTx = admin.signTransaction(tx);
  const hash = await client.sendTransaction(signedTx);
  const receipt = await client.getReceipt(hash);

  return receipt;
}

async function getMetadata() {
  const res = await client.queryService({
    serviceName: 'metadata',
    method: 'get_metadata',
    payload: '',
  });
  return res;
}

describe('node manager service API test via muta-sdk-js', () => {
  beforeAll(async () => {
    await addFeeTokenToAccounts(accounts.map((a) => a.address));
  });

  test('test regular progress', async () => {
    // Set admin
    let receipt = await setAdmin(accounts[0].address);
    expect(hexToNum(receipt.response.response.code)).toBe(0);

    // Get admin
    let res = await getAdmin();
    expect(hexToNum(res.code)).toBe(0);

    const adminAddr = JSON.parse(res.succeedData);
    expect(adminAddr).toBe(accounts[0].address);

    // Update interval
    const admin = accounts[0];
    receipt = await updateInterval(admin, 666);
    expect(hexToNum(receipt.response.response.code)).toBe(0);

    res = await getMetadata();
    let metadata = JSON.parse(res.succeedData);
    expect(hexToNum(metadata.interval)).toBe(666);

    // Update ratio
    receipt = await updateRatio(admin, 16, 16, 16, 6);
    expect(hexToNum(receipt.response.response.code)).toBe(0);

    res = await getMetadata();
    metadata = JSON.parse(res.succeedData);

    expect(metadata.propose_ratio).toBe(16);
    expect(metadata.prevote_ratio).toBe(16);
    expect(metadata.precommit_ratio).toBe(16);
    expect(metadata.brake_ratio).toBe(6);
  });
});
