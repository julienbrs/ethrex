window.BENCHMARK_DATA = {
  "lastUpdate": 1753947274940,
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
      },
      {
        "commit": {
          "author": {
            "email": "estefano.bargas@fing.edu.uy",
            "name": "Estéfano Bargas",
            "username": "xqft"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0d5a49278a85ec3e7d663b403e53c23d6f1b5cd0",
          "message": "fix(l2): integration tests hotfix (#3892)\n\n**Motivation**\n\nafter merging #3776 a test was added that spams the bridge with big gas\nlimit transactions. In #3851 we upped the gas limit of all transactions\nby x2 because the estimation was too low. These two contributed to\nexceed the block gas limit.\n\nThis PR lowers the amount of spam txs\n\nthanks @iovoid !",
          "timestamp": "2025-07-30T21:01:48Z",
          "tree_id": "26a5877ab68e3eac5b1d233323d327de12e0e04e",
          "url": "https://github.com/julienbrs/ethrex/commit/0d5a49278a85ec3e7d663b403e53c23d6f1b5cd0"
        },
        "date": 1753947273061,
        "tool": "cargo",
        "benches": [
          {
            "name": "Block import/Block import ERC20 transfers",
            "value": 160481725035,
            "range": "± 182344157",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}