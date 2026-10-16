# Complete-stock acidic oxalate follow-up

The [prospective design](design.json) changes preparation explicitly to molecular
H2SO4, H2C2O4 and KMnO4 at60°C in finite sealed headspace. It follows the first
analytical control's charge/phase boundaries; it is neither an oxalate-ion alias
nor a replay of the original25°C ionic feed.

One API request completes in4.74s. The [assessment](results/assessment.json)
meets all eight frozen checks: permanganate consumed,10mmol reactionCO2,
10mmol retained carbon increment,2mmolMn,2mmolK,10mmolS,finalpH1.292<2 and
zero solver failures. All retained element/formal-charge residuals are below
6e-15mol. No model-boundary warning occurs. Initial trapped carbon and its small
repartition are accounted separately from the reaction event.

[Raw receipts and hashes](results/sha256.json) retain enginec446/binary670bd3 and
changed gateway266121dd bindings. Use the sibling
[saved reader](../prepared-basis-first2/assess_saved.py) to reproduce the assessment.
No calibrated initiation temperature, reaction rate, heat or empirical yield is
established. Original missing C2O4-- identity and first refusal remain open;
catalog selection requires separate review.
