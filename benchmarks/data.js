window.BENCHMARK_DATA = {
  "lastUpdate": 1753033099579,
  "repoUrl": "https://github.com/julienbrs/ethrex",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "72628438+avilagaston9@users.noreply.github.com",
            "name": "Avila Gastón",
            "username": "avilagaston9"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ce5c47df70fa92c91814f36df65c01a090b19de1",
          "message": "fix(l2): estimate gas in call_to_contract_with_deposit (#3734)\n\n**Motivation**\n\nThe CI is failing on\n[main](https://github.com/lambdaclass/ethrex/actions/runs/16376083320/job/46276248732)\nwith the following error:\n\n```\nthread 'l2_integration_test' panicked at crates/l2/tests/tests.rs:1604:65:\ncalled `Option::unwrap()` on a `None` value\n```\n\nThis is because we were using a hardcoded `gas_limit` for the\n`l1_to_l2_tx` in the `call_to_contract_with_deposit` test, and sometimes\nthe tx fails due to the gas limit being exceeded. Then, the expected\nlogs of are never created.\n\n**Description**\n\n- Replaces the hardcoded `gas limit` with `None` to allow the SDK to\nestimate the value.\n\nCloses None",
          "timestamp": "2025-07-18T22:01:22Z",
          "tree_id": "77ef3295f487c841398d311894bdbbe16ec60cc8",
          "url": "https://github.com/julienbrs/ethrex/commit/ce5c47df70fa92c91814f36df65c01a090b19de1"
        },
        "date": 1753033097899,
        "tool": "cargo",
        "benches": [
          {
            "name": "Block import/Block import ERC20 transfers",
            "value": 209986143901,
            "range": "± 397557034",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}