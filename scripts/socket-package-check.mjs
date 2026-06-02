#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));

const cliSpec = process.env.SOCKET_CLI_SPEC || 'socket@1.1.112';
const requirePublished = process.env.SOCKET_REQUIRE_PUBLISHED === '1';
const attempts = numberEnv('SOCKET_RETRIES', requirePublished ? 3 : 1);
const retryDelayMs = numberEnv('SOCKET_RETRY_DELAY_MS', 15000);
const failOnAlertSeverities = new Set(
  (process.env.SOCKET_FAIL_ON_ALERT_SEVERITIES || 'critical,high')
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean),
);

const thresholds = {
  main: {
    maintenance: numberEnv('SOCKET_MIN_MAIN_MAINTENANCE', 90),
    supplyChain: numberEnv('SOCKET_MIN_MAIN_SUPPLY_CHAIN', 70),
  },
  platform: {
    maintenance: numberEnv('SOCKET_MIN_PLATFORM_MAINTENANCE', 88),
    supplyChain: numberEnv('SOCKET_MIN_PLATFORM_SUPPLY_CHAIN', 50),
  },
};

const packages = [
  {
    kind: 'main',
    name: packageJson.name,
    version: packageJson.version,
  },
  ...Object.entries(packageJson.optionalDependencies ?? {})
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, version]) => ({ kind: 'platform', name, version })),
];

const packageByPurl = new Map(
  packages.map((pkg) => [`pkg:npm/${pkg.name}@${pkg.version}`, pkg]),
);
const purls = packages.map((pkg) => `pkg:npm/${pkg.name}@${pkg.version}`);

let response;
let lastOutput = '';
let lastStatus = 1;

for (let attempt = 1; attempt <= attempts; attempt += 1) {
  const result = spawnSync(
    'npx',
    ['--yes', cliSpec, 'package', 'shallow', ...purls, '--json'],
    {
      encoding: 'utf8',
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
  lastStatus = result.status ?? 1;
  lastOutput = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  response = parseSocketJson(lastOutput);

  if (lastStatus === 0 && response?.ok === true) {
    break;
  }

  if (attempt < attempts && shouldRetry(response, lastOutput)) {
    console.error(
      `Socket package score unavailable, retrying in ${retryDelayMs}ms (${attempt}/${attempts})`,
    );
    await sleep(retryDelayMs);
  } else {
    break;
  }
}

if (lastStatus !== 0 || response?.ok !== true) {
  if (!requirePublished && isSkippableUnavailable(response, lastOutput)) {
    console.log('Socket package score check skipped: package score is not available yet.');
    process.exit(0);
  }
  console.error(lastOutput.trim());
  process.exit(1);
}

let failures = 0;
for (const item of response.data ?? []) {
  const purl = item.inputPurl;
  const pkg = packageByPurl.get(purl);
  if (!pkg) {
    console.error(`unexpected Socket package result: ${purl}`);
    failures += 1;
    continue;
  }

  const score = item.score ?? {};
  const supplyChain = scorePercent(score.supplyChain);
  const maintenance = scorePercent(score.maintenance);
  const min = thresholds[pkg.kind];

  console.log(
    `${pkg.name}@${pkg.version}: supplyChain=${supplyChain.toFixed(0)} ` +
      `maintenance=${maintenance.toFixed(0)} kind=${pkg.kind}`,
  );

  if (supplyChain < min.supplyChain) {
    console.error(
      `${pkg.name}@${pkg.version} Socket supplyChain ${supplyChain.toFixed(0)} ` +
        `is below required ${min.supplyChain}`,
    );
    failures += 1;
  }
  if (maintenance < min.maintenance) {
    console.error(
      `${pkg.name}@${pkg.version} Socket maintenance ${maintenance.toFixed(0)} ` +
        `is below required ${min.maintenance}`,
    );
    failures += 1;
  }

  for (const alert of item.alerts ?? []) {
    const severity = String(alert.severity ?? '').toLowerCase();
    if (failOnAlertSeverities.has(severity) && alert.action !== 'ignore') {
      console.error(
        `${pkg.name}@${pkg.version} has Socket ${severity} alert: ` +
          `${alert.type ?? alert.key ?? 'unknown'}`,
      );
      failures += 1;
    }
  }
}

const missing = packages.filter(
  (pkg) => !response.data?.some((item) => item.inputPurl === `pkg:npm/${pkg.name}@${pkg.version}`),
);
for (const pkg of missing) {
  console.error(`Socket package result is missing for ${pkg.name}@${pkg.version}`);
  failures += 1;
}

if (failures > 0) {
  process.exit(1);
}

function numberEnv(name, fallback) {
  const raw = process.env[name];
  if (!raw) {
    return fallback;
  }
  const parsed = Number(raw);
  if (!Number.isFinite(parsed)) {
    console.error(`${name} must be a number, got ${raw}`);
    process.exit(2);
  }
  return parsed;
}

function parseSocketJson(output) {
  const start = output.indexOf('{');
  if (start < 0) {
    return undefined;
  }
  try {
    return JSON.parse(output.slice(start));
  } catch {
    return undefined;
  }
}

function scorePercent(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return 0;
  }
  return numeric <= 1 ? numeric * 100 : numeric;
}

function isSkippableUnavailable(payload, output) {
  const code = payload?.data?.code;
  return (
    code === 401 ||
    code === 404 ||
    code === 429 ||
    /not found|not published|too many requests|unauthorized/i.test(output)
  );
}

function shouldRetry(payload, output) {
  const code = payload?.data?.code;
  return code === 404 || code === 429 || /not found|not published|too many requests/i.test(output);
}

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
