import importlib.util, json, pathlib, tempfile, unittest

PATH=pathlib.Path(__file__).with_name('i18n-engine-vocabulary-lint.py')
spec=importlib.util.spec_from_file_location('vocab_lint',PATH); lint=importlib.util.module_from_spec(spec); spec.loader.exec_module(lint)

class VocabularyGate(unittest.TestCase):
    def test_current_tree_is_covered(self): self.assertEqual([],lint.audit())
    def test_sources_stay_connected_to_term_substitution(self):
        chip=(lint.ROOT/'web/app/src/lib/components/SpeciesChip.svelte').read_text()
        vessel=(lint.ROOT/'web/app/src/lib/components/Vessel.svelte').read_text()
        session=(lint.ROOT/'web/app/src/lib/session.svelte.ts').read_text()
        for needle in ('t(item.appearance)','hazards.map((h) => t(h))'): self.assertIn(needle,chip)
        for needle in ('t(layer.colour_word)','t(layer.name)','t(apparatusTitle)'): self.assertIn(needle,vessel)
        self.assertIn('t(missionTitle(name))',session)
    def test_a_new_emittable_species_fails(self):
        with tempfile.TemporaryDirectory() as d:
            root=pathlib.Path(d)
            for rel in ('data/registry','web/app/src/locales','crates/kerotakis-core/src','crates/kerotakis-core/i18n','crates/kerotakis-safety/src','lessons'):(root/rel).mkdir(parents=True,exist_ok=True)
            for rel in ('data/registry/registry-source-v1.json','web/app/src/locales/de.json','crates/kerotakis-core/src/appearance.rs','crates/kerotakis-core/i18n/de.toml','crates/kerotakis-safety/src/lib.rs'):
                src=lint.ROOT/rel; (root/rel).write_bytes(src.read_bytes())
            registry=json.loads((root/'data/registry/registry-source-v1.json').read_text()); registry['identities'].append({'name':'untranslatedium'})
            (root/'data/registry/registry-source-v1.json').write_text(json.dumps(registry))
            self.assertTrue(any('untranslatedium' in p for p in lint.audit(root)))

if __name__=='__main__': unittest.main()
