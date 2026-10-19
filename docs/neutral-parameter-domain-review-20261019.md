# Neutral parameter-source compatibility review

This is source review, with no parameter import or runtime change. Finish current
PR841 acceptance/delivery before extending chemical coverage; use
[the ordered neutral lane](post-hydrolysis-gap-lanes-20261019.md#1-neutral-partition-and-chromatography-parameter-coverage).

The [coauthor-hosted original1979 revision](https://www.researchgate.net/profile/Baerbel-Kolbe/publication/231297193_Vapor-Liquid_Equilibria_by_UNIFAC_Group_Contribution_Revision_and_Extension/links/5a7700c1aca2722e4deff048/Vapor-Liquid-Equilibria-by-UNIFAC-Group-Contribution-Revision-and-Extension.pdf?origin=publication_detail)
provides important compatibility limits (printed pages719–720). It distinguishes
older water/alkane parameters fitted to mutual solubility from revised values
fitted to vapor/liquid data; the authors favor the older values for water/alkane
mixtures. It also changes the higher-alcohol group definition. Therefore a newer
matrix is not an interchangeable upgrade for the app's water/hexane layer model.
The introductory300–425K range describes that publication's method context; it
does not independently validate the app's mixed-source table or room-temperature
chromatographic predictions. TableII specifies rows as the first interaction
index and columns as the second; both directions must be independently checked.
This review read the original paper's prose through the browser, but did not
recover a locally inspectable original table image or verified numerical cells.
No values were inferred from the interleaved OCR.

Next actions:

1. Preserve the existing approved table and independent reference fixture. Record
   their exact source bindings before any proposed extension.
2. Obtain reliable original table layout and subgroup definitions for the chosen
   revision. Independently transcribe each required cell with direction, page,
   parameter vintage and source terms; compare existing overlapping rows.
3. Decide explicitly which published model domain owns water/hexane layer
   detection, infinite-dilution solute transfer and chromatography. A parameter
   extension must not silently swap all three to a different fitted domain.
4. Review attribution for the current isolated OH subgroup and its surface/volume
   parameters against the publication that introduced that definition. Do not
   label a complete mixed-vintage matrix as the1975 table without per-row proof.
5. Only then propose complete compatible ester subgroup and directional data with
   provenance under CONTRIBUTING. Keep missing cells unsupported. Add independent
   coefficients, reference comparisons, owned transfer and actual app controls
   before a separately frozen unchanged MO011 replay.

The proprietary Consortium export remains excluded. A coauthor's accessible
paper supports primary-source review; accessibility alone is not a new grant to
redistribute the entire article or an unrelated modern matrix.
