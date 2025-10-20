# Benchmark Report
- Timestamp: 2025-10-20 17:23:17
- Host: Windows 11 (AMD64), CPU: Intel64 Family 6 Model 167 Stepping 1, GenuineIntel
- Rust: rustc 1.90.0 (1159e78c4 2025-09-14)
- Python: Python 3.14.0
- Rust bench env: JR_MIN_BYTES=1048576, JR_MEAS_SEC=3, JR_WARMUP_SEC=1, JR_SAMPLE_SIZE=10
- Metrics: mean(s) (lower is better), throughput (higher is better)

## Container Cases (strict)
| case | size(bytes) | python | jsonrepair(strict) | llm_json(strict) |
|---|---|---|---|---|
| array_dense/100000 | 1048576 | - | 0.0198678 (50.33 MiB/s) | 0.0173458 (57.65 MiB/s) |
| array_spaces/1024 | 1048576 | 0.236706 (4.22 MiB/s) | 0.00436576 (229.06 MiB/s) | 0.00888843 (112.51 MiB/s) |
| array_spaces/64 | 1048576 | 0.27537 (3.63 MiB/s) | 0.00403507 (247.83 MiB/s) | 0.00690911 (144.74 MiB/s) |
| array_spaces/8192 | 1630899 | 0.00109088 (1425.77 MiB/s) | 0.00147447 (1054.85 MiB/s) | 0.00100351 (1549.91 MiB/s) |
| fence_jsonp/fixed | 1048576 | 1.11763 (916.22 KiB/s) | 0.000152926 (6539.12 MiB/s) | 0.0078373 (127.59 MiB/s) |
| flat_object/10000 | 1048576 | - | 0.0135515 (73.79 MiB/s) | 0.00974357 (102.63 MiB/s) |
| mixed_comments/200 | 1048576 | 2.24373 (456.38 KiB/s) | 0.0110759 (90.29 MiB/s) | 0.00662798 (150.88 MiB/s) |
| mixed_comments/50 | 1048576 | 1.93961 (527.94 KiB/s) | 0.0108003 (92.59 MiB/s) | 0.0065396 (152.91 MiB/s) |
| ndjson/500 | 1048576 | 1.47132 (695.97 KiB/s) | 46.3881 (22.07 KiB/s) | 0.00506005 (197.63 MiB/s) |
| nested_object/16 | 1048576 | - | 0.0217459 (45.99 MiB/s) | 0.00521718 (191.67 MiB/s) |
| object_newlines/1 | 1048576 | 1.5844 (646.30 KiB/s) | 0.0366781 (27.26 MiB/s) | 0.00582924 (171.55 MiB/s) |
| object_newlines/64 | 1048576 | 0.679952 (1.47 MiB/s) | 0.00634912 (157.50 MiB/s) | 0.00610625 (163.77 MiB/s) |
| object_newlines/8 | 1048576 | 1.30729 (783.30 KiB/s) | 0.0208 (48.08 MiB/s) | 0.00639917 (156.27 MiB/s) |
| strings_unicode/1000 | 1048576 | - | 0.0172645 (57.92 MiB/s) | 0.00632363 (158.14 MiB/s) |
| trailing_commas/fixed | 1048576 | - | 34.5699 (29.62 KiB/s) | 0.00515813 (193.87 MiB/s) |
| typical/fixed | 1048576 | 1.7805 (575.12 KiB/s) | 0.0180512 (55.40 MiB/s) | 0.00499805 (200.08 MiB/s) |
| unicode_comments/fixed | 1048576 | 2.21125 (463.09 KiB/s) | 0.019771 (50.58 MiB/s) | 0.00540291 (185.09 MiB/s) |

## Valid JSON Cases (strict vs fastpath)
| case | size(bytes) | jsonrepair(strict) | jsonrepair(fast) | llm_json(strict) | llm_json(fast) |
|---|---|---|---|---|---|
| valid_json/fixed | 1048576 | 0.0182742 (54.72 MiB/s) | - | 0.00520408 (192.16 MiB/s) | - |
| valid_json_ensure_ascii/fixed | 1048576 | 0.0240645 (41.56 MiB/s) | - | - | - |
| valid_json_fastpath/fixed | 1048576 | - | 0.000320009 (3124.91 MiB/s) | - | 0.00490318 (203.95 MiB/s) |

## Stream Cases
| case | size(bytes) | jsonrepair(stream) |
|---|---|---|
| stream/ndjson_1000_lines | 1048576 | 0.000589298 (1696.93 MiB/s) |

