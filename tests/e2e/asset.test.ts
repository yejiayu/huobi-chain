/* eslint-env node, jest */
import { Account } from '@mutadev/account';
import { Client } from '@mutadev/client';
import { BigNumber } from '@mutadev/shared';
import { Address } from '@mutadev/types';
import { AssetService } from 'huobi-chain-sdk';
import { genRandomString, genRandomAccount } from './utils';

const account = Account.fromPrivateKey(
  '0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2',
);
const native_asset_id = "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c";
const basic_fee = 51000;

const client = new Client({
  defaultCyclesLimit: '0xffffffff',
});
const assetService = new AssetService(client, account);

async function create_asset(service = assetService, expectCode = 0, relayable = false, nameLen = 20, symbolLen = 8, supply = 0xfffffffffff, precision = 18) {
  const name = genRandomString('c', nameLen);
  const symbol = genRandomString('S', symbolLen);
  const res0 = await service.write.create_asset({
    name,
    symbol,
    supply,
    precision,
    relayable,
  });
  const code = Number(res0.response.response.code);
  expect(Number(res0.response.response.code)).toBe(expectCode);

  if(code == 0) {
    expect(Number(res0.cyclesUsed)).toBe(basic_fee);
    const asset = res0.response.response.succeedData;
    expect(asset.name).toBe(name);
    expect(asset.symbol).toBe(symbol);
    expect(asset.supply).toBe(supply);
    expect(asset.precision).toBe(precision);
    expect(asset.relayable).toBe(relayable);

    const asset_id = asset.id;
    const res1 = await service.read.get_asset({ id : asset_id });
    expect(Number(res1.code)).toBe(0);
    const data = res1.succeedData;
    expect(data.name).toBe(name);
    expect(data.symbol).toBe(symbol);
    expect(data.supply).toBe(supply);
    expect(data.precision).toBe(precision);
    expect(data.relayable).toBe(relayable);

    return asset_id;
  } else {
    return 'null';
  }
}

async function get_supply(assetId: string) {
  const res = await assetService.read.get_asset({
    id: assetId,
  });
  return new BigNumber(res.succeedData.supply);
}

async function get_native_supply() {
  return await get_supply(native_asset_id);
}

async function get_balance(assetId: string, user: Address) {
  const res0 = await assetService.read.get_balance({
    asset_id: assetId,
    user,
  });
  expect(Number(res0.code)).toBe(0);
  expect(res0.succeedData.asset_id).toBe(assetId);
  expect(res0.succeedData.user).toBe(user);
  return new BigNumber(res0.succeedData.balance);
}

async function get_native_balance(user: Address) {
  return await get_balance(native_asset_id, user);
}

async function get_allowance(assetId: string, grantor: Address, grantee: Address) {
  const res0 = await assetService.read.get_allowance({
    asset_id: assetId,
    grantor,
    grantee,
  });
  expect(Number(res0.code)).toBe(0);
  return new BigNumber(res0.succeedData.value);
}

async function get_native_allowance(grantor: Address, grantee: Address) {
  return await get_allowance(native_asset_id, grantor, grantee);
}

async function transfer(assetId: string, to: Address, value: number, service = assetService, expectCode = 0) {
  const res = await service.write.transfer({
    asset_id: assetId,
    to,
    value,
    memo: 'transfer',
  });
  const code = Number(res.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res.cyclesUsed)).toBe(basic_fee);
}

async function native_transfer(to: Address, value: number, service = assetService, expectCode = 0) {
  return await transfer(native_asset_id, to, value, service, expectCode);
}

async function approve(assetId: string, to: Address, value: number, service = assetService, expectCode = 0) {
  const res = await service.write.approve({
    asset_id: assetId,
    to,
    value,
    memo: 'approve',
  });
  const code = Number(res.response.response.code);
  expect(Number(res.cyclesUsed)).toBe(basic_fee);
  expect(code).toBe(expectCode);
  if(code == 0) {
    const data = JSON.parse(res.events[0].data);
    expect(data.asset_id).toBe(assetId);
    expect(data.grantee).toBe(to);
    expect(data.value).toBe(value);
  }
}

async function native_approve(to: Address, value: number, service = assetService, expectCode = 0) {
  return await approve(native_asset_id, to, value, service, expectCode)
}

async function transfer_from(assetId: string, sender: Address, recipient: Address, value: number, service = assetService, expectCode = 0) {
  const res = await service.write.transfer_from({
    asset_id: assetId,
    sender,
    recipient,
    value,
    memo: 'transfer_from',
  });
  const code = Number(res.response.response.code);
  expect(Number(res.response.response.code)).toBe(expectCode);
  expect(Number(res.cyclesUsed)).toBe(basic_fee);
  if(code == 0) {
    const data = JSON.parse(res.events[0].data);
    expect(data.asset_id).toBe(assetId);
    expect(data.sender).toBe(sender);
    expect(data.recipient).toBe(recipient);
    expect(data.value).toBe(value);
  }
}

async function native_transfer_from(sender: Address, recipient: Address, value: number, service = assetService, expectCode = 0) {
  return await transfer_from(native_asset_id, sender, recipient, value, service, expectCode);
}

async function burn(assetId: string, amount: number, service = assetService, expectCode = 0) {
  const res1 = await service.write.burn({
    asset_id: assetId,
    amount,
    proof: '0x23311',
    memo: 'burn',
  });
  const code = Number(res1.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res1.cyclesUsed)).toBe(basic_fee);
  if(code == 0) {
    const data = JSON.parse(res1.events[0].data);
    expect(data.asset_id).toBe(assetId);
    expect(Number(data.amount)).toBe(amount);
  }
}

async function native_burn(amount: number, service = assetService, expectCode = 0) {
  return await burn(native_asset_id, amount, service, expectCode);
}

async function mint(assetId: string, to: Address, amount: number, service = assetService, expectCode = 0) {
  const res1 = await service.write.mint({
    asset_id: assetId,
    to,
    amount,
    proof: '0x23311',
    memo: 'mint',
  });
  const code = Number(res1.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res1.cyclesUsed)).toBe(basic_fee);
  if(code == 0) {
    const data = JSON.parse(res1.events[0].data);
    expect(data.asset_id).toBe(assetId);
    expect(data.to).toBe(to);
    expect(Number(data.amount)).toBe(amount);
  }
}

async function native_mint(to: Address, amount: number, service = assetService, expectCode = 0) {
  return await mint(native_asset_id, to, amount, service, expectCode);
}

async function relay(assetId: string, amount: number, service = assetService, expectCode = 0) {
  const res1 = await service.write.relay({
    asset_id: assetId,
    amount,
    proof: '0x23311',
    memo: 'burn',
  });
  const code = Number(res1.response.response.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    expect(Number(res1.cyclesUsed)).toBe(basic_fee + 21000);
  } else {
    expect(Number(res1.cyclesUsed)).toBe(basic_fee);
  }
  if(code == 0) {
    const data = JSON.parse(res1.events[0].data);
    expect(data.asset_id).toBe(assetId);
    expect(Number(data.amount)).toBe(amount);
  }
}

async function native_relay(amount: number, service = assetService, expectCode = 0) {
  return await relay(native_asset_id, amount, service, expectCode);
}

async function change_admin(addr: Address, service = assetService, expectCode = 0) {
  const res1 = await service.write.change_admin({
    addr,
  });
  expect(Number(res1.response.response.code)).toBe(expectCode);
  expect(Number(res1.cyclesUsed)).toBe(basic_fee);
}

describe('asset service API test via huobi-sdk-js', () => {
  test('test create_asset', async () => {
    await create_asset();
  });

  test('test transfer', async () => {
    const newAccount = genRandomAccount();
    const balance_before = await get_native_balance(newAccount.address);
    const value = 0xfffff;
    await native_transfer(newAccount.address, value);
    // check balance
    const balance_after = await get_native_balance(newAccount.address);
    expect(balance_after.minus(balance_before).eq(value)).toBe(true);
  });

  test('test approve and transfer_from', async () => {
    const account1 = genRandomAccount();
    const service1 = new AssetService(client, account1);
    const account2 = genRandomAccount();
    // transfer
    await native_transfer(account1.address, 0xffff1111);
    // approve
    const value0 = 0xfffff;
    await native_approve(account1.address, value0);
    // get_allowance
    const al_before = await get_native_allowance(account.address, account1.address);
    expect(al_before.minus(value0).eq(0)).toBe(true);
    // transfer_from
    const value1 = 0x65a41;
    await native_transfer_from(account.address, account2.address, value1, service1);
    // check balance
    const al_after = await get_native_allowance(account.address, account1.address);
    expect(al_before.minus(al_after).eq(value1)).toBe(true);
    const balance = await get_native_balance(account2.address);
    expect(balance.eq(value1)).toBe(true);
  });

  test('test mint', async () => {
    const newAccount = genRandomAccount();
    const newService = new AssetService(client, newAccount);
    // transfer
    const value = 0xfffffffff;
    await native_transfer(newAccount.address, value);
    // query balance
    const balance_before = await get_native_balance(newAccount.address);
    const supply_before = await get_native_supply();
    // mint
    const amount = 0x652a1fff;
    await native_mint(newAccount.address, amount, newService, 0x6d);
    await native_mint(newAccount.address, amount);
    // check balance
    const balance_after = await get_native_balance(newAccount.address);
    const supply_after = await get_native_supply();
    expect(balance_after.minus(balance_before).eq(amount)).toBe(true);
    expect(supply_after.minus(supply_before).eq(amount)).toBe(true);
  });

  test('test burn', async () => {
    const newAccount = genRandomAccount();
    const newService = new AssetService(client, newAccount);
    // transfer
    const value = 0xfffffffff;
    await native_transfer(newAccount.address, value);
    // query balance
    const balance_before = await get_native_balance(newAccount.address);
    const supply_before = await get_native_supply();
    // burn
    const amount = 0x652a1fff;
    await native_burn(amount, newService);
    // check balance
    const balance_after = await get_native_balance(newAccount.address);
    const supply_after = await get_native_supply();
    expect(balance_before.minus(balance_after).eq(amount)).toBe(true);
    expect(supply_before.minus(supply_after).eq(amount)).toBe(true);
  });

  test('test relay', async () => {
    const asset_id_1 = await create_asset();
    // test relay of unrelayable asset
    const amount = 0x3ab12451;
    await relay(asset_id_1, amount, assetService, 0x6f);
    // test relay of relayable asset
    await native_relay(amount);
  });

  test('test change_admin', async () => {
    const newAccount = genRandomAccount();
    const newService = new AssetService(client, newAccount);
    // transfer
    await native_transfer(newAccount.address, 0xfff26635);
    // change_admin
    await change_admin(newAccount.address, newService, 0x6d);
    // change_admin
    await change_admin(newAccount.address);
    // check mint, change_admin
    await change_admin(account.address, newService);
  });

  test('test drain transfer', async () => {
    const newAccount = genRandomAccount();
    // transfer
    const value = 0xfffff;
    await native_transfer(newAccount.address, value);
    // drain transfer
    const newService = new AssetService(client, newAccount);
    await native_transfer(account.address, value, newService);
  });
});