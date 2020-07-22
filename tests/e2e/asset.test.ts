/* eslint-env node, jest */
import { BigNumber } from '@mutadev/shared';
import { Address } from '@mutadev/types';
import { AssetService } from 'huobi-chain-sdk';
import { admin, client, genRandomString, genRandomAccount, nativeAssetId } from './utils';

const assetService = new AssetService(client, admin);

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
  return await get_supply(nativeAssetId);
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
  return await get_balance(nativeAssetId, user);
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
  return await get_allowance(nativeAssetId, grantor, grantee);
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
}

async function native_transfer(to: Address, value: number, service = assetService, expectCode = 0) {
  return await transfer(nativeAssetId, to, value, service, expectCode);
}

async function approve(assetId: string, to: Address, value: number, service = assetService, expectCode = 0) {
  const res = await service.write.approve({
    asset_id: assetId,
    to,
    value,
    memo: 'approve',
  });
  const code = Number(res.response.response.code);
  expect(code).toBe(expectCode);
}

async function native_approve(to: Address, value: number, service = assetService, expectCode = 0) {
  return await approve(nativeAssetId, to, value, service, expectCode)
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
  expect(code).toBe(expectCode);
}

async function native_transfer_from(sender: Address, recipient: Address, value: number, service = assetService, expectCode = 0) {
  return await transfer_from(nativeAssetId, sender, recipient, value, service, expectCode);
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
}

async function native_burn(amount: number, service = assetService, expectCode = 0) {
  return await burn(nativeAssetId, amount, service, expectCode);
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
}

async function native_relay(amount: number, service = assetService, expectCode = 0) {
  return await relay(nativeAssetId, amount, service, expectCode);
}

async function change_admin(addr: Address, service = assetService, expectCode = 0) {
  const res1 = await service.write.change_admin({
    addr,
  });
  expect(Number(res1.response.response.code)).toBe(expectCode);
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
    const al_before = await get_native_allowance(admin.address, account1.address);
    expect(al_before.minus(value0).eq(0)).toBe(true);
    // transfer_from
    const value1 = 0x65a41;
    await native_transfer_from(admin.address, account2.address, value1, service1);
    // check balance
    const al_after = await get_native_allowance(admin.address, account1.address);
    expect(al_before.minus(al_after).eq(value1)).toBe(true);
    const balance = await get_native_balance(account2.address);
    expect(balance.eq(value1)).toBe(true);
  });

  test('test mint', async () => {
    const newAccount = genRandomAccount();
    const newService = new AssetService(client, newAccount);
    // transfer
    const value = 0xfffffff;
    await native_transfer(newAccount.address, value);
    // create_asset
    const assetId = await create_asset(newService);
    // unauthorized mint
    const amount = 0x652a1fff;
    await mint(assetId, newAccount.address, amount, assetService, 0x6d);

    const balance_before = await get_balance(assetId, admin.address);
    const supply_before = await get_supply(assetId);
    // mint
    await mint(assetId, admin.address, amount, newService);
    // check balance
    const balance_after = await get_balance(assetId, admin.address);
    expect(balance_after.minus(balance_before).eq(amount)).toBe(true);
    const supply_after = await get_supply(assetId);
    expect(supply_after.minus(supply_before).eq(amount)).toBe(true);
  });

  test('test burn', async () => {
    const newAccount = genRandomAccount();
    const newService = new AssetService(client, newAccount);
    // transfer
    const value = 0xffffffff;
    await native_transfer(newAccount.address, value);
    const supply_before = await get_native_supply();
    // burn
    const amount = 0x652a1fff;
    await native_burn(amount, newService);
    const supply_after = await get_native_supply();
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
    // change_admin
    await change_admin(admin.address, newService);
  });

  test('test drain transfer', async () => {
    const newAccount = genRandomAccount();
    // transfer
    const value = 0xfffff;
    await native_transfer(newAccount.address, value);
    // drain transfer
    const newService = new AssetService(client, newAccount);
    await native_transfer(admin.address, value, newService, 0x66);
  });
});