# Baseline tree equivalence clarification

The source-attribution addendum was intentionally conservative before the source binding was checked. The later authenticated binding for the 155-control focus and full native CI records tested merge `37bb699f26ec8921af5848f5d3a4c973a44d0cdc`, parents `ccf30b2aeac4042ffe79942d4c8746ec10a553ef` and `6e44feab03a646b412c123b21b275e3b95942931`, and tree `e4aa4c8dad734b370c77100ea47a1c201d3c4a50`. That tree is exactly the tree of source commit `6e44feab03a646b412c123b21b275e3b95942931`.

Therefore the two native failures were run on a distinct merge commit whose complete source tree is identical to the diagnostic base. The diagnostic run still records actual outcomes without assuming they reproduce, because the added tracing commit changes the source tree and the output is evidence rather than acceptance.
