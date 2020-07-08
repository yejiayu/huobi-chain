import { readFileSync, writeFileSync } from 'fs';
import * as path from 'path';
import * as _ from 'lodash';
// eslint-disable-next-line import/extensions
import { keypairs, init_node_num, init_bp_num } from './common';

const shell = require('shelljs');

const configPath = path.resolve(__dirname, 'configs');
const templatesPath = path.resolve(__dirname, 'templates');

shell.rm('-rf', path.resolve(configPath, '*'));

function render(input_str : string, output_name: string, obj) {
  const compiled = _.template(input_str);
  const output_str = compiled(obj);
  writeFileSync(path.resolve(configPath, output_name), output_str);
}

const configTemplate = readFileSync(
  path.resolve(templatesPath, 'chain.toml.template'),
);
const dockerComposeTemplate = readFileSync(
  path.resolve(templatesPath, 'docker-compose.yml.template'),
);
const genesisTemplate = readFileSync(
  path.resolve(templatesPath, 'genesis.toml.template'),
);

// console.log(keypairs.keypairs);
keypairs.keypairs.map((key) => {
  render(configTemplate, `chain-${key.index}.toml`, { keypairs, ...key });
});

render(dockerComposeTemplate, 'docker-compose.yml', keypairs);
render(genesisTemplate, 'genesis.toml', { init_bp_num, ...keypairs });
