import { execFileSync } from 'node:child_process';

const { packages } = JSON.parse(execFileSync('cargo', [
  'metadata', '--locked', '--no-deps', '--format-version', '1',
], { encoding: 'utf8' }));

const missing = [];
for (const { name, version, publish } of packages) {
  if (publish && !publish.includes('crates-io')) continue;
  if (`v${version}` !== process.env.FORGEJO_REF_NAME) {
    throw new Error(`${name} version ${version} does not match ${process.env.FORGEJO_REF_NAME}`);
  }

  const response = await fetch(`https://crates.io/api/v1/crates/${name}/${version}`, {
    headers: { 'User-Agent': '9dev-site-lib release workflow (https://github.com/9-Developers/9dev-site-lib)' },
  });
  if (response.status === 404) {
    missing.push('-p', name);
    continue;
  }
  if (!response.ok) throw new Error(`Could not check ${name}@${version}: HTTP ${response.status}`);
  const { version: published } = await response.json();
  if (published?.crate !== name || published.num !== version) {
    throw new Error(`Unexpected registry response for ${name}@${version}`);
  }
  console.log(`Skipping ${name}@${version}: already published`);
}

if (missing.length) {
  execFileSync('cargo', ['publish', '--registry', 'crates-io', '--locked', ...missing], { stdio: 'inherit' });
}
