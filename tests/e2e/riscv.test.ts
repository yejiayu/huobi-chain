import {
  muta,
  CHAIN_CONFIG,
  delay,
  client,
  accounts,
  admin,
  str2hex,
  fee_asset_id
} from "./utils";
import { add_fee_token_to_accounts, getBalance, transfer } from "./helper";
import { readFileSync } from "fs";
import { Muta } from "muta-sdk";

const account = Muta.accountFromPrivateKey(
  "d6ef93ed5d27327fd10349a75d3b7a91aa5c1d0f42994be10c1cb0e357e722f5"
);

async function deploy(code, init_args, intp_type, acc = null) {
  const account_to_sign = acc || account;
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
  const tx_hash = await client.sendTransaction(
    account_to_sign.signTransaction(tx)
  );
  // console.log(tx_hash);

  const receipt = await client.getReceipt(tx_hash);
  // console.log('deploy:', { tx_hash, receipt });

  try {
    const addr = JSON.parse(receipt.response.ret).address;
    return addr;
  } catch (err) {
    throw receipt;
  }
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
  beforeAll(async () => {
    let accounts_to_add_fee = accounts.map(a => a.address);
    accounts_to_add_fee.push(account.address);
    await add_fee_token_to_accounts(accounts_to_add_fee);
  });

  test("test_riscv_deploy_auth", async () => {
    const acc = accounts[1];
    const code = readFileSync("../../services/riscv/src/tests/simple_storage");
    // not authed
    try {
      let addr = await deploy(code, "set k init", "Binary", acc);
      expect(true).toBe(false);
    } catch (err) {
      expect(err.response.ret).toBe(
        "[ProtocolError] Kind: Service Error: NonAuthorized"
      );
    }

    // check auth
    let deploy_auth_res = await client.queryService({
      serviceName: "riscv",
      method: "check_deploy_auth",
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address]
      })
    });
    // console.log({deploy_auth_res});
    expect(deploy_auth_res.isError).toBe(false);
    expect(JSON.parse(deploy_auth_res.ret).addresses).toStrictEqual([]);

    // grant deploy auth to account
    let tx = await client.composeTransaction({
      method: "grant_deploy_auth",
      payload: {
        addresses: [acc.address]
      },
      serviceName: "riscv"
    });
    let tx_hash = await client.sendTransaction(admin.signTransaction(tx));
    let receipt = await client.getReceipt(tx_hash);
    let addr = await deploy(code, "set k init", "Binary", acc);

    // check auth
    deploy_auth_res = await client.queryService({
      serviceName: "riscv",
      method: "check_deploy_auth",
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address]
      })
    });
    // console.log({deploy_auth_res});
    expect(deploy_auth_res.isError).toBe(false);
    expect(JSON.parse(deploy_auth_res.ret).addresses).toStrictEqual([
      acc.address
    ]);

    // revoke auth
    tx = await client.composeTransaction({
      method: "revoke_deploy_auth",
      payload: {
        addresses: [acc.address]
      },
      serviceName: "riscv"
    });
    tx_hash = await client.sendTransaction(admin.signTransaction(tx));
    receipt = await client.getReceipt(tx_hash);

    // check auth
    deploy_auth_res = await client.queryService({
      serviceName: "riscv",
      method: "check_deploy_auth",
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address]
      })
    });
    // console.log({deploy_auth_res});
    expect(deploy_auth_res.isError).toBe(false);
    expect(JSON.parse(deploy_auth_res.ret).addresses).toStrictEqual([]);
  });

  test("test_riscv_normal_process", async () => {
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

  test("test_service_call", async () => {
    const code = readFileSync("./riscv_contracts/contract_test");
    const addr = await deploy(code, "", "Binary");
    // console.log(addr);

    // contract call
    let exec_res = await exec(addr, "test_call_dummy_method");
    // console.log(exec_res);
    let exec_res2 = await exec(addr, "dummy_method");
    // console.log(exec_res);
    expect(exec_res.response.ret).toBe(exec_res2.response.ret);

    // invoke pvm_service_call failed
    exec_res = await exec(addr, "test_service_call_read_fail");
    // console.log(exec_res);
    expect(
      exec_res.response.ret.includes(
        "[ProtocolError] Kind: Service Error: CkbVm(EcallError"
      )
    ).toBe(true);
    expect(exec_res.response.ret.includes("NotFoundMethod")).toBe(true);

    // invoke pvm_service_read success
    exec_res = await exec(addr, "test_service_read");
    // console.log(exec_res);
    expect(exec_res.response.isError).toBe(false);

    // transfer via asset service
    let b = await getBalance(fee_asset_id, addr);
    expect(JSON.parse(b.ret).balance).toBe(0);
    const amount = 10000;
    const transfer_receipt = await transfer(admin, fee_asset_id, addr, amount);
    b = await getBalance(fee_asset_id, addr);
    expect(JSON.parse(b.ret).balance).toBe(10000);
    const to_addr = "0x0000000000000000000000000000000000000001";
    b = await getBalance(fee_asset_id, to_addr);
    const to_balance_before = JSON.parse(b.ret).balance;
    // transfer 100 from contract to to_addr via contract
    exec_res = await exec(addr, "test_transfer_from_contract");
    // console.log(exec_res);
    expect(exec_res.response.isError).toBe(false);
    b = await getBalance(fee_asset_id, to_addr);
    const to_balance_after = JSON.parse(b.ret).balance;
    expect(to_balance_before + 100).toBe(to_balance_after);
    b = await getBalance(fee_asset_id, addr);
    expect(JSON.parse(b.ret).balance).toBe(9900);
  });

  test("test_riscv_invalid_contract", async () => {
    const code = str2hex("invalid contract");
    try {
      const addr = await deploy(code, "invalid params", "Binary");
      expect(true).toBe(false);
    } catch (err) {
      expect(err.response.ret).toBe(
        "[ProtocolError] Kind: Service Error: CkbVm(ParseError)"
      );
    }
  });
});
