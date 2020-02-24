import { muta, CHAIN_CONFIG, delay, client, accounts } from "./utils";

describe("basic API test via muta-sdk-js", () => {
  test("getLatestBlockHeight", async () => {
    const current_height = await client.getLatestBlockHeight();
    // console.log(current_height);
    expect(current_height).toBeGreaterThan(0);
  });

  test("getBlock", async () => {
    const block = await client.getBlock("0x1");
    // console.log(block);
    expect(block.header.height).toBe("0000000000000001");
  });

  test("send_tx_exceed_cycles_limit", async () => {
    const tx = await client.composeTransaction({
      method: "create_asset",
      payload: {
        name: "Muta Token",
        symbol: "MT",
        supply: 1000000000
      },
      serviceName: "asset"
    });
    tx.cyclesLimit = "0xE8D4A51FFF";
    const account = accounts[0];
    const signed_tx = account.signTransaction(tx);
    // console.log(signed_tx);
    try {
      const hash = await client.sendTransaction(signed_tx);
      expect(true).toBe(false);
    } catch (err) {
      // console.log(err);
      expect(err.response.errors[0].message.includes("ExceedCyclesLimit")).toBe(
        true
      );
    }
  });

  test("send_tx_exceed_tx_size_limit", async () => {
    const tx = await client.composeTransaction({
      method: "create_asset",
      payload: {
        name: "Muta Token",
        symbol: "MT",
        supply: 1000000000,
        bigdata: "a".repeat(30000)
      },
      serviceName: "asset"
    });
    const account = accounts[0];
    const signed_tx = account.signTransaction(tx);
    // console.log(signed_tx);
    try {
      const hash = await client.sendTransaction(signed_tx);
      expect(true).toBe(false);
    } catch (err) {
      // console.log(err);
      expect(err.response.errors[0].message.includes("ExceedSizeLimit")).toBe(
        true
      );
    }
  });

  test("send tx, get tx and receipt", async () => {
    const tx = await client.composeTransaction({
      method: "create_asset",
      payload: {
        name: "Muta Token",
        symbol: "MT",
        supply: 1000000000
      },
      serviceName: "asset"
    });
    const account = accounts[0];
    const signed_tx = account.signTransaction(tx);
    const hash = await client.sendTransaction(signed_tx);
    // console.log(hash);
    const receipt = await client.getReceipt(hash);
    // console.log(receipt);
    expect(receipt.txHash).toBe(hash);
    const get_signed_tx = await client.getTransaction(hash);
    // console.log(get_signed_tx);
    expect(get_signed_tx.txHash).toBe(hash);
  });
});
