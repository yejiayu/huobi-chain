export const keypairs = require('./keypairs.json');

export const init_node_num = 5;
export const init_bp_num = 3;

keypairs.keypairs = keypairs.keypairs.slice(0, init_node_num);
keypairs.keypairs.map((keypair) => {
  const ip = `173.20.0.${keypair.index + 20}`;
  keypair.ip = ip;
});
