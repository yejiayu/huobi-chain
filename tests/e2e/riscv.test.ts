import { muta, CHAIN_CONFIG, delay, client, accounts, admin } from "./utils";
import { readFileSync } from "fs";

const account = accounts[0];

async function deploy(code, init_args, intp_type) {
  const tx = await client.composeTransaction({
    method: "deploy",
    payload: {
      intp_type,
      init_args,
      code: code.toString("hex")
    },
    serviceName: "riscv"
  });
  // console.log(tx);
  const tx_hash = await client.sendTransaction(account.signTransaction(tx));
  // console.log(tx_hash);

  const receipt = await client.getReceipt(tx_hash);
  // console.log('deploy:', { tx_hash, receipt });

  const addr = JSON.parse(receipt.response.ret).address;
  return addr;
}

async function query(address, args) {
  const res = await client.queryService({
    serviceName: "riscv",
    method: "call",
    payload: JSON.stringify({
      address,
      args
    })
  });
  // console.log('query:', {address, args, res});
  res.ret = JSON.parse(res.ret);
  return res;
}

async function exec(address, args) {
  const payload = {
    address,
    args
  };
  const exec_tx = await client.composeTransaction({
    payload,
    serviceName: "riscv",
    method: "exec"
  });
  // console.log('send_tx:', {address, args, exec_tx});
  const tx_hash = await client.sendTransaction(
    account.signTransaction(exec_tx)
  );
  // console.log('tx_hash:', tx_hash);
  const exec_receipt = await client.getReceipt(tx_hash);
  // console.log('send_tx:', {exec_receipt, address, args});
  return exec_receipt;
}

describe("riscv service", () => {
  test("test normal process", async () => {
    const code = readFileSync("../../services/riscv/src/tests/simple_storage");
    const addr = await deploy(code, "set k init", "Binary");
    // console.log(addr);
    const v_init = await query(addr, "get k");
    expect(v_init.ret).toBe("init");
    const exec_res = await exec(addr, "set k v");
    const v1 = await query(addr, "get k");
    expect(v1.ret).toBe("v");

    // get code
    const get_contract_res = await client.queryService({
      serviceName: "riscv",
      method: "get_contract",
      payload: JSON.stringify({
        address: addr,
        get_code: true,
        storage_keys: [Buffer.from("k", "utf8").toString("hex"), "", "1a"]
      })
    });
    // console.log(get_contract_res);
    expect(get_contract_res.isError).toBeFalsy();
    const ret = JSON.parse(get_contract_res.ret);
    expect(ret.code).toBe(code.toString("hex"));
    expect(ret.storage_values).toStrictEqual([
      Buffer.from("v", "utf8").toString("hex"),
      "",
      ""
    ]);
  });
});
