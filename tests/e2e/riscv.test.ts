/* eslint-env node, jest */
import { readFileSync } from 'fs';
import { Muta } from 'muta-sdk';
import { hexToNum } from '@mutajs/utils';
// eslint-disable-next-line
import { addFeeTokenToAccounts, getBalance, transfer } from './helper';
import {
  client,
  accounts,
  admin,
  feeAssetID,
// eslint-disable-next-line
} from './utils';

function strToHex(s) {
  return Buffer.from(s, 'utf8').toString('hex');
}

const account = Muta.accountFromPrivateKey(
  'd6ef93ed5d27327fd10349a75d3b7a91aa5c1d0f42994be10c1cb0e357e722f5',
);

async function deploy(code, initArgs, intpType, acc = null) {
  const accountToSign = acc || account;
  const tx = await client.composeTransaction({
    method: 'deploy',
    payload: {
      intp_type: intpType,
      init_args: initArgs,
      code: code.toString('hex'),
    },
    serviceName: 'riscv',
  });

  const txHash = await client.sendTransaction(
    accountToSign.signTransaction(tx),
  );

  const receipt = await client.getReceipt(txHash);

  try {
    const addr = JSON.parse(receipt.response.response.succeedData).address;
    return addr;
  } catch (err) {
    throw receipt;
  }
}

async function query(address, args) {
  const res = await client.queryService({
    serviceName: 'riscv',
    method: 'call',
    payload: JSON.stringify({
      address,
      args,
    }),
  });

  return JSON.parse(res.succeedData);
}

async function exec(address, args) {
  const payload = {
    address,
    args,
  };

  const tx = await client.composeTransaction({
    payload,
    serviceName: 'riscv',
    method: 'exec',
  });

  const txHash = await client.sendTransaction(
    account.signTransaction(tx),
  );

  const receipt = await client.getReceipt(txHash);

  return receipt;
}

describe('riscv service', () => {
  beforeAll(async () => {
    const accountsToAddFee = accounts.map((a) => a.address);
    accountsToAddFee.push(account.address);
    await addFeeTokenToAccounts(accountsToAddFee);
  });

  test('test_riscv_deploy_auth', async () => {
    const acc = accounts[1];
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');

    // not authed
    try {
      await deploy(code, 'set k init', 'Binary', acc);
    } catch (err) {
      expect(err.response.ret).toBe(
        '[ProtocolError] Kind: Service Error: NonAuthorized',
      );
    }

    // check auth
    let deployAuthResp = await client.queryService({
      serviceName: 'riscv',
      method: 'check_deploy_auth',
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address],
      }),
    });

    expect(hexToNum(deployAuthResp.code)).toBe(0);
    expect(JSON.parse(deployAuthResp.succeedData).addresses).toStrictEqual([]);

    // grant deploy auth to account
    let tx = await client.composeTransaction({
      method: 'grant_deploy_auth',
      payload: {
        addresses: [acc.address],
      },
      serviceName: 'riscv',
    });

    let txHash = await client.sendTransaction(admin.signTransaction(tx));
    let receipt = await client.getReceipt(txHash);

    await deploy(code, 'set k init', 'Binary', acc);

    // check auth
    deployAuthResp = await client.queryService({
      serviceName: 'riscv',
      method: 'check_deploy_auth',
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address],
      }),
    });
    expect(hexToNum(deployAuthResp.code)).toBe(0);
    expect(JSON.parse(deployAuthResp.succeedData).addresses).toStrictEqual([
      acc.address,
    ]);

    // revoke auth
    tx = await client.composeTransaction({
      method: 'revoke_deploy_auth',
      payload: {
        addresses: [acc.address],
      },
      serviceName: 'riscv',
    });
    txHash = await client.sendTransaction(admin.signTransaction(tx));
    receipt = await client.getReceipt(txHash);
    expect(hexToNum(receipt.response.response.code)).toBe(0);

    // check auth
    deployAuthResp = await client.queryService({
      serviceName: 'riscv',
      method: 'check_deploy_auth',
      payload: JSON.stringify({
        addresses: [acc.address, accounts[2].address],
      }),
    });
    expect(hexToNum(deployAuthResp.code)).toBe(0);
    expect(JSON.parse(deployAuthResp.succeedData).addresses).toStrictEqual([]);
  });

  test('test_riscv_normal_process', async () => {
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');
    const addr = await deploy(code, 'set k init', 'Binary');
    const vInit = await query(addr, 'get k');
    expect(vInit).toBe('init');
    const execResp = await exec(addr, 'set k v');
    expect(hexToNum(execResp.response.response.code)).toBe(0);
    const v1 = await query(addr, 'get k');
    expect(v1).toBe('v');

    // get code
    const getContractResp = await client.queryService({
      serviceName: 'riscv',
      method: 'get_contract',
      payload: JSON.stringify({
        address: addr,
        get_code: true,
        storage_keys: [Buffer.from('k', 'utf8').toString('hex'), '', '1a'],
      }),
    });
    expect(hexToNum(getContractResp.code)).toBeFalsy();
    const ret = JSON.parse(getContractResp.succeedData);
    expect(ret.code).toBe(code.toString('hex'));
    expect(ret.storage_values).toStrictEqual([
      Buffer.from('v', 'utf8').toString('hex'),
      '',
      '',
    ]);
  });

  test('test_service_call', async () => {
    const code = readFileSync('./riscv_contracts/contract_test');
    const addr = await deploy(code, '', 'Binary');

    // contract call
    let execResp = await exec(addr, 'test_call_dummy_method');
    const execResp2 = await exec(addr, 'dummy_method');
    expect(execResp.response.response.succeedData).toBe(execResp2.response.response.succeedData);

    // invoke pvm_service_call failed
    execResp = await exec(addr, 'test_service_call_read_fail');
    expect(
      execResp.response.response.errorMessage.includes(
        'VM: EcallError(',
      ),
    ).toBe(true);
    expect(execResp.response.response.errorMessage.includes('not found method')).toBe(true);

    // invoke pvm_service_read success
    execResp = await exec(addr, 'test_service_read');
    expect(hexToNum(execResp.response.response.code)).toBe(0);

    // transfer via asset service
    let b = await getBalance(feeAssetID, addr);
    expect(JSON.parse(b.succeedData).balance).toBe(0);
    const amount = 10000;
    const transferReceipt = await transfer(admin, feeAssetID, addr, amount);
    expect(hexToNum(transferReceipt.response.response.code)).toBe(0);
    b = await getBalance(feeAssetID, addr);
    expect(JSON.parse(b.succeedData).balance).toBe(10000);
    const recipientAddress = '0x0000000000000000000000000000000000000001';
    b = await getBalance(feeAssetID, recipientAddress);
    const recipientBalanceBefore = JSON.parse(b.succeedData).balance;
    // transfer 100 from contract to recipientAddress via contract
    execResp = await exec(addr, 'test_transfer_from_contract');
    expect(hexToNum(execResp.response.response.code)).toBe(0);
    b = await getBalance(feeAssetID, recipientAddress);
    const recipientBalanceAfter = JSON.parse(b.succeedData).balance;
    expect(recipientBalanceBefore + 100).toBe(recipientBalanceAfter);
    b = await getBalance(feeAssetID, addr);
    expect(JSON.parse(b.succeedData).balance).toBe(9900);
  });

  test('test_riscv_invalid_contract', async () => {
    const code = strToHex('invalid contract');
    try {
      await deploy(code, 'invalid params', 'Binary');
    } catch (err) {
      expect(err.response.response.errorMessage).toBe(
        'VM: ParseError',
      );
    }
  });
});
