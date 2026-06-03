#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const args = parseArgs(process.argv.slice(2));

if (!args.dir) {
  fail('--dir is required');
}
if (!args.mainTarball && !args.registry) {
  fail('one of --main-tarball or --registry is required');
}
if (args.mainTarball && args.registry) {
  fail('--main-tarball and --registry are mutually exclusive');
}

const root = process.cwd();
const installDir = path.resolve(args.dir);
const targets = readJson(path.join(root, 'npm/prebuilt-targets.json'));
const targetByPackage = new Map(
  Object.values(targets).map((target) => [target.packageName, target]),
);

fs.mkdirSync(installDir, { recursive: true });

const main = args.mainTarball
  ? localPackage(args.mainTarball)
  : registryPackage(args.registry);
const prebuilt = args.prebuiltTarball ? localPackage(args.prebuiltTarball) : undefined;

const rootDependencies = {
  [main.packageJson.name]: main.spec,
};
if (prebuilt) {
  rootDependencies[prebuilt.packageJson.name] = prebuilt.spec;
}

const packages = {
  '': {
    name: 'jscpd-rs-smoke',
    version: '1.0.0',
    private: true,
    dependencies: rootDependencies,
  },
  [`node_modules/${main.packageJson.name}`]: lockEntry(main, { optional: false }),
};

for (const [name, version] of Object.entries(main.packageJson.optionalDependencies ?? {}).sort()) {
  if (prebuilt?.packageJson.name === name) {
    packages[`node_modules/${name}`] = lockEntry(prebuilt, { optional: true });
  } else if (args.registry) {
    packages[`node_modules/${name}`] = lockEntry(registryPackage(`${name}@${version}`), {
      optional: true,
    });
  } else {
    packages[`node_modules/${name}`] = placeholderOptionalEntry(name, version);
  }
}

writeJson(path.join(installDir, 'package.json'), {
  name: 'jscpd-rs-smoke',
  version: '1.0.0',
  private: true,
  dependencies: rootDependencies,
});
writeJson(path.join(installDir, 'package-lock.json'), {
  name: 'jscpd-rs-smoke',
  version: '1.0.0',
  lockfileVersion: 3,
  requires: true,
  packages,
});

const ciArgs = ['ci', '--ignore-scripts', '--no-audit', '--no-fund'];
if (args.omitOptional) {
  ciArgs.push('--omit=optional');
}
const ci = spawnSync('npm', ciArgs, {
  cwd: installDir,
  encoding: 'utf8',
  stdio: 'inherit',
});
process.exit(ci.status ?? 1);

function parseArgs(values) {
  const parsed = {
    dir: undefined,
    mainTarball: undefined,
    omitOptional: false,
    prebuiltTarball: undefined,
    registry: undefined,
  };
  for (let index = 0; index < values.length; index += 1) {
    const arg = values[index];
    if (arg === '--dir') {
      parsed.dir = requiredValue(values, ++index, arg);
    } else if (arg === '--main-tarball') {
      parsed.mainTarball = requiredValue(values, ++index, arg);
    } else if (arg === '--omit-optional') {
      parsed.omitOptional = true;
    } else if (arg === '--prebuilt-tarball') {
      parsed.prebuiltTarball = requiredValue(values, ++index, arg);
    } else if (arg === '--registry') {
      parsed.registry = requiredValue(values, ++index, arg);
    } else {
      fail(`unknown argument: ${arg}`);
    }
  }
  return parsed;
}

function requiredValue(values, index, flag) {
  const value = values[index];
  if (!value || value.startsWith('--')) {
    fail(`${flag} requires a value`);
  }
  return value;
}

function localPackage(tarball) {
  const resolved = path.resolve(tarball);
  const packageJson = packageJsonFromTarball(resolved);
  return {
    integrity: fileIntegrity(resolved),
    packageJson,
    resolved: `file:${resolved}`,
    spec: `file:${resolved}`,
  };
}

function registryPackage(spec) {
  const result = spawnSync('npm', ['view', spec, '--json'], {
    encoding: 'utf8',
    maxBuffer: 10 * 1024 * 1024,
  });
  if (result.status !== 0) {
    process.stderr.write(result.stderr);
    fail(`npm view failed for ${spec}`);
  }
  const packageJson = JSON.parse(result.stdout);
  return {
    integrity: packageJson.dist?.integrity,
    packageJson,
    resolved: packageJson.dist?.tarball,
    spec: packageJson.version,
  };
}

function lockEntry(pkg, { optional }) {
  const entry = {
    version: pkg.packageJson.version,
    resolved: pkg.resolved,
    integrity: pkg.integrity,
  };
  copyIfPresent(entry, pkg.packageJson, 'bin');
  copyIfPresent(entry, pkg.packageJson, 'cpu');
  copyIfPresent(entry, pkg.packageJson, 'engines');
  copyIfPresent(entry, pkg.packageJson, 'libc');
  copyIfPresent(entry, pkg.packageJson, 'license');
  copyIfPresent(entry, pkg.packageJson, 'optionalDependencies');
  copyIfPresent(entry, pkg.packageJson, 'os');
  if (optional) {
    entry.optional = true;
  }
  return entry;
}

function placeholderOptionalEntry(name, version) {
  const target = targetByPackage.get(name);
  const entry = {
    version,
    resolved: `https://registry.npmjs.org/${name}/-/${name}-${version}.tgz`,
    integrity: `sha512-${Buffer.alloc(64).toString('base64')}`,
    license: 'MIT',
    optional: true,
    engines: { node: '>=18' },
  };
  if (target?.cpu) {
    entry.cpu = [target.cpu];
  }
  if (target?.libc) {
    entry.libc = [target.libc];
  }
  if (target?.os) {
    entry.os = [target.os];
  }
  return entry;
}

function packageJsonFromTarball(tarball) {
  const result = spawnSync('tar', ['-xOf', tarball, 'package/package.json'], {
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  if (result.status !== 0) {
    process.stderr.write(result.stderr);
    fail(`failed to read package/package.json from ${tarball}`);
  }
  return JSON.parse(result.stdout);
}

function fileIntegrity(file) {
  const digest = crypto.createHash('sha512').update(fs.readFileSync(file)).digest('base64');
  return `sha512-${digest}`;
}

function copyIfPresent(target, source, key) {
  if (source[key] !== undefined) {
    target[key] = source[key];
  }
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function writeJson(file, data) {
  fs.writeFileSync(file, `${JSON.stringify(data, null, 2)}\n`);
}

function fail(message) {
  console.error(`npm ci install failed: ${message}`);
  process.exit(1);
}
