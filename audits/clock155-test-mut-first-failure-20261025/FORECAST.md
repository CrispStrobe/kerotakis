# Clock integration fixture compiler stop —2026-10-25

Source d01bb090 passes formatting but strict all-target Clippy reports three unnecessary mutable GasExchangeClock bindings in clock_co2_boundary.rs (lines311,387,576). No tests ran: all155 and downstream wasm/CLI/full1536 remain NOT_REACHED. The attached stderr is a public projection, with only the producer checkout prefix replaced; its raw hash is preserved in binding.json.

Before editing, freeze the expected repair: remove only these three `mut` keywords. Clock::advance takes an immutable receiver; fixture inputs, assertions, test identities, physical budgets, compiler flags and lock stay identical. Require one separately source-bound successor retaining all155 controls/16targets, three CLI protocols and full1536 sweep. A compiler correction does not earn scientific or product acceptance.
