import { test, describe } from "node:test";
import assert from "node:assert/strict";

// Import the compiled or direct implementations
import {
  parsePkrInput,
  formatPkr,
  formatPkrWhole,
  formatDate,
  advanceSubscriptionDate,
  calcSubscriptionMonthlyCost,
  totalMonthlySubscriptionCost,
  calculateMonthsRemaining,
  evaluatePlanMetrics,
  computePlanProgress,
} from "../src/lib/utils.ts";

describe("Web PWA Unit Tests - Currency and Math Parity", () => {
  test("parsePkrInput strips commas and parses correctly", () => {
    assert.equal(parsePkrInput("1000"), 1000);
    assert.equal(parsePkrInput("1,000"), 1000);
    assert.equal(parsePkrInput("1,234,567.89"), 1234567.89);
    assert.equal(parsePkrInput("  50,000.50  "), 50000.5);
    assert.equal(parsePkrInput("invalid"), null);
    assert.equal(parsePkrInput(""), null);
  });

  test("formatPkr formats correctly with commas and 2 decimals", () => {
    assert.equal(formatPkr(1234567.89), "Rs 1,234,567.89");
    assert.equal(formatPkr(-500.5), "-Rs 500.50");
    assert.equal(formatPkr(0), "Rs 0.00");
  });

  test("formatPkrWhole formats whole rupees", () => {
    assert.equal(formatPkrWhole(106736.2), "Rs 106,736");
    assert.equal(formatPkrWhole(-2500), "-Rs 2,500");
  });

  test("formatDate formats YYYY-MM-DD to DD Mon YYYY", () => {
    assert.equal(formatDate("2026-09-29"), "29 Sep 2026");
    assert.equal(formatDate("2027-01-05"), "5 Jan 2027");
  });

  test("advanceSubscriptionDate advances by monthly, yearly, and custom cycles", () => {
    assert.equal(advanceSubscriptionDate("2026-09-01", "monthly"), "2026-10-01");
    assert.equal(advanceSubscriptionDate("2026-09-01", "yearly"), "2027-09-01");
    assert.equal(advanceSubscriptionDate("2026-09-01", "custom:14"), "2026-09-15");
  });

  test("calcSubscriptionMonthlyCost prorates cycles and ignores paused", () => {
    const monthlySub = {
      id: "1",
      card_id: "c1",
      name: "Spotify",
      amount: 400,
      billing_cycle: "monthly",
      next_due_date: "2026-10-01",
      is_paused: false,
      updated_at: new Date().toISOString(),
    };
    assert.equal(calcSubscriptionMonthlyCost(monthlySub), 400);

    const yearlySub = {
      ...monthlySub,
      name: "HBO MAX",
      amount: 8200,
      billing_cycle: "yearly",
    };
    assert.ok(Math.abs(calcSubscriptionMonthlyCost(yearlySub) - (8200 / 12)) < 0.001);

    const pausedSub = {
      ...monthlySub,
      is_paused: true,
    };
    assert.equal(calcSubscriptionMonthlyCost(pausedSub), 0);
  });

  test("computePlanProgress calculates surplus, progress %, and feasibility", () => {
    const plan = {
      id: "p1",
      name: "MacBook",
      target_amount: 300000,
      deadline: "2026-12-31",
      monthly_income: 200000,
      spending_limit: 100000,
      is_active: true,
      deduct_overspending: true,
      linked_card_ids: [],
      created_at: "2026-09-01",
      updated_at: new Date().toISOString(),
    };

    const txs = [
      {
        id: "t1",
        card_id: "c1",
        date: "2026-09-10",
        description: "Rent",
        category: "Other",
        amount: 60000,
        is_income: false,
        auto_generated: false,
        updated_at: new Date().toISOString(),
      },
    ];

    const calc = computePlanProgress(plan, txs, 0, "2026-09-29", true);
    assert.ok(calc.is_feasible);
    // Limit is 100,000; Spent is 60,000 -> Surplus is 40,000
    assert.equal(calc.total_saved, 40000);
    assert.ok(calc.percent_reached > 0);
  });
});
