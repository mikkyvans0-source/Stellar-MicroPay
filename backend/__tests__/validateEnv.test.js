/**
 * __tests__/validateEnv.test.js
 * Unit tests for startup environment validation.
 */

"use strict";

const { collectErrors } = require("../src/config/validateEnv");

describe("validateEnv.collectErrors", () => {
  it("returns no errors when required vars are valid", () => {
    expect(
      collectErrors({
        STELLAR_NETWORK: "testnet",
        HORIZON_URL: "https://horizon-testnet.stellar.org",
      })
    ).toEqual([]);
  });

  it("flags missing STELLAR_NETWORK and HORIZON_URL", () => {
    const errors = collectErrors({});
    expect(errors).toEqual(
      expect.arrayContaining([
        expect.stringContaining("STELLAR_NETWORK is required"),
        expect.stringContaining("HORIZON_URL is required"),
      ])
    );
  });

  it("rejects invalid STELLAR_NETWORK values", () => {
    const errors = collectErrors({
      STELLAR_NETWORK: "devnet",
      HORIZON_URL: "https://horizon-testnet.stellar.org",
    });
    expect(errors).toEqual(
      expect.arrayContaining([
        expect.stringContaining('STELLAR_NETWORK must be "testnet" or "mainnet"'),
      ])
    );
  });

  it("rejects malformed HORIZON_URL", () => {
    const errors = collectErrors({
      STELLAR_NETWORK: "testnet",
      HORIZON_URL: "not-a-url",
    });
    expect(errors).toEqual(
      expect.arrayContaining([
        expect.stringContaining("HORIZON_URL must be a valid URL"),
      ])
    );
  });
});
