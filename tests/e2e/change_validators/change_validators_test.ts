import { Muta } from 'muta-sdk';
import * as assert from 'assert';
import * as _ from 'lodash';
import { keypairs, init_node_num, init_bp_num } from './common';

export const CHAIN_ID = '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036';

const clients = keypairs.keypairs.map((keypair) => {
  const muta = new Muta({
    endpoint: `http://${keypair.ip}:8000/graphql`,
    chainId: CHAIN_ID,
  });
  const client = muta.client('0xffffffff', '0x1');
  return client;
});
const client = clients[0];

const admin = Muta.accountFromPrivateKey(
  '0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2',
);

// console.log(keypairs);

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function check_produces_block(heightNow?) {
  heightNow = heightNow || (await client.getBlock()).header.execHeight;
  const timeoutLoopTimes = 100;
  for (const i of _.range(timeoutLoopTimes)) {
    await sleep(200);
    const newHeight = (await client.getBlock()).header.execHeight;
    // console.log({ heightNow, newHeight, i });
    if (newHeight > heightNow) {
      return;
    }
  }
  throw new Error(`Not producing block for ${timeoutLoopTimes} loops`);
}

async function check_validators(validators_index) {
  const block = await client.getBlock();
  console.log(
    `check validators: ${validators_index}, block: ${JSON.stringify(
      block,
      null,
      2,
    )}`,
  );
  assert.deepEqual(
    block.header.validators.map((v) => v.address),
    validators_index.map((i) => keypairs.keypairs[i].address),
  );
  console.log(`check validators success: ${validators_index}`);
}

async function change_validators(validators_index) {
  const verifier_list = validators_index.map((i) => {
    const validator = {
      bls_pub_key: keypairs.keypairs[i].bls_public_key,
      address: keypairs.keypairs[i].address,
      propose_weight: 1,
      vote_weight: 1,
    };
    return validator;
  });
  const tx = await client.composeTransaction({
    method: 'update_validators',
    payload: {
      verifier_list,
    },
    serviceName: 'governance',
  });
  const signed_tx = admin.signTransaction(tx);
  const hash = await client.sendTransaction(signed_tx);
  const receipt = await client.getReceipt(hash);
  console.log(receipt);
  return receipt;
}

async function changeAndCheckValidators(validators_index) {
  console.log({ name: 'start_change_validator', validators_index });
  const receipt = await change_validators(validators_index);
  const heightFinishChange = receipt.height;
  await check_produces_block(heightFinishChange);
  await check_validators(validators_index);
  console.log({ name: 'finish_change_validator', validators_index });
}

async function main() {
  await check_produces_block();
  await check_validators(_.range(init_bp_num));
  console.log({ init_bp_num, init_node_num });
  for (const i of _.range(init_bp_num - 1, 0)) {
    await changeAndCheckValidators(_.range(i));
  }
  for (const i of _.range(1, init_node_num + 1, 1)) {
    await changeAndCheckValidators(_.range(i));
  }
  const nodes = _.range(init_node_num);
  for (const i of _.range(50)) {
    const randomNodes = _.sampleSize(nodes, _.random(1, init_node_num));
    // console.log({i, randomNodes});
    await changeAndCheckValidators(randomNodes);
  }
}

main();
