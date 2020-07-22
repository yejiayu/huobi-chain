/* eslint-env node, jest */
import { readFileSync } from 'fs';
import { Address } from '@mutadev/types';
import { InterpreterType, RISCVService } from 'huobi-chain-sdk';
import { admin, client, genRandomAccount, get_balance, transfer } from './utils';

const riscvService = new RISCVService(client, admin);

async function deploy(code: string, initArgs: string, service = riscvService, expectCode = 0) {
  const res0 = await service.write.deploy({
    code,
    intp_type: InterpreterType.Binary,
    init_args: initArgs,
  });
  const resCode = Number(res0.response.response.code);
  expect(resCode).toBe(expectCode);
  if(resCode == 0) {
    return res0.response.response.succeedData.address;
  } else {
    return ''
  }
}

async function check_deploy_auth(address: Address, is_exist: boolean) {
  const res0 = await riscvService.read.check_deploy_auth({
    addresses: [ address ],
  });
  expect(Number(res0.code)).toBe(0);
  const addresses = res0.succeedData.addresses;
  if(is_exist) {
    expect(addresses.indexOf(address)).not.toBe(-1);
  } else {
    expect(addresses.indexOf(address)).toBe(-1);
  }
}

async function grant_deploy_auth(address: Address, service = riscvService, expectCode = 0) {
  const res0 = await service.write.grant_deploy_auth({
    addresses: [ address ],
  });
  expect(Number(res0.response.response.code)).toBe(expectCode);
}

async function revoke_deploy_auth(address: Address, service = riscvService, expectCode = 0) {
  const res0 = await service.write.revoke_deploy_auth({
    addresses: [ address ],
  });
  expect(Number(res0.response.response.code)).toBe(expectCode);
}

async function call(contractAddress: Address, args: string, service = riscvService, expectCode = 0) {
  const res0 = await service.read.call({
    address: contractAddress,
    args,
  });
  const code = Number(res0.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    return res0.succeedData;
  } else {
    return '';
  }
}

async function exec(contractAddress: Address, args: string, service = riscvService, expectCode = 0) {
  const res0 = await service.write.exec({
    address: contractAddress,
    args,
  });
  expect(Number(res0.response.response.code)).toBe(expectCode);
}

async function approve_contracts(contractAddress: Address, service = riscvService, expectCode = 0) {
  const res0 = await service.write.approve_contracts({
    addresses: [ contractAddress ],
  });
  expect(Number(res0.response.response.code)).toBe(expectCode);
}

async function revoke_contracts(contractAddress: Address, service = riscvService, expectCode = 0) {
  const res0 = await service.write.revoke_contracts({
    addresses: [ contractAddress ],
  });
  expect(Number(res0.response.response.code)).toBe(expectCode);
}

async function get_contract(contractAddress: Address, getCode: boolean, storageKeys: Array<string>) {
  const res0 = await riscvService.read.get_contract({
    address: contractAddress,
    get_code: getCode,
    storage_keys: storageKeys,
  });
  expect(Number(res0.code)).toBe(0);
  return res0.succeedData;
}

describe('riscv service', () => {
  test('test_deploy_auth', async () => {
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');
    const account = genRandomAccount();
    const newService = new RISCVService(client, account);
    await transfer(account.address, 9999999);
    // deploy before auth
    await deploy(code.toString('hex'), 'set k init', newService, 0x6d);
    // auth
    await check_deploy_auth(account.address, false);
    await grant_deploy_auth(account.address);
    await check_deploy_auth(account.address, true);
    // deploy after auth
    await deploy(code.toString('hex'), 'set k init', newService);
    // revoke
    await revoke_deploy_auth(account.address);
    // deploy after revoke
    await deploy(code.toString('hex'), 'set k init', newService, 0x6d);
    await check_deploy_auth(account.address, false);
  });

  test('test_contract_auth', async () => {
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');
    const account = genRandomAccount();
    const newService = new RISCVService(client, account);
    await transfer(account.address, 9999999);
    await grant_deploy_auth(account.address);
    const contractAddress = await deploy(code.toString('hex'), 'set k init', newService);
    // before auth
    await call(contractAddress, 'get k', riscvService, 0x6d);
    await exec(contractAddress, 'set k v', riscvService, 0x6d);
    // auth
    await approve_contracts(contractAddress);
    await call(contractAddress, 'get k');
    await exec(contractAddress, 'set k v');
    // revoke
    await revoke_contracts(contractAddress);
    await call(contractAddress, 'get k', riscvService, 0x6d);
    await exec(contractAddress, 'set k v', riscvService, 0x6d);
  });

  test('test_normal_process', async () => {
    const code = readFileSync('../../services/riscv/src/tests/simple_storage');
    const account = genRandomAccount();
    const newService = new RISCVService(client, account);
    await transfer(account.address, 9999999);
    await grant_deploy_auth(account.address);
    const contractAddress = await deploy(code.toString('hex'), 'set k init', newService);
    await approve_contracts(contractAddress);

    const res0 = await call(contractAddress, 'get k');
    expect(res0).toBe('init');
    await exec(contractAddress, 'set k v', newService);
    const res1 = await call(contractAddress, 'get k');
    expect(res1).toBe('v');
    // get code
    const contract = await get_contract(contractAddress, true, [Buffer.from('k', 'utf8').toString('hex'), '', '1a']);
    expect(contract.code).toBe(code.toString('hex'));
    expect(contract.storage_values).toStrictEqual([
      Buffer.from('v', 'utf8').toString('hex'),
      '',
      '',
    ]);
  });

  test('test_service_call', async () => {
    const code = readFileSync('./riscv_contracts/contract_test');
    await grant_deploy_auth(admin.address);
    const contractAddress = await deploy(code.toString('hex'), '');
    await approve_contracts(contractAddress);
    // contract call
    await exec(contractAddress, 'test_service_call_read_fail', riscvService, 0x2);
    await call(contractAddress, 'test_service_read');
    // transfer to contract
    const amount = 0x768762;
    await transfer(contractAddress, amount);
    const balance = await get_balance(contractAddress);
    expect(balance.minus(amount).eq(0)).toBe(true);

    const recipientAddress = '0x0000000000000000000000000000000000000001';
    const balance_before = await get_balance(recipientAddress);
    // transfer 100 from contract to recipientAddress via contract
    await exec(contractAddress, 'test_transfer_from_contract');
    const balance_after = await get_balance(recipientAddress);
    expect(balance_after.minus(balance_before).eq(100)).toBe(true);
  });

  test('test_riscv_invalid_contract', async () => {
    const code = Buffer.from('invalid contract', 'utf8').toString('hex')
    await grant_deploy_auth(admin.address);
    await deploy(code, 'invalid params', riscvService, 0x69);
  });
});
