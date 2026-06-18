#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const out = process.argv[2] || path.join(process.cwd(), ".nanda", "finance-16k-risk-cluster.json");

const tickers = [
  {
    t: "NVDA",
    kind: "equity",
    price: "210.69 USD",
    change: "+2.849%",
    route: "ai-compute-leader",
  },
  {
    t: "AMD",
    kind: "equity",
    price: "537.37 USD",
    change: "+4.718%",
    route: "ai-compute-challenger",
  },
  {
    t: "TSM",
    kind: "equity",
    price: "462.12 USD",
    change: "+6.970%",
    route: "foundry-supply-chain",
  },
  {
    t: "QQQ",
    kind: "fund",
    price: "740.62 USD",
    change: "+2.375%",
    route: "mega-cap-tech-index",
  },
  {
    t: "SMH",
    kind: "fund",
    price: "659.88 USD",
    change: "+5.614%",
    route: "semiconductor-basket",
  },
];

const routes = [
  "ai-demand",
  "valuation-risk",
  "foundry-capacity",
  "index-concentration",
  "macro-rates",
  "earnings-quality",
  "geopolitical-risk",
  "options-momentum",
  "liquidity-flow",
  "semiconductor-cycle",
  "false-diversification",
  "hedge-coverage",
  "news-vs-price",
  "multiple-expansion",
  "supply-bottleneck",
  "customer-concentration",
];

const relations = [
  "loads_on",
  "depends_on",
  "amplifies",
  "is_sensitive_to",
  "correlates_with",
  "is_exposed_to",
  "is_not_same_as",
  "requires_confirmation_from",
];

const triads = [];
let nextId = 1;

function add(subject, relation, object, route, group, confidence, evidence) {
  triads.push({
    id: `m${nextId++}`,
    subject,
    relation,
    object,
    evidence,
    confidence,
    subject_role: subject.includes("risk")
      ? "risk"
      : subject.includes("thesis")
        ? "thesis"
        : "instrument",
    object_role: object.includes("risk")
      ? "risk"
      : object.includes("route")
        ? "route"
        : "factor",
    route,
    group,
  });
}

for (const ticker of tickers) {
  add(
    ticker.t,
    "has_live_price_snapshot",
    ticker.price,
    "price-snapshot",
    "live-market-snapshot",
    1.0,
    "captured during 2026-06-18 NANDA finance load-test",
  );
  add(
    ticker.t,
    "has_intraday_change",
    ticker.change,
    "price-snapshot",
    "live-market-snapshot",
    1.0,
    "captured during 2026-06-18 NANDA finance load-test",
  );
  add(
    ticker.t,
    "has_market_identity",
    ticker.kind,
    "instrument-identity",
    "instrument-identity",
    1.0,
    "finance fixture metadata",
  );
  add(
    ticker.t,
    "belongs_to_route",
    ticker.route,
    ticker.route,
    ticker.route,
    0.95,
    "finance fixture taxonomy",
  );
}

for (let i = 0; triads.length < 16384; i++) {
  const ticker = tickers[i % tickers.length].t;
  const route = routes[i % routes.length];
  const relation = relations[(i * 7 + Math.floor(i / 13)) % relations.length];
  const factorBucket = Math.floor(i / routes.length) % 128;
  const scenario = Math.floor(i / 128) % 128;
  const subject = `${ticker} ${route} exposure ${factorBucket}`;
  const object = `${route} factor scenario ${scenario}`;
  const group =
    route.includes("false") || route.includes("concentration") || route.includes("cycle")
      ? "portfolio-risk-cluster"
      : `${route}-group`;
  const confidence = Number((0.55 + (i % 45) / 100).toFixed(2));
  add(subject, relation, object, route, group, confidence, `generated-load-test:${i}`);
}

const packet = {
  task_id: "finance-16k-risk-cluster-load-test",
  domain: "finance-risk-structure",
  query:
    "Does a NVDA AMD TSM SMH QQQ portfolio have hidden AI semiconductor concentration despite different tickers?",
  triads,
  candidate_triads: [
    {
      id: "q1",
      subject: "NVDA AMD TSM SMH QQQ portfolio",
      relation: "is_diversified_because",
      object: "tickers are different",
      evidence: "user trade thesis",
      confidence: 1.0,
      subject_role: "portfolio",
      object_role: "claim",
      route: "candidate-diversification-thesis",
      group: "query",
    },
    {
      id: "q2",
      subject: "NVDA AMD TSM SMH QQQ portfolio",
      relation: "has_hidden_common_exposure_to",
      object: "AI semiconductor risk cluster",
      evidence: "risk review query",
      confidence: 1.0,
      subject_role: "portfolio",
      object_role: "risk_cluster",
      route: "candidate-risk-check",
      group: "query",
    },
    {
      id: "q3",
      subject: "different ticker count",
      relation: "is_not_same_as",
      object: "independent risk routes",
      evidence: "risk review query",
      confidence: 1.0,
      subject_role: "portfolio_metric",
      object_role: "risk_property",
      route: "candidate-risk-check",
      group: "query",
    },
  ],
  candidate_answer: "The portfolio is diversified because it has several different tickers.",
};

fs.mkdirSync(path.dirname(out), { recursive: true });
fs.writeFileSync(out, `${JSON.stringify(packet, null, 2)}\n`);
console.log(
  JSON.stringify(
    {
      out,
      triads: packet.triads.length,
      candidate_triads: packet.candidate_triads.length,
      routes: routes.length,
    },
    null,
    2,
  ),
);
