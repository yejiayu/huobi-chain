/* eslint-env node, jest */
import { readFileSync } from 'fs';
import { Muta } from '@mutadev/muta-sdk';
// eslint-disable-next-line
import { Account } from "@mutadev/account";
import { hexToNum } from '@mutadev/utils';
import { retry } from '@mutadev/client';
// eslint-disable-next-line import/extensions,import/no-unresolved
import { addFeeTokenToAccounts, getBalance, transfer } from './helper';
import {
  client,
  accounts,
  admin,
  feeAssetID,
  // eslint-disable-next-line
} from "./utils";

function strToHex(s: string) {
  return Buffer.from(s, 'utf8').toString('hex');
}

const deployAccount = Muta.accountFromPrivateKey(
  'd6ef93ed5d27327fd10349a75d3b7a91aa5c1d0f42994be10c1cb0e357e722f5',
);

class Receipt {
  code: Number;

  errorMessage: string;

  succeedData: string;

  constructor(succeedData: string, code: number, errorMessage: string) {
    this.succeedData = succeedData;
    this.errorMessage = errorMessage;
    this.code = code;
  }

  decode(): any {
    try {
      return JSON.parse(this.succeedData);
    } catch (err) {
      throw this.succeedData;
    }
  }
}

async function getReceipt(txHash: any) {
  const raw = await retry(() => client.getReceipt(txHash));

  return new Receipt(
    raw.response.response.succeedData,
    hexToNum(raw.response.response.code),
    raw.response.response.errorMessage,
  );
}

async function read(method: string, payload: any) {
  const res = await client.queryService({
    serviceName: 'riscv',
    method,
    payload: JSON.stringify(payload),
  });

  return JSON.parse(res.succeedData);
}

async function write(method: string, payload: any, account: Account) {
  const tx = await client.composeTransaction({
    serviceName: 'riscv',
    method,
    payload: JSON.stringify(payload),
    sender: account.address,
  });

  const stx = account.signTransaction(tx);
  const txHash = await client.sendTransaction(stx);
  const receipt = await getReceipt(txHash);

  if (receipt.code !== 0) {
    throw receipt;
  }

  return receipt;
}

async function deploy(
  code: any,
  initArgs: string,
  intpType: string,
  account: any = null,
) {
  const payload = {
    intp_type: intpType,
    init_args: initArgs,
    code: code.toString('hex'),
  };

  const receipt = await write('deploy', payload, account || deployAccount);
  return receipt.decode().address;
}

async function authorize(
  method: string,
  addressList: any,
  account: any = null,
) {
  return write(method, { addresses: addressList }, account || admin);
}

async function exec(address: string, args: string, account: any = null) {
  return write('exec', { address, args }, account || admin);
}

async function call(address: string, args: string) {
  return read('call', { address, args });
}

describe('riscv service', () => {
  beforeAll(async () => {
    const accountsToAddFee = accounts.map((a) => a.address);
    accountsToAddFee.push(deployAccount.address);
    await addFeeTokenToAccounts(accountsToAddFee);
  });

  test('test_riscv_deploy_auth', async () => {
    const acc = accounts[1];
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');

    // not authed
    try {
      await deploy(code, 'set k init', 'Binary', acc);
    } catch (err) {
      expect(err.errorMessage).toBe('Not authorized');
    }

    // check auth
    let authorized = await read('check_deploy_auth', {
      addresses: [acc.address, accounts[2].address],
    });
    expect(authorized.addresses).toStrictEqual([]);

    // grant deploy auth to account
    await authorize('grant_deploy_auth', [acc.address]);

    // check auth again
    authorized = await read('check_deploy_auth', {
      addresses: [acc.address, accounts[2].address],
    });
    expect(authorized.addresses).toStrictEqual([acc.address]);

    // deploy again
    await deploy(code, 'set k init', 'Binary', acc);

    // revoke auth
    await authorize('revoke_deploy_auth', [acc.address]);

    // check auth
    authorized = await read('check_deploy_auth', {
      addresses: [acc.address, accounts[2].address],
    });
    expect(authorized.addresses).toStrictEqual([]);
  });

  test('test_riscv_normal_process', async () => {
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');
    const address = await deploy(code, 'set k init', 'Binary');
    // approve contract
    await authorize('approve_contracts', [address]);

    expect(await call(address, 'get k')).toBe('init');
    await exec(address, 'set k v', admin);
    expect(await call(address, 'get k')).toBe('v');

    // get code
    const contract = await read('get_contract', {
      address,
      get_code: true,
      storage_keys: [Buffer.from('k', 'utf8').toString('hex'), '', '1a'],
    });

    expect(contract.code).toBe(code.toString('hex'));
    expect(contract.storage_values).toStrictEqual([
      Buffer.from('v', 'utf8').toString('hex'),
      '',
      '',
    ]);
  });

  test('test_service_call', async () => {
    const code = readFileSync('./riscv_contracts/contract_test');
    const address = await deploy(code, '', 'Binary');
    // approve contract
    await authorize('approve_contracts', [address]);

    // contract call
    const indirectDummy = (
      await exec(address, 'test_call_dummy_method')
    ).decode();
    const dummy = (await exec(address, 'dummy_method')).decode();
    expect(indirectDummy).toBe(dummy);

    // invoke pvm_service_call failed
    // try {
    //   await exec(address, 'test_service_call_read_fail');
    // } catch (err) {
    //   console.log(err);
    //   expect(err.errorMessage.includes('VM: InvalidEcall(')).toBe(true);
    // }

    // invoke pvm_service_read success
    await call(address, 'test_service_read');

    // transfer via asset service
    const balanceBefore = await getBalance(feeAssetID, address);
    expect(JSON.parse(balanceBefore.succeedData).balance).toBe(0);

    const amount = 10000;
    const transferReceipt = await transfer(admin, feeAssetID, address, amount);
    expect(hexToNum(transferReceipt.response.response.code)).toBe(0);

    const balanceAfter = await getBalance(feeAssetID, address);
    expect(JSON.parse(balanceAfter.succeedData).balance).toBe(10000);

    const recipientAddress = '0x0000000000000000000000000000000000000001';
    let recipientBalance = await getBalance(feeAssetID, recipientAddress);
    const recipientBalanceBefore = JSON.parse(recipientBalance.succeedData)
      .balance;

    // transfer 100 from contract to recipientAddress via contract
    await exec(address, 'test_transfer_from_contract');

    recipientBalance = await getBalance(feeAssetID, recipientAddress);
    const recipientBalanceAfter = JSON.parse(recipientBalance.succeedData)
      .balance;
    expect(recipientBalanceBefore + 100).toBe(recipientBalanceAfter);

    const contractBalance = await getBalance(feeAssetID, address);
    expect(JSON.parse(contractBalance.succeedData).balance).toBe(9900);
  });

  test('test_riscv_invalid_contract', async () => {
    const code = strToHex('invalid contract');

    try {
      await deploy(code, 'invalid params', 'Binary');
    } catch (err) {
      expect(err.errorMessage).toBe('VM: ParseError');
    }
  });
});
