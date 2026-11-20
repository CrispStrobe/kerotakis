# Salt and oxide boiling/decomposition candidates (reviewed, not yet in the registry)

Status: candidates only. Landing them means registry-source entries plus regenerating the golden registry and coverage files, which needs a build. CaO, MgO and SiO2 stay without a boiling point.

# Boiling / decomposition points: primary-source findings (2026-11-20)

Method: free full texts only (archive.org scans of US Bureau of Mines publications, NIST SRD reprint). Page images viewed to confirm every digit quoted. No repo edits, no builds, no jobs. Circular 500: its scanned text layer is too poor to locate these compounds, and the Circular 500 tables list fusion for MgO etc. but no boiling point, so no OCR was run.
Licence for all three sources below: US Government publications (17 USC 105), public domain (Stern: reprint notice "(c) 1972 The U.S. Secretary of Commerce on behalf of the United States").
Kelley's "B.P." values are in kelvin (his degree sign means absolute): checked with dH/dS = T on the KCl line (38,840/23.1 = 1,681 K).

## Summary table
| Substance | Value | Status |
|---|---|---|
| KCl | 1680 K (1407 C) | FOUND, Kelley 1935, calculated from the adopted vapor-pressure fit to Fiock & Rodebush (liquid 1,179-1,378 K), extrapolated ~300 K |
| NaOH | 1651 K (1378 C) | FOUND, Kelley 1935, single data set (Von Wartenberg & Albrecht, 1,283-1,675 K), Kelley calls it "erratic... a simple vapor-pressure relationship" |
| CaCl2 | 1600 C (1873 K) | FOUND but BRACKETED = extrapolated/uncertain, Maier 1925 |
| BaCl2 | 1562 C (1835 K) | FOUND but BRACKETED = extrapolated/uncertain, Maier 1925 |
| CaO | not found | Kelley gives only a solid->gas equation, no bp |
| MgO | not found | Kelley explicitly omits equations (data unusable) |
| SiO2 | not found (reliable) | Kelley gives only an "approximate" equation |
| Pb(NO3)2 | decomposes, 539-632 K onset range of first step | FOUND, Stern 1972 (via Neumann & Sonntag) |
| AgNO3 | decomposes, appreciable ~30-40 K above mp 483 K (~513-523 K) | FOUND qualitative, Stern 1972 |

## KCl
- Normal boiling point 1680 K = 1406.9 C (Kelley prints "B. P. = 1,680").
- Citation: K. K. Kelley, "Contributions to the Data on Theoretical Metallurgy. III. The Free Energies of Vaporization and Vapor Pressures of Inorganic Substances", U.S. Bureau of Mines Bulletin 383 (1935), p. 86 (section "Potassium", subsection "Chloride", equation block "Liquid -> Gas (KCl)": dH = 50,600 - 7.0T; dF = 50,600 + 16.1 T log T - 82.04 T; B.P. = 1,680; dH_1680 = 38,840; dS_1680 = 23.1). Printed page 86 = leaf 88 of the scan. https://archive.org/details/DTIC_ADB184291 (PDF https://archive.org/download/DTIC_ADB184291/DTIC_ADB184291.pdf). Also HathiTrust record https://catalog.hathitrust.org/Record/011397801.
- It is the normal boiling point of the liquid (total vapor pressure 1 atm). Data adopted: Fiock & Rodebush (1,179-1,378 K), judged by Kelley "by far the best"; seven other investigators cited (up to 1,691 K, Von Wartenberg & Albrecht 1921 Z. Elektrochem. 27, 162). Vapor composition: Kelley treats the gas as a single species "(KCl)" (monomer) with Cp(g)=9.0; he does not discuss dimers on this page. Caveat: monomer assumption (KCl vapor has some dimer fraction at the bp), so this is a bp of the equation as written.

## NaOH
- Normal boiling point 1651 K = 1377.9 C (Kelley prints "B. P. = 1,651"; log P(atm) = -7,519/T + 4.553 -> P=1 at T = 1,651.5 K, consistent).
- Citation: Kelley, Bull. 383 (1935), p. 97 (section "Sodium", subsection "Hydroxide", "Liquid -> Gas (NaOH)"). Leaf 99 of the archive.org scan (same URL as above).
- Caveat: only one measurement set (Von Wartenberg & Albrecht, Z. Elektrochem. 27, 162 (1921), 1,283-1,675 K); Kelley: "results are erratic and warrant no more than a simple vapor-pressure relationship". Vapor species written NaOH (monomer); real vapor of NaOH contains substantial (NaOH)2 dimer, so treat as an effective value. It agrees to 10 K with the usual 1388 C (1661 K) quoted from CRC (which I could not verify against a primary).

## CaCl2
- 1600 C (1873 K), printed in square brackets as "[1,600].0". Brackets in the Bureau of Mines tables mean "extrapolated values or uncertain values" (Maier, text before Tables 24-29).
- Citation: C. G. Maier, "Vapor Pressures of the Common Metallic Chlorides and a Static Method for High Temperatures", U.S. Bureau of Mines Technical Paper 360 (1925), Table 31 "Normal boiling points, arranged with decreasing temperatures", p. 51 (continues p. 52); PDF page 57 of https://archive.org/details/pub_usgov-bmines-technical-paper_0360 (Google-digitized from Univ. of Michigan copy, marked Public Domain). Experimental section for CaCl2 p. 27.
- Caveat: Maier measured NO vapor pressure for CaCl2 up to 1,137 C (< 0.4 cm Hg), so 1600 C is an extrapolation, not a measurement. Treat as an estimate-grade historical value; I could not find a better primary source.

## BaCl2
- 1562 C (1835 K), printed "[1,562]" (bracketed = extrapolated/uncertain). Same Maier Table 31, p. 51.
- Maier's BaCl2 data (Table 9, p. 26, Figure 7) cover only about 1,329-1,487 K (four points) and he states they are "probably not dependable" (vapor pressure extremely low). So the value is an extrapolation of poor data.
- Secondary (NOT verified to a primary): Wikipedia/CAMEO list 1560 C, consistent with Maier within 2 K; likely derived from the same lineage. I did not find an independent measurement.

## CaO
- Not found. Kelley Bull. 383 p. 31 ("Oxide", leaf 33) gives only a SOLID -> gas equation from Claasen & Veenemans (Z. Physik 80, 342 (1933), 1,600-1,750 K... Kelley's range printed as 1,600-1,750; Ruff & Schmidt 2,190-2,910 judged "very erratic"): log P(atm) = -27,400/T + 6.89. Heat and free-energy equations omitted "because of uncertainties". Solving for P=1 atm would give about 3977 K, which is far beyond the data range and beyond the CaO melting point (so physically meaningless); I do NOT offer it as a boiling point. Kelley's page has no B.P. line for CaO. Wikipedia's 2850 C is the melting point range/GESTIS value at 100 hPa, unverified.

## MgO
- Not found. Kelley Bull. 383 (section "Magnesium", "Oxide") says Ruff & Schmidt (2,223-3,033 K) results "are very uncertain, the same pressure being recorded at temperatures differing by as much as 600 C, consequently equations for the substance are omitted." (leaf near MgCl2 text; located by text, not image-verified). So the primary compilation refuses to give a bp. The commonly listed 3600 C (CRC via Wikipedia/PubChem/NIOSH) is not traced to a primary here; reported as secondary only.

## SiO2
- No reliable primary bp. Kelley Bull. 383 p. 90 (leaf 92), "Oxide": Ruff & Schmidt measured 2,057-2,603 K "results very erratic, so only an approximate vapor-pressure equation is included": Liquid -> Gas ((SiO2)n): log P(atm) = -19,130/T + 7.65 (image-verified). Taken literally P=1 atm at T = 19,130/7.65 = 2501 K (2228 C), which matches the 2230 C figure quoted in ATSDR, but Kelley's species "(SiO2)n" is nominal, whereas the real vapor is SiO + 1/2 O2 (dissociation), and the CRC/Wikipedia value is 2950 C (3223 K). The two differ by ~720 K, which is exactly the vapor-composition ambiguity. Recommendation: do not set a SiO2 bp; if one is wanted, record 2501 K only as "Ruff-Schmidt/Kelley approximate fit, total pressure of undefined gas", flagged low confidence. This is a calculation from Kelley's printed equation, not a printed bp.

## Pb(NO3)2 (decomposition; no melting, it decomposes first)
- Source: K. H. Stern, "High Temperature Properties and Decomposition of Inorganic Salts. Part 3. Nitrates and Nitrites", J. Phys. Chem. Ref. Data 1 (3), 747-772 (1972), doi:10.1063/1.3253104; free reprint https://srd.nist.gov/jpcrdreprint/1.3253104.pdf; section "Lead", pp. 758-759.
- Decomposition proceeds through basic nitrates (reversible equilibria measured by Neumann & Sonntag, Z. Elektrochem. 39, 799 (1933), Stern ref [78]; Stern's OCR-adjacent text cites it with [23,78,120]):
  I  539-632 K: 3 Pb(NO3)2 = [Pb(NO3)2.2PbO] + 4 NO2 + O2
  II 625-719 K: 2 [Pb(NO3)2.2PbO] = [Pb(NO3)2.5PbO] + 2 NO2 + 1/2 O2
  III 688-809 K: [Pb(NO3)2.5PbO] = 6 PbO + 2 NO2 + 1/2 O2
  Overall to PbO, NO2, O2 (Stern dH ~ 337 kJ for the first step). Pb stays Pb(II); a nitrite intermediate is not established. A TGA study (Patil, Gosavi, Rao 1972) claimed final product PbO1.4 "without details".
- Onset caveat: the 539 K is the lower end of the equilibrium pressure measurement range, not a strict thermogravimetric onset; no single decomposition temperature is given by Stern. Commonly quoted ~470 C total decomposition is not in this source.

## AgNO3 (decomposition; melts at 483 K = 210 C)
- Source: Stern 1972 (same reprint), section "Silver", pp. 766-768. Melting (C beta -> I) 483 K, dH 12.49 kJ/mol (ref [62], Janz). Stern: "Qualitatively, decomposition is negligible in the solid state, but becomes appreciable 30-40 degrees above the melting point [34]" (Centnerszwer & Blumenthal, Bull. Intern. Acad. Polon. Sci. A 482 (1936)). So onset ~513-523 K (240-250 C), qualitative only.
- Products given as reactions (all at 298.15 K standard data; p. 767): (1) 2 AgNO3(c) = Ag2O(c) + 2 NO2(g) + 1/2 O2(g), dH = 283.93 kJ; (2) 2 AgNO3(c) = 2 Ag(c) + 2 NO2(g) + O2(g), dH = 314.97 kJ. Ag2O is itself unstable above ~400 K (Stern), so the final solid at these temperatures is metallic Ag; NO is also in the gas equilibrium (2 NO2 = 2 NO + O2). Partial-pressure table p. 767 (e.g. 600 K: total ~2.1e-1 atm in his model).
- Caveat: Stern's AgNO2 line "decomposes at 120 C to Ag + NO2" is for silver NITRITE, not nitrate. The CRC "440 C decomposes" value (Wikipedia) is not in Stern and is not verified here.

## Items checked and not usable
- NIST-JANAF: melting only (as stated); derived bp from liquid-vs-gas rejected as instructed.
- Stull 1947 (Ind. Eng. Chem. 39, 517) and Kelley's later bulletins: not freely accessible, not checked.
- Circular 500: text layer poor; no bp for NaOH/CaCl2 per brief. KCl/BaCl2/MgO/CaO/SiO2 not located in the text layer; image OCR not run (load ~5-6).

## Suggested registry citation shape (for the entries that exist)
KCl and NaOH: Kelley 1935 Bull. 383 pp. 86 / 97, URL https://archive.org/details/DTIC_ADB184291, licence public domain (US Government work). CaCl2/BaCl2: Maier 1925 TP 360 Table 31 p. 51, URL https://archive.org/details/pub_usgov-bmines-technical-paper_0360, licence public domain, note "bracketed = extrapolated". Pb(NO3)2/AgNO3: Stern 1972 JPCRD 1, 747, URL https://srd.nist.gov/jpcrdreprint/1.3253104.pdf.
