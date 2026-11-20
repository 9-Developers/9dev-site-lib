import { execFileSync } from 'node:child_process';

const mode = process.argv[2];
if (mode !== 'check' && mode !== 'publish') throw new Error('Expected check or publish');
const tag = process.env.FORGEJO_REF_NAME;
const { packages } = JSON.parse(execFileSync('cargo', [
  'metadata', '--locked', '--no-deps', '--format-version', '1',
], { encoding: 'utf8' }));

const target = packages.find(({ name, version, publish }) =>
  tag === `${name}-v${version}` && /^\d+\.\d+\.\d+$/.test(version) &&
  (!publish || publish.includes('crates-io'))
);
if (!target) throw new Error(`No publishable workspace crate matches release tag: ${tag}`);
const { name, version } = target;
console.log(`Release target: ${name}@${version}`);
if (mode === 'check') process.exit(0);

const response = await fetch(`https://crates.io/api/v1/crates/${name}/${version}`, {
  headers: { 'User-Agent': '9dev-site-lib release workflow (https://github.com/9-Developers/9dev-site-lib)' },
});
if (response.status === 404) {
  execFileSync('cargo', ['publish', '--registry', 'crates-io', '--locked', '-p', name], { stdio: 'inherit' });
} else {
  if (!response.ok) throw new Error(`Could not check ${name}@${version}: HTTP ${response.status}`);
  const { version: published } = await response.json();
  if (published?.crate !== name || published.num !== version) {
    throw new Error(`Unexpected registry response for ${name}@${version}`);
  }
  console.log(`Skipping ${name}@${version}: already published`);
}
