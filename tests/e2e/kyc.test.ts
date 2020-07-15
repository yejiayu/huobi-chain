import { Account } from '@mutadev/account';
import { Client } from '@mutadev/client';
import { Address } from '@mutadev/types';
import { AssetService, KycService } from 'huobi-chain-sdk';
import { genRandomString, genRandomStrings, genRandomAccount } from './utils';

const account = Account.fromPrivateKey(
  '0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2',
);
const client = new Client({
  defaultCyclesLimit: '0xffffffff',
});
const kycService = new KycService(client, account);

const basic_fee = 51000;

async function register_org(service = kycService, expectCode = 0, nameLen = 12, tagNum = 3, tagLen = 12) {
  const orgName = genRandomString('', nameLen);

  // pre-check
  const res0 = await service.read.get_org_info(orgName);
  expect(Number(res0.code)).toBe(0x67);

  const res01 = await service.read.get_orgs();
  expect(Number(res01.code)).toBe(0);
  expect(res01.succeedData.indexOf(orgName)).toBe(-1);

  const res02 = await service.read.get_org_supported_tags(orgName);
  expect(Number(res02.code)).toBe(0x67);

  // register org
  const description = genRandomString('d', 50);
  const supportedTags = genRandomStrings(tagNum, 'r', tagLen);
  const res1 = await service.write.register_org({
    name: orgName,
    description,
    admin: account.address,
    supported_tags: supportedTags,
  });
  let code = Number(res1.response.response.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    expect(Number(res1.cyclesUsed)).toBe(basic_fee + (nameLen + 50 + 20) * 1000 + tagNum * 10000);
  } else {
    expect(Number(res1.cyclesUsed)).toBe(basic_fee);
  }

  // post-check
  if(code == 0) {
    const res2 = await service.read.get_org_info(orgName);
    const data2 = res2.succeedData;
    expect(Number(res2.code)).toBe(0);
    expect(data2.name).toBe(orgName);
    expect(data2.description).toBe(description);
    expect(data2.admin).toBe(account.address);
    expect(JSON.stringify(data2.supported_tags)).toBe(JSON.stringify(supportedTags));
    expect(data2.approved).toBe(false);

    const res3 = await service.read.get_orgs();
    expect(Number(res3.code)).toBe(0);
    expect(res3.succeedData.indexOf(orgName)).not.toBe(-1);

    const res4 = await service.read.get_org_supported_tags(orgName);
    expect(Number(res4.code)).toBe(0);
    expect(JSON.stringify(res4.succeedData)).toBe(JSON.stringify(supportedTags));
  }

  return { 'org_name': orgName, 'tags': supportedTags };
}

async function approve(orgName: string, approved = true, service = kycService, expectCode = 0) {
  const res0 = await service.write.change_org_approved({
    org_name: orgName,
    approved,
  });

  let code = Number(res0.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res0.cyclesUsed)).toBe(basic_fee);

  if(code == 0) {
    const data = JSON.parse(res0.events[0].data);
    expect(data.org_name).toBe(orgName);
    expect(data.approved).toBe(approved);

    const res1 = await service.read.get_org_info(orgName);
    expect(res1.succeedData.approved).toBe(approved);
  }
}

async function update_supported_tags(orgName: string, service = kycService, expectCode = 0, tagNum = 3, tagLen = 12) {
  const newSupportedTags = genRandomStrings(tagNum, 'r', tagLen);
  const res0 = await service.write.update_supported_tags({
    org_name: orgName,
    supported_tags: newSupportedTags,
  });
  let code = Number(res0.response.response.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    expect(Number(res0.cyclesUsed)).toBe(basic_fee + tagNum * 10000);
  } else {
    expect(Number(res0.cyclesUsed)).toBe(basic_fee);
  }

  if(code == 0) {
    const data = JSON.parse(res0.events[0].data);
    expect(data.org_name).toBe(orgName);
    expect(JSON.stringify(data.supported_tags)).toBe(JSON.stringify(newSupportedTags));

    const res2 = await service.read.get_org_info(orgName);
    expect(JSON.stringify(res2.succeedData.supported_tags)).toBe(JSON.stringify(newSupportedTags));
  }

  return newSupportedTags;
}

async function update_user_tags(orgName: string, supportedTags: Array<string>, service = kycService, expectCode = 0, valNum = 3, valLen = 12) {
  const user = genRandomAccount().address;

  let tags = <Record<string, Array<string>>>{};
  supportedTags.map(tag => {
    tags[tag] = genRandomStrings(valNum, '', valLen);
  });

  const res0 = await service.write.update_user_tags({
    org_name: orgName,
    user,
    tags,
  });
  const code = Number(res0.response.response.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    expect(Number(res0.cyclesUsed)).toBe(basic_fee + (supportedTags.length + 1) * valNum * 10000);
  } else {
    expect(Number(res0.cyclesUsed)).toBe(basic_fee);
  }

  if(code == 0) {
    const data = JSON.parse(res0.events[0].data);
    expect(data.org_name).toBe(orgName);
    expect(data.user).toBe(user);
    expect(data.tags.length).toBe(tags.length);
    for (const k in data.tags) {
      expect(JSON.stringify(data.tags[k])).toBe(JSON.stringify(tags[k]));
    }

    const res1 = await service.read.get_user_tags({
      org_name: orgName,
      user,
    });
    expect(Number(res1.code)).toBe(0);
    expect(res1.succeedData.length).toBe(tags.length);
    for (const k in res1.succeedData) {
      expect(JSON.stringify(res1.succeedData[k])).toBe(JSON.stringify(tags[k]));
    }
  }

  return { 'user': user, 'values': tags };
}

async function change_service_admin(newAdmin: Address, service = kycService, expectCode = 0) {
  const res0 = await service.write.change_service_admin({
    new_admin: newAdmin,
  });
  const code = Number(res0.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res0.cyclesUsed)).toBe(basic_fee);
}

async function change_org_admin(orgName: string, newAdmin: Address, service = kycService, expectCode = 0) {
  const res0 = await service.write.change_org_admin({
    name: orgName,
    new_admin: newAdmin,
  });
  const code = Number(res0.response.response.code);
  expect(code).toBe(expectCode);
  expect(Number(res0.cyclesUsed)).toBe(basic_fee);
}

async function eval_user_tag_expression(user: Address, expression: string, expectCode = 0, result = true) {
  const res = await kycService.read.eval_user_tag_expression({
    user,
    expression,
  });
  const code = Number(res.code);
  expect(code).toBe(expectCode);
  if(code == 0) {
    expect(res.succeedData).toBe(result);
  }
}

describe('kyc service API test via huobi-sdk-js', () => {
  test('test register_org', async () => {
    await register_org();
  });

  test('test change_org_approved', async () => {
    // register org
    const res = await register_org();
    const orgName = res['org_name'];
    // approve
    await approve(orgName, true);
    // disapprove
    await approve(orgName, false);
  });

  test('test update_supported_tags', async () => {
    // register org
    let res = await register_org();
    const orgName = res['org_name'];
    // update supported tags
    await update_supported_tags(orgName);
  });

  test('test update_user_tags', async () => {
    // register org
    let res = await register_org();
    const orgName = res['org_name'];
    const tags = res['tags'];
    // update user tags before approved
    await update_user_tags(orgName, tags, kycService, 0x6c);
    // approve
    await approve(orgName, true);
    // update user tags after approved
    await update_user_tags(orgName, tags);
  });

  test('test change_service_admin', async () => {
    // register org
    let res = await register_org();
    const orgName = res['org_name'];
    // create new account and transfer coins
    const newAccount = genRandomAccount();
    const assetService = new AssetService(client, account);
    const res1 = await assetService.write.transfer({
      asset_id: '0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c',
      to: newAccount.address,
      value: 99999999999,
      memo: 'test',
    });
    expect(Number(res1.response.response.code)).toBe(0);
    // before change, check change_org_approved, change_service_admin, register_org, update_supported_tags
    const newService = new KycService(client, newAccount);
    await register_org(newService, 0x68);
    await approve(orgName, true, newService, 0x68);
    await update_supported_tags(orgName, newService, 0x68);
    await change_service_admin(newAccount.address, newService, 0x68);
    // change_service_admin
    await change_service_admin(newAccount.address);
    // recheck
    await register_org(newService);
    await approve(orgName, true, newService);
    await update_supported_tags(orgName, newService);
    await change_service_admin(account.address, newService);
  });

  test('test change_org_admin', async () => {
    // register org and approve
    let res = await register_org();
    const orgName = res['org_name'];
    const tags = res['tags'];
    await approve(orgName);
    // create new account and transfer coins
    const newAccount = genRandomAccount();
    const assetService = new AssetService(client, account);
    const res2 = await assetService.write.transfer({
      asset_id: '0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c',
      to: newAccount.address,
      value: 99999999,
      memo: 'test',
    });
    expect(Number(res2.response.response.code)).toBe(0);
    // before update check update_user_tags, change_org_admin
    const newService = new KycService(client, newAccount);
    await change_org_admin(orgName, newAccount.address, newService, 0x68);
    await update_user_tags(orgName, tags, newService, 0x68);
    // update org admin
    await change_org_admin(orgName, newAccount.address);
    // recheck update_user_tags, change_org_admin
    await update_user_tags(orgName, tags, newService);
    await change_org_admin(orgName, account.address, newService);
  });

  // test eval_user_tag_expression
  test('test eval_user_tag_expression', async () => {
    // register org and approve
    let res = await register_org();
    const orgName = res['org_name'];
    const supportedTags = res['tags'];
    await approve(orgName);
    // update user tags after approved
    const res1 = await update_user_tags(orgName, supportedTags);
    const user = res1['user'];
    const values = res1['values'];
    // test basic expression
    const expression_0 = orgName + '.' + supportedTags[0] + '@`' + values[supportedTags[0]][0] +'`';
    eval_user_tag_expression(user, expression_0, 0, true);
    const randomAddress = genRandomAccount().address;
    eval_user_tag_expression(randomAddress, expression_0, 0, false);
    const expression_1 = orgName + '.' + supportedTags[0] + '@`' + values[supportedTags[1]][0] +'`';
    eval_user_tag_expression(user, expression_1, 0, false);
    // test complex expression
    const expression_2 = '(' + orgName + '.' + supportedTags[0] + '@`' + values[supportedTags[0]][0]
      + '` || ' + orgName + '.' + supportedTags[1] + '@`' + values[supportedTags[0]][0] + '`) && '
      + orgName + '.' + supportedTags[2] + '@`' + values[supportedTags[2]][2] + '`';
    eval_user_tag_expression(user, expression_2, 0, true);
  });
});
