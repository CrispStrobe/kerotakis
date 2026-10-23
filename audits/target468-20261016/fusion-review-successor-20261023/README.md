# Independent review and saved-reader successor

The original four-package component run on `51bc28dd` discovered **3,047 tests in 342 binaries: 3,044 passed and three genuinely failed**. Formatting and Clippy passed. The failures were the scene golden and two colligative expectations; the independently justified fixture correction on `6164acd8` now [passes all seven affected-target controls](../water-fusion-fixture7-focused-accepted-20261023/README.md). Runtime code is unchanged between those revisions. Neither this composition nor the original failed run is called a uniform green replay.

The original producer omitted two standard ` - should panic` rows in its accounting parser. Its first output remains preserved. `libtest_rows.py` accepts that standard spelling while preserving failed results and enforcing exact discovered identities and totals. `check_actual_parser.py --evidence ORIGINAL_SAVED_COLLECTION` read the actual 342 saved stdout/list pairs: all 3,047 identities accounted for, exactly the original three failures, no ignored tests. Negative controls change a real expected-panic result to failure and remove/duplicate its row; failures remain failures and incomplete accounting is rejected. No test binary or archive body is executed/read by this reader.

The component review authenticated all 2,450 nonarchive members and the original manifest/collection commitments. Its archive metadata is still producer-declared until separate remote byte verification completes. This is an explicit pending gate, not full component acceptance.

`verify_aggregate.py --repo-root .` independently checks both earlier public capsules, actual commit-to-tree resolution, exact source-pin bytes and global uniqueness across the 51 original focused controls. The individual frozen capsules remain unchanged. Independent review receipts describe their actual scope; public projections do not claim to contain the complete private raw collections.

Full own-fleet, both platforms, protected CI, normal integration and changed-release API/browser acceptance remain pending. Original qualified coverage remains **338/468**, requiring 63 further earned originals to reach 401.
