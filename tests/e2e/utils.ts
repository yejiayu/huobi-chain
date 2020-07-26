import { parse } from 'toml';
import { find } from 'lodash';
import { readFileSync } from 'fs';
import { utils } from '@mutadev/muta-sdk';
import { Client } from '@mutadev/client';
import { Account } from '@mutadev/account';
import { BigNumber } from '@mutadev/shared';
import { AssetService, InterpreterType, RISCVService } from 'huobi-chain-sdk';

const { hexToNum } = utils;

const ADMIN_PRIVATE_KEY = '0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2';

const client = new Client({
  defaultCyclesLimit: '0xffffffff',
});

const admin: Account = Account.fromPrivateKey(ADMIN_PRIVATE_KEY);
const nativeAssetId = "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c";
const randomString = require("randomstring");
const genesis = parse(readFileSync('./genesis.toml', 'utf-8'));

const governance = JSON.parse(
  find(genesis.services, (s) => s.name === 'governance').payload,
);

export function genRandomString(prefix: String = 'r', length: number = 12) {
  expect(prefix.length <= length);
  return prefix + randomString.generate(length - prefix.length);
}

export function genRandomStrings(size: number = 3, prefix: String = 't', length: number = 12) {
  const names = new Array(0);

  for(var i = 0; i < size; i++) {
    names.push(genRandomString(prefix, length));
  }

  return names;
}

export function genRandomAccount() {
  const randomPriKey = randomString.generate({
    charset: '0123456789abcdef',
    length: 64,
  });
  return Account.fromPrivateKey('0x' + randomPriKey);
}

export function genRandomInt(min = 0x0, max = 0xfffffffff) {
  min = Math.ceil(min);
  max = Math.floor(max);
  return Math.floor(Math.random() * (max - min)) + min;
}

export async function transfer(to: string, value: number) {
  const service = new AssetService(client, admin);
  await service.write.transfer({
    asset_id: nativeAssetId,
    to,
    value,
    memo: 'transfer',
  });
}

export async function get_balance(user: string) {
  const service = new AssetService(client, admin);
  const res0 = await service.read.get_balance({
    asset_id: nativeAssetId,
    user,
  });
  return new BigNumber(res0.succeedData.balance);
}

export async function deploy(code: string, initArgs: string) {
  const service = new RISCVService(client, admin);
  const res0 = await service.write.grant_deploy_auth({
    addresses: [ admin.address ],
  });
  expect(Number(res0.response.response.code)).toBe(0);

  const res1 = await service.write.deploy({
    code,
    intp_type: InterpreterType.Binary,
    init_args: initArgs,
  });
  expect(Number(res1.response.response.code)).toBe(0);

  const contractAddress = res1.response.response.succeedData.address;
  const res2 = await service.write.approve_contracts({
    addresses: [ contractAddress ],
  });
  expect(Number(res2.response.response.code)).toBe(0);
  return contractAddress;
}

export {
  admin, client, governance, hexToNum, nativeAssetId
};
