# First finite-molar gas native acceptance

Exact source `8eece0b1`, tree `308bc297`, passes Clippy and all524 PHREEQC
native tests across97 targets, with zero failures or ignored tests. Both new
finite-molar controls pass. Raw streams, uploaded worker identity, actual source
and producer lock are independently authenticated. The original producer lock
bytes and complete native stdout are included; stderr hashes preserve the full
original streams without publishing host details.

This is the first source's native acceptance, not combined-source or full-fleet
acceptance. PR833 subsequently incorporated the normally merged phase repair
`0a5a92f1`; current head `e9cc8782` needs its own combined validation. Its first
combined worker stopped before compilation because extra native checks preceded
recursive submodule initialization. No experiments ran. A corrected worker
initializes those same source submodules before unchanged native commands and
all33 original logical fleet jobs. No source/assertion/forecast changes hide the
setup failure. No catalog selection or empirical calibration is added.
