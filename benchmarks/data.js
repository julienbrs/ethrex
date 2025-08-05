window.BENCHMARK_DATA = {
  "lastUpdate": 1754409857917,
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
      },
      {
        "commit": {
          "author": {
            "email": "47506558+MegaRedHand@users.noreply.github.com",
            "name": "Tomás Grüner",
            "username": "MegaRedHand"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fb1fd37e64e6bc51ee7bdf0f5491fa401535604d",
          "message": "fix(l1): disable default request body limit on authenticated RPC (#3929)\n\n**Motivation**\n\nConsensus client calls to `engine_newPayload` sometimes failed with `413\npayload too large` HTTP errors (and sometimes `broken pipe`, which might\nbe related).\n\n**Description**\n\nThis PR solves this by disabling the request body limit, since it\ndoesn't make much sense to add a limit to authenticated requests.",
          "timestamp": "2025-08-01T23:30:19Z",
          "tree_id": "73cd781393cb7f53596e34b4ef6a27bc203b2e20",
          "url": "https://github.com/julienbrs/ethrex/commit/fb1fd37e64e6bc51ee7bdf0f5491fa401535604d"
        },
        "date": 1754167588247,
        "tool": "cargo",
        "benches": [
          {
            "name": "Block import/Block import ERC20 transfers",
            "value": 164365992478,
            "range": "± 377200508",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "manuel.bilbao@lambdaclass.com",
            "name": "Manuel Iñaki Bilbao",
            "username": "ManuelBilbao"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e610d4a9781d216c07289a650a7aa6e334d1fbae",
          "message": "refactor(l2): improve error handling in `EthClient`'s `send_request_to_all` (#3932)\n\n**Motivation**\n\n<!-- Why does this pull request exist? What are its goals? -->\nWhen sending a transaction, the `EthClient` returns the last `Ok()`\nresult, if there is one, but it doesn't take into account\n`Ok(RpcResponse::Error)` cases, resulting in confusing errors.\n\n**Description**\n\n<!-- A clear and concise general description of the changes this PR\nintroduces -->\nImprove the way responses are handled.\n\n<!-- Link to issues: Resolves #111, Resolves #222 -->",
          "timestamp": "2025-08-04T19:39:07Z",
          "tree_id": "ca9cd4c29fe79a5d44b398d1279bbf7faa7cf5fa",
          "url": "https://github.com/julienbrs/ethrex/commit/e610d4a9781d216c07289a650a7aa6e334d1fbae"
        },
        "date": 1754340231714,
        "tool": "cargo",
        "benches": [
          {
            "name": "Block import/Block import ERC20 transfers",
            "value": 165210554184,
            "range": "± 580295787",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "git@edgl.dev",
            "name": "Edgar",
            "username": "edg-l"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ec00e544af6f1d39f61a811d279c7be3b137b5f",
          "message": "perf(l1): save chain config in vm,  cache latestblock header (#3878)\n\n**Motivation**\n\nOn samply, most of the work seems to be in validate_tx on the rpc.\n\nInside validate tx, get_block_header took 22% and latest_block_number\nabout 3%.\nThis pr turns those numbers into 0.2%. Now get account info takes 93%. \n\nPR based on #3865\n\nBefore (pr #3865):\n<img width=\"1008\" height=\"188\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/4de19499-aa0c-4a07-84ab-3d6c1403e6ea\"\n/>\n\n\nAfter:\n<img width=\"968\" height=\"194\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/3dba9a15-710d-4e8a-986e-273c23deb587\"\n/>\n\n\nmake load-test\n\nPrev(perf/aux_sender_nonce_mempool) gigas of first 4 blocks:\n0.6, 0.5, 0.4, 0.4, 0.4\n\nPR Gigagas of first 4 blocks:\n0.7, 0.56, 0.43, 0.42,\n\n\nSlightly improves ethrex transfers bench on gas benches (7.82%).\n\n---------\n\nCo-authored-by: Mario Rugiero <mrugiero@gmail.com>",
          "timestamp": "2025-08-05T15:07:41Z",
          "tree_id": "9d45a300c14ddfb48c171b18ff458dce16adf818",
          "url": "https://github.com/julienbrs/ethrex/commit/5ec00e544af6f1d39f61a811d279c7be3b137b5f"
        },
        "date": 1754409856108,
        "tool": "cargo",
        "benches": [
          {
            "name": "Block import/Block import ERC20 transfers",
            "value": 163550961383,
            "range": "± 471241562",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}