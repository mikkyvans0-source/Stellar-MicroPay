/**
 * src/config/validateEnv.js
 * Fail-fast validation for required backend environment variables.
 */

"use strict";

const VALID_NETWORKS = ["testnet", "mainnet"];

function collectErrors(env) {
  const errors = [];

  const stellarNetwork = env.STELLAR_NETWORK?.trim();
  if (!stellarNetwork) {
    errors.push('STELLAR_NETWORK is required (e.g. "testnet" or "mainnet")');
  } else if (!VALID_NETWORKS.includes(stellarNetwork)) {
    errors.push(
      `STELLAR_NETWORK must be "testnet" or "mainnet", got "${stellarNetwork}"`
    );
  }

  const horizonUrl = env.HORIZON_URL?.trim();
  if (!horizonUrl) {
    errors.push(
      'HORIZON_URL is required (e.g. "https://horizon-testnet.stellar.org")'
    );
  } else {
    try {
      new URL(horizonUrl);
    } catch {
      errors.push(`HORIZON_URL must be a valid URL, got "${horizonUrl}"`);
    }
  }

  return errors;
}

/**
 * Validate required environment variables.
 * Logs actionable errors and exits the process when validation fails.
 *
 * @param {NodeJS.ProcessEnv} [env=process.env]
 */
function validateEnv(env = process.env) {
  const errors = collectErrors(env);

  if (errors.length === 0) {
    return;
  }

  console.error("\nEnvironment validation failed:\n");
  for (const message of errors) {
    console.error(`  - ${message}`);
  }
  console.error("\nCopy backend/.env.example to backend/.env and set the required values.\n");
  process.exit(1);
}

module.exports = { validateEnv, collectErrors };
