/* eslint-env node, jest */
import { retry } from '@mutadev/client';
import {
  client,
  hexToNum,
  admin,
  // eslint-disable-next-line
} from "./utils";

describe('basic API test via muta-sdk-js', () => {
  test('getLatestBlockHeight', async () => {
    const currentHeight = await client.getLatestBlockHeight();
    expect(currentHeight).toBeGreaterThan(0);
  });

  test('getBlock', async () => {
    const block = await client.getBlock('0x01');
    expect(hexToNum(block.header.height)).toBe(1);
  });

  test('send_tx_exceed_cycles_limit', async () => {
    const tx = await client.composeTransaction({
      method: 'create_asset',
      payload: {
        name: 'Muta Token',
        symbol: 'MT',
        supply: 1000000000,
      },
      serviceName: 'asset',
      sender: admin.address,
    });
    tx.cyclesLimit = '0xE8D4A51FFF';
    const signedTx = admin.signTransaction(tx);

    try {
      await client.sendTransaction(signedTx);
      expect(true).toBe(false);
    } catch (err) {
      expect(err.response.errors[0].message.includes('ExceedCyclesLimit')).toBe(
        true,
      );
    }
  });

  test('send_tx_exceed_tx_size_limit', async () => {
    const tx = await client.composeTransaction({
      method: 'create_asset',
      payload: {
        name: 'Muta Token',
        symbol: 'MT',
        supply: 1000000000,
        bigdata: 'a'.repeat(300000),
      },
      serviceName: 'asset',
      sender: admin.address,
    });
    const signedTx = admin.signTransaction(tx);

    try {
      await client.sendTransaction(signedTx);
    } catch (err) {
      const errMsg = err.response.errors[0].message;
      expect(errMsg.includes('ExceedSizeLimit')).toBe(true);
    }
  });

  test('send tx, get tx and receipt', async () => {
    const tx = await client.composeTransaction({
      method: 'create_asset',
      payload: {
        name: 'Muta Token',
        symbol: 'MT',
        supply: 1000000000,
      },
      serviceName: 'asset',
      sender: admin.address,
    });
    const signedTx = admin.signTransaction(tx);

    const hash = await client.sendTransaction(signedTx);
    const receipt = await retry(() => client.getReceipt(hash));
    expect(receipt.txHash).toBe(hash);

    const committedTx = await client.getTransaction(hash);
    expect(committedTx.txHash).toBe(hash);
  });
});
