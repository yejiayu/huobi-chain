/* eslint-env node, jest */
import { readFileSync } from 'fs';
import { GovernanceService, RISCVService } from 'huobi-chain-sdk';
import { admin, client, deploy, genRandomAccount, genRandomInt, get_balance, governance, transfer } from './utils';

const governanceService = new GovernanceService(client, admin);

const txFailureFee = governance.info.tx_failure_fee;
const txFloorFee = governance.info.tx_floor_fee;
const profitDeductRate = governance.info.profit_deduct_rate_per_million;
const txFeeDiscount: Array<any> = governance.info.tx_fee_discount;

function cal_profit(balance: number, actualFee: number) {
  const discountFee = Math.max(actualFee, txFloorFee);
  const discount = txFeeDiscount.filter(discount => discount.threshold <= balance).sort((d1, d2) => d1 - d2);
  let fee = discountFee;
  if(discount.length > 0) {
    fee = Math.ceil(fee * 100 / discount[0].discount_percent);
  }
  return Math.ceil(fee * 1000_000 / profitDeductRate);
}

function cal_fee(balance: number, accumulatedProfit: number) {
  const fee = Math.floor(accumulatedProfit * profitDeductRate / 1000_000);
  const discount = txFeeDiscount.filter(discount => discount.threshold <= balance).sort((d1, d2) => d1 - d2);
  let discountFee = fee;
  if(discount.length > 0) {
    discountFee = Math.floor(discountFee * discount[0].discount_percent / 100);
  }
  return Math.max(discountFee, txFloorFee);
}

async function accumulate_profit(accumulatedProfit: number, service = governanceService, expectCode = 0) {
  const res0 = await service.write.accumulate_profit({
    address: '0xd2d268749ffe54def4e2e73e5e06a4ebf0d6f585',  // any address is ok
    accumulated_profit: accumulatedProfit,
  });
  const resCode = Number(res0.response.response.code);
  expect(resCode).toBe(expectCode);
}

async function test_accumulate_profit(initBalance: number, accumulatedProfit: number, expectFee: number, expectCode = 0) {
  const account = genRandomAccount();
  const service = new GovernanceService(client, account);

  await transfer(account.address, initBalance);
  const balance_before = await get_balance(account.address);
  expect(balance_before.eq(initBalance)).toBe(true);

  await accumulate_profit(accumulatedProfit, service, expectCode);

  const balance_after = await get_balance(account.address);
  expect(balance_before.minus(balance_after).eq(expectFee)).toBe(true);
}

describe('governance service', () => {
  test('test_accumulate_profit', async () => {
    // balance < tx_failure_fee, this transaction will be blocked by mempool
    // const initBalance = genRandomInt(0, txFailureFee);
    // const profit = genRandomInt();
    // await test_accumulate_profit(initBalance, profit, 0);

    // tx_failure_fee <= balance < actual_fee
    const actualFee0 = genRandomInt(txFailureFee + 1);
    const initBalance0 = genRandomInt(txFailureFee, actualFee0);
    const profit0 = cal_profit(initBalance0, actualFee0);
    await test_accumulate_profit(initBalance0, profit0, txFailureFee);

    // actual_fee <= balance
    const actualFee1 = genRandomInt(txFailureFee + 1);
    const initBalance1 = genRandomInt(actualFee1);
    const profit1 = cal_profit(initBalance1, actualFee1);
    await test_accumulate_profit(initBalance1, profit1, actualFee1);
  });

  test('test_contract_call_accumulate_profit', async () => {
    const code = readFileSync('./riscv_contracts/contract_test');
    const contractAddress = await deploy(code.toString('hex'), '');

    const account = genRandomAccount();
    const fee = cal_fee(0, 876544545);
    const initBalance = genRandomInt(Math.max(txFailureFee, fee));
    const actualFee = cal_fee(initBalance, 876544545);

    await transfer(account.address, initBalance);
    const balance_before = await get_balance(account.address);
    expect(balance_before.eq(initBalance)).toBe(true);

    const service = new RISCVService(client, account);
    const res = await service.write.exec({address: contractAddress, args: 'test_accumulate_profits_contract'});
    expect(Number(res.response.response.code)).toBe(0);

    const balance_after = await get_balance(account.address);
    expect(balance_before.minus(balance_after).eq(actualFee)).toBe(true);
  });
});