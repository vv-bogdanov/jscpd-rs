#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));

const cliSpec = process.env.SOCKET_CLI_SPEC || 'socket@1.1.112';
const requirePublished = process.env.SOCKET_REQUIRE_PUBLISHED === '1';
const allowUnavailable = process.env.SOCKET_ALLOW_UNAVAILABLE === '1';
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
    license: numberEnv('SOCKET_MIN_MAIN_LICENSE', 100),
    maintenance: numberEnv('SOCKET_MIN_MAIN_MAINTENANCE', 90),
    quality: numberEnv('SOCKET_MIN_MAIN_QUALITY', 95),
    supplyChain: numberEnv('SOCKET_MIN_MAIN_SUPPLY_CHAIN', 70),
    vulnerability: numberEnv('SOCKET_MIN_MAIN_VULNERABILITY', 100),
  },
  platform: {
    license: numberEnv('SOCKET_MIN_PLATFORM_LICENSE', 100),
    maintenance: numberEnv('SOCKET_MIN_PLATFORM_MAINTENANCE', 88),
    quality: numberEnv('SOCKET_MIN_PLATFORM_QUALITY', 50),
    supplyChain: numberEnv('SOCKET_MIN_PLATFORM_SUPPLY_CHAIN', 50),
    vulnerability: numberEnv('SOCKET_MIN_PLATFORM_VULNERABILITY', 100),
  },
};
const scoreKeys = ['supplyChain', 'quality', 'maintenance', 'vulnerability', 'license'];

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
let unavailableIssues = [];

for (let attempt = 1; attempt <= attempts; attempt += 1) {
  const result = runSocketCheck(process.env);
  lastStatus = result.status ?? 1;
  lastOutput = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  response = parseSocketJson(lastOutput);

  if (response?.ok === true) {
    const unavailable = unavailablePackageResults(response);
    unavailableIssues = unavailable;
    if (unavailable.length === 0) {
      break;
    }
    if (unavailable.length > 0) {
      lastOutput = unavailable.join('\n');
    }
    if (unavailable.length > 0 && attempt < attempts) {
      console.error(
        `Socket package score unavailable, retrying in ${retryDelayMs}ms (${attempt}/${attempts})`,
      );
      for (const issue of unavailable) {
        console.error(issue);
      }
      await sleep(retryDelayMs);
      continue;
    }
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

if (response?.ok !== true) {
  if ((!requirePublished || allowUnavailable) && isSkippableUnavailable(response, lastOutput)) {
    console.log('Socket package score check skipped: package score is not available yet.');
    process.exit(0);
  }
  console.error(lastOutput.trim());
  process.exit(1);
}

if (unavailableIssues.length > 0) {
  if (!requirePublished || allowUnavailable) {
    console.log('Socket package score check skipped: package score is not available yet.');
    process.exit(0);
  }
  for (const issue of unavailableIssues) {
    console.error(issue);
  }
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
  const scores = Object.fromEntries(
    scoreKeys.map((key) => [key, scorePercent(score[key])]),
  );
  const min = thresholds[pkg.kind];

  console.log(
    `${pkg.name}@${pkg.version}: ` +
      scoreKeys.map((key) => `${key}=${scores[key].toFixed(0)}`).join(' ') +
      ` kind=${pkg.kind}`,
  );

  for (const key of scoreKeys) {
    if (scores[key] < min[key]) {
      console.error(
        `${pkg.name}@${pkg.version} Socket ${key} ${scores[key].toFixed(0)} ` +
          `is below required ${min[key]}`,
      );
      failures += 1;
    }
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
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = start; index < output.length; index += 1) {
    const char = output[index];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === '"') {
        inString = false;
      }
      continue;
    }
    if (char === '"') {
      inString = true;
    } else if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
      if (depth === 0) {
        try {
          return JSON.parse(output.slice(start, index + 1));
        } catch {
          return undefined;
        }
      }
    }
  }
  return undefined;
}

function runSocketCheck(env) {
  return spawnSync(
    'npx',
    ['--yes', cliSpec, 'package', 'shallow', ...purls, '--json'],
    {
      encoding: 'utf8',
      env,
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
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
    /not found|not published|pending scan|score unavailable|too many requests|unauthorized/i.test(output)
  );
}

function shouldRetry(payload, output) {
  const code = payload?.data?.code;
  return code === 404 || code === 429 || /not found|not published|pending scan|score unavailable|too many requests/i.test(output);
}

function unavailablePackageResults(payload) {
  const results = new Map((payload.data ?? []).map((item) => [item.inputPurl, item]));
  const issues = [];

  for (const pkg of packages) {
    const purl = `pkg:npm/${pkg.name}@${pkg.version}`;
    const item = results.get(purl);
    if (!item) {
      issues.push(`${pkg.name}@${pkg.version} is missing from Socket package results`);
      continue;
    }

    const retryableAlert = (item.alerts ?? []).find((alert) => {
      const text = [
        alert.type,
        alert.key,
        alert.name,
        alert.title,
        alert.message,
        alert.description,
      ]
        .filter(Boolean)
        .join(' ');
      return /not.?found|not.?published|pending.?scan|score.?unavailable|package.?unavailable/i.test(text);
    });
    if (retryableAlert) {
      issues.push(
        `${pkg.name}@${pkg.version} Socket score is not indexed yet: ` +
          `${retryableAlert.type ?? retryableAlert.key ?? retryableAlert.message ?? 'unavailable'}`,
      );
      continue;
    }

    const score = item.score ?? {};
    const hasScores = scoreKeys.every((key) => Number.isFinite(Number(score[key])));
    if (!hasScores) {
      issues.push(`${pkg.name}@${pkg.version} Socket score is not available yet`);
    }
  }

  return issues;
}

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
