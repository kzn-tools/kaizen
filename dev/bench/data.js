window.BENCHMARK_DATA = {
  "lastUpdate": 1766682570423,
  "repoUrl": "https://github.com/kzn-tools/kaizen",
  "entries": {
    "Kaizen Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f2bbae49e200d366ffcb514b753c070face8258",
          "message": "Merge pull request #11 from kzn-tools/feature/licensing-module\n\nfeat(core): add licensing module with HMAC validation",
          "timestamp": "2025-12-20T09:26:45+01:00",
          "tree_id": "ad3f9cba8c4e50a25b7131f844fbd14a30d90e7a",
          "url": "https://github.com/kzn-tools/kaizen/commit/3f2bbae49e200d366ffcb514b753c070face8258"
        },
        "date": 1766219496682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 394020,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65831,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35474,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 276880,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 276660,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 260880,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1794700,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 23228000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2330800,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 5820900,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 11715000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 23267000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 105550,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 150660,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 37.727,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "771af8a0351297c34ca3a6521ad992244b940b51",
          "message": "Merge pull request #12 from kzn-tools/feature/licensing-module\n\nfeat(cli): integrate licensing in check command",
          "timestamp": "2025-12-20T09:50:36+01:00",
          "tree_id": "b588a19a5b95b8dcae927ebcac2bc4d751688780",
          "url": "https://github.com/kzn-tools/kaizen/commit/771af8a0351297c34ca3a6521ad992244b940b51"
        },
        "date": 1766220860961,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 388880,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66365,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34272,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 274500,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 277300,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 263130,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1773300,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 24434000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2418400,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6102600,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12238000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 24827000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 98377,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 149270,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.126,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a314c8eea618952bbd9a80d08570679229567e41",
          "message": "Merge pull request #13 from kzn-tools/feature/licensing-module\n\nfeat(lsp): integrate licensing in LSP server",
          "timestamp": "2025-12-20T10:11:38+01:00",
          "tree_id": "ec17fb8b27d7c17f8ece436d89e786d57039f5a9",
          "url": "https://github.com/kzn-tools/kaizen/commit/a314c8eea618952bbd9a80d08570679229567e41"
        },
        "date": 1766222130842,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 388890,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65054,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34010,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 272670,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 273500,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 261170.00000000003,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1763800,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 23503000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2317800,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 5787600,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 11593000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 23513000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 102870,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 146720,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.37,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d44f236a45ee41f99720aec5015c29025d7ce94",
          "message": "Merge pull request #14 from kzn-tools/feature/auth-command\n\nfeat(cli): add kaizen auth command",
          "timestamp": "2025-12-20T10:26:19+01:00",
          "tree_id": "386004dfa456890f109cbde26f858a27059fca46",
          "url": "https://github.com/kzn-tools/kaizen/commit/5d44f236a45ee41f99720aec5015c29025d7ce94"
        },
        "date": 1766223002095,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 393630,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66261,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35039,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 281720,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 287550,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 266450,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1920000,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 23508000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2440700,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 5846800,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12194000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25309000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 114800,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 151250,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.31,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "34ad92dde687cd96bb8df9be8c76c898a312f476",
          "message": "Merge pull request #15 from kzn-tools/feature/tier-filtering\n\nfeat(rules): add tier-based rule filtering",
          "timestamp": "2025-12-20T11:00:23+01:00",
          "tree_id": "1f7b3821806bc1cf0b3eab6673e72a4ac2a69897",
          "url": "https://github.com/kzn-tools/kaizen/commit/34ad92dde687cd96bb8df9be8c76c898a312f476"
        },
        "date": 1766225053973,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 390200,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65301,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35195,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 272570,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 273560,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 257300,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1776000,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 22873000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2332100,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 5731400,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12423000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 24713000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 103380,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 145080,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 39.466,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a5f56e287051ec6df9ce997a1018547721eadd96",
          "message": "Merge pull request #16 from kzn-tools/feature/prototype-pollution\n\nfeat(rules): implement S020 no-prototype-pollution rule",
          "timestamp": "2025-12-20T11:16:17+01:00",
          "tree_id": "6ff2f0ada770c68239f2a428e2742910b3f7ca77",
          "url": "https://github.com/kzn-tools/kaizen/commit/a5f56e287051ec6df9ce997a1018547721eadd96"
        },
        "date": 1766226000851,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 396940,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66193,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35234,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 299960,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 299270,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 285460,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1782200,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25094000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2489600,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6209400,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12999000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 26924000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 104060,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 161470,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.305,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a7919d0fa83b78a19e991898598d68312217bf28",
          "message": "Merge pull request #17 from kzn-tools/feature/redos\n\nfeat(rules): add S021 no-redos rule for ReDoS detection",
          "timestamp": "2025-12-20T11:38:46+01:00",
          "tree_id": "56cd3f6fcc3e02026a0bf2170af81baf4f4d42c6",
          "url": "https://github.com/kzn-tools/kaizen/commit/a7919d0fa83b78a19e991898598d68312217bf28"
        },
        "date": 1766227348581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 396350,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66062,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34871,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 301010,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 300870,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 286740,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1829800,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25124000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2607400,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6458300,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12858000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25623000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 105640,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 162720,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.316,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "af1adf5c59d0a6441b71ca6d68be45d701cb7a1a",
          "message": "Merge pull request #18 from kzn-tools/feature/unsafe-deserialization\n\nfeat(rules): add S022 no-unsafe-deserialization rule",
          "timestamp": "2025-12-20T16:17:53+01:00",
          "tree_id": "954bd3bedcff9be507abf829b0ae9d1393a2d1d4",
          "url": "https://github.com/kzn-tools/kaizen/commit/af1adf5c59d0a6441b71ca6d68be45d701cb7a1a"
        },
        "date": 1766244097342,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 392000,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65028.00000000001,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34385,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 298980,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 300530,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 284830,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 1829700,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25260000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2548600,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6269000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12650000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25770000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 105770,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 162580,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.462,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "42c3c2f7e35653c10055798692c100ef73e543e5",
          "message": "Merge pull request #19 from kzn-tools/fix/reduce-false-positives\n\nfix: reduce false positives in linting rules",
          "timestamp": "2025-12-21T12:43:38+01:00",
          "tree_id": "f9d066b791b9d067b5743b3c9aa47b91715f793b",
          "url": "https://github.com/kzn-tools/kaizen/commit/42c3c2f7e35653c10055798692c100ef73e543e5"
        },
        "date": 1766317633984,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 394840,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66524,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35002,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 300650,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 304490,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 269940,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2129000,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25111000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2537900,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6312400,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12645000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 26112000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 140370,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 161550,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.689,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8fe2b70563db9939740c748d9515bbf22c531d6b",
          "message": "Merge pull request #20 from kzn-tools/fix/more-parse-errors\n\nfix: enable decorators_before_export and fn_bind in parser",
          "timestamp": "2025-12-21T12:57:35+01:00",
          "tree_id": "2c084be0a21d3944a472094e3b029737f1606b6e",
          "url": "https://github.com/kzn-tools/kaizen/commit/8fe2b70563db9939740c748d9515bbf22c531d6b"
        },
        "date": 1766318487528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 395320,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 66658,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35240,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 299020,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 304150,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 266210,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2119700,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 24993000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2569700,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6331100,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12425000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25270000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 141100,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 164450,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.533,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0167765ea67429127341ecaa9ec49a301a48c725",
          "message": "Merge pull request #21 from kzn-tools/fix/q004-switch-false-positive\n\nfix: Q004 no longer flags code after switch with break as unreachable",
          "timestamp": "2025-12-21T13:32:20+01:00",
          "tree_id": "9c67c8e2b328ae55437dc9f6df1d04ae03e75cb3",
          "url": "https://github.com/kzn-tools/kaizen/commit/0167765ea67429127341ecaa9ec49a301a48c725"
        },
        "date": 1766320561248,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 401840,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 64950,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34826,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 326010,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 335760,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 270720,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2035499.9999999998,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 26220000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2593300,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6454000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12619000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25957000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 128220,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 169960,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 32.215,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "75523e544dd6843f2e1f79944e592a7d5e3d969b",
          "message": "Merge pull request #23 from kzn-tools/refactor/cloud-api-validation\n\nrefactor: migrate to cloud-based API key validation",
          "timestamp": "2025-12-23T16:35:38+01:00",
          "tree_id": "ebdbd1dd041c21ac2231bfd18524169f5de8ee73",
          "url": "https://github.com/kzn-tools/kaizen/commit/75523e544dd6843f2e1f79944e592a7d5e3d969b"
        },
        "date": 1766504355421,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 389050,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65455,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34196,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 304760,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 307660,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 275240,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2159900,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25324000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2622600,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6561700,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 13012000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 26048000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 143800,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 167800,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.36,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f087e3391c95ba121a0874a98648d2a938123551",
          "message": "Merge pull request #25 from kzn-tools/chore/remove-editors-folder\n\nchore: remove editors folder (migrated to separate repos)",
          "timestamp": "2025-12-23T16:45:27+01:00",
          "tree_id": "ad25c464eed05d5d04b3269bb364d0c12541e413",
          "url": "https://github.com/kzn-tools/kaizen/commit/f087e3391c95ba121a0874a98648d2a938123551"
        },
        "date": 1766504948802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 392280,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 65471,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 34910,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 301690,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 306600,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 272850,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2180800,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25720000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2665100,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6569300,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12940000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25965000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 142700,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 163100,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 36.511,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "goore.csmoviz@gmail.com",
            "name": "Mathieu",
            "username": "mpiton"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ece11f4f45f72198f2f8856b4e6badc6346e88ad",
          "message": "Merge pull request #29 from kzn-tools/feature/oauth-device-flow\n\nfeat(auth): OAuth Device Flow for browser-based authentication",
          "timestamp": "2025-12-25T18:05:47+01:00",
          "tree_id": "bba7b58d60b5b1a3394d92f0cd4040e9812d6905",
          "url": "https://github.com/kzn-tools/kaizen/commit/ece11f4f45f72198f2f8856b4e6badc6346e88ad"
        },
        "date": 1766682569904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "parsing/parse_500_loc",
            "value": 394090,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_tsx_component",
            "value": 67086,
            "unit": "ns"
          },
          {
            "name": "parsing/parse_typescript_interfaces",
            "value": 35436,
            "unit": "ns"
          },
          {
            "name": "rules/quality_rules",
            "value": 305710,
            "unit": "ns"
          },
          {
            "name": "rules/security_rules",
            "value": 307280,
            "unit": "ns"
          },
          {
            "name": "rules/clean_code",
            "value": 270810,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_500_loc",
            "value": 2149700,
            "unit": "ns"
          },
          {
            "name": "analysis/analyze_100_files",
            "value": 25618000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/10",
            "value": 2603800,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/25",
            "value": 6362100,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/50",
            "value": 12628000,
            "unit": "ns"
          },
          {
            "name": "analysis/project_size/100",
            "value": 25901000,
            "unit": "ns"
          },
          {
            "name": "latency/p95_500_loc_parse_analyze",
            "value": 146080,
            "unit": "ns"
          },
          {
            "name": "latency/p95_per_file_100_files",
            "value": 166880,
            "unit": "ns"
          },
          {
            "name": "memory/100_files_retained",
            "value": 41.852,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}