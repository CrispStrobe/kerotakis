export type Locale = "en" | "de";

type Vars = Record<string, string | number>;

/** Domain vocabulary returned by the engine rather than authored in components. */
const DE_TERMS: Record<string, string> = {
  water: "Wasser", ethanol: "Ethanol", "sodium chloride": "Natriumchlorid",
  "silver nitrate": "Silbernitrat", "silver chloride": "Silberchlorid",
  "hydrogen peroxide": "Wasserstoffperoxid", catalase: "Katalase",
  "manganese dioxide": "Mangandioxid", "sodium sulfite": "Natriumsulfit",
  "sodium thiosulfate": "Natriumthiosulfat", sulfur: "Schwefel",
  "sulfur dioxide": "Schwefeldioxid", "slaked lime (calcium hydroxide)": "Löschkalk (Calciumhydroxid)",
  "copper(II) hydroxide": "Kupfer(II)-hydroxid", "copper(II) oxide": "Kupfer(II)-oxid",
  "sodium ion": "Natrium-Ion", "chloride ion": "Chlorid-Ion", "silver ion": "Silber-Ion",
  "nitrate ion": "Nitrat-Ion", "hydrochloric acid": "Salzsäure", "sulfuric acid": "Schwefelsäure",
  "sodium hydroxide": "Natriumhydroxid", "ammonia solution": "Ammoniaklösung",
  "bleach (sodium hypochlorite)": "Bleichmittel (Natriumhypochlorit)", chloramine: "Chloramin",
  "chlorine gas": "Chlorgas", "acetic acid": "Essigsäure", "sodium acetate": "Natriumacetat",
  "acetate ion": "Acetat-Ion", "baking soda (sodium bicarbonate)": "Natron (Natriumhydrogencarbonat)",
  "washing soda (sodium carbonate)": "Waschsoda (Natriumcarbonat)", "carbon dioxide": "Kohlenstoffdioxid",
  "bicarbonate ion": "Hydrogencarbonat-Ion", "phosphoric acid": "Phosphorsäure",
  "dihydrogen phosphate ion": "Dihydrogenphosphat-Ion", "potassium chloride": "Kaliumchlorid",
  "calcium chloride": "Calciumchlorid", "chalk (calcium carbonate)": "Kreide (Calciumcarbonat)",
  "magnesium sulfate": "Magnesiumsulfat", "gypsum (calcium sulfate dihydrate)": "Gips (Calciumsulfat-Dihydrat)",
  "potassium ion": "Kalium-Ion", "calcium ion": "Calcium-Ion", "magnesium ion": "Magnesium-Ion",
  "strontium ion": "Strontium-Ion", "sulfate ion": "Sulfat-Ion", "quicklime (calcium oxide)": "Branntkalk (Calciumoxid)",
  magnesium: "Magnesium", copper: "Kupfer", zinc: "Zink", silver: "Silber", iron: "Eisen",
  "magnesium oxide": "Magnesiumoxid", "carbon (charcoal)": "Kohlenstoff (Holzkohle)", oxygen: "Sauerstoff",
  nitrogen: "Stickstoff", "copper sulfate": "Kupfersulfat", "copper(II) ion": "Kupfer(II)-Ion",
  "potassium permanganate": "Kaliumpermanganat", "iron(II) sulfate": "Eisen(II)-sulfat",
  "iron(II) ion": "Eisen(II)-Ion", "iron(III) ion": "Eisen(III)-Ion", "copper(I) ion": "Kupfer(I)-Ion",
  "manganese(II) ion": "Mangan(II)-Ion", "manganate ion": "Manganat-Ion",
  "manganese(III) ion": "Mangan(III)-Ion", "permanganate ion": "Permanganat-Ion",
  phenolphthalein: "Phenolphthalein", "methyl orange": "Methylorange", "bromothymol blue": "Bromthymolblau",
  "zinc ion": "Zink-Ion", "zinc sulfate": "Zinksulfat", hydrogen: "Wasserstoff", lead: "Blei",
  "lead(II) ion": "Blei(II)-Ion", "lead(II) nitrate": "Blei(II)-nitrat", methanol: "Methanol",
  hexane: "Hexan", propanone: "Aceton", ethyl_acetate: "Ethylacetat", "potassium hydroxide": "Kaliumhydroxid",
  "hydroxide ion": "Hydroxid-Ion", "sodium nitrate": "Natriumnitrat", "potassium nitrate": "Kaliumnitrat",
  "ascorbic acid (vitamin C)": "Ascorbinsäure (Vitamin C)", iodine: "Iod", "dehydroascorbic acid": "Dehydroascorbinsäure",
  "hydrogen iodide": "Iodwasserstoff", starch: "Stärke", amylase: "Amylase", maltose: "Maltose",
  "potassium iodide": "Kaliumiodid", "potassium iodate": "Kaliumiodat", "sodium bisulfite": "Natriumhydrogensulfit",
  "sodium bisulfate": "Natriumhydrogensulfat", "polyethylene (HDPE)": "Polyethylen (HDPE)",
  polypropylene: "Polypropylen", "polyethylene terephthalate": "Polyethylenterephthalat", polystyrene: "Polystyrol",
  "betanin (beetroot red)": "Betanin (Rote-Bete-Rot)", "curcumin (turmeric yellow)": "Curcumin (Kurkuma-Gelb)",
  "indigo carmine (E132)": "Indigocarmin (E132)", "oxidised betanin (colourless)": "oxidiertes Betanin (farblos)",
  "oxidised curcumin (colourless)": "oxidiertes Curcumin (farblos)",
  "oxidised indigo carmine (colourless)": "oxidiertes Indigocarmin (farblos)", aluminium: "Aluminium",
  liquid: "Flüssigkeit", aqueous: "wässrig", colourless: "farblos", white: "weiß", black: "schwarz",
  blue: "blau", green: "grün", yellow: "gelb", orange: "orange", red: "rot", crimson: "karminrot",
  violet: "violett", lilac: "lila", "apple-green": "apfelgrün", "brick-red": "ziegelrot",
  corrosive: "ätzend", toxic: "giftig", flammable: "entzündlich", oxidizer: "oxidierend",
  "start here": "hier anfangen", "acids & bases": "Säuren & Basen", "heat & fire": "Wärme & Feuer",
  "redox & electricity": "Redox & Elektrizität", "water chemistry": "Wasserchemie",
  "gases & pressure": "Gase & Druck", rates: "Reaktionsgeschwindigkeit", separations: "Trennverfahren", safety: "Sicherheit",
  buffer: "Puffer", calorimetry: "Kalorimetrie", conductivity: "Leitfähigkeit", "counting in fives": "Zählen in Fünferschritten",
  electrode: "Elektrode", electrolysis: "Elektrolyse", fire: "Feuer", "first warmth": "Erste Wärme", fizz: "Sprudeln",
  grit: "Streusalz", "hard water": "Hartes Wasser", limewater: "Kalkwasser", "neutral moves": "Neutral verschiebt sich",
  "never mix": "Niemals mischen", "one thing at a time": "Eins nach dem anderen", "salt from brine": "Salz aus Sole",
  "sealed gas": "Eingeschlossenes Gas", "silver and salt": "Silber und Salz", spannungsreihe: "Spannungsreihe",
  "spirit still": "Spiritusbrennerei", "there and back": "Hin und zurück", "three protons": "Drei Protonen",
  "titration manual": "Titration von Hand", titration: "Titration", "transport column": "Transportsäule", "two roads": "Zwei Wege",
  "Buffers: why your blood does not change pH when you drink lemonade": "Puffer: Warum sich der pH-Wert deines Blutes beim Trinken von Limonade nicht ändert",
  "Neutralisation enthalpy: how much heat does an acid-base reaction produce?": "Neutralisationsenthalpie: Wie viel Wärme erzeugt eine Säure-Base-Reaktion?",
  "Ionic conductivity: strong vs weak electrolytes": "Ionenleitfähigkeit: starke und schwache Elektrolyte",
  "Permanganate counts in fives: a redox titration, stopped at the endpoint": "Permanganat zählt in Fünferschritten: eine Redoxtitration bis zum Endpunkt",
  "The electrochemical series: zinc displaces copper": "Die elektrochemische Spannungsreihe: Zink verdrängt Kupfer",
  "Electrolysis of copper sulfate: Faraday's law": "Elektrolyse von Kupfersulfat: Faradays Gesetz",
  "Fire: what burns, what does not, and what it leaves behind": "Feuer: Was brennt, was nicht und was zurückbleibt",
  "First warmth: mix hot and cold water, watch the middle": "Erste Wärme: Heißes und kaltes Wasser mischen und die Mitte beobachten",
  "The fizz: vinegar meets baking soda": "Das Sprudeln: Essig trifft Natron",
  "Grit for the road: salt holds water liquid below zero": "Streusalz: Salz hält Wasser unter null Grad flüssig",
  "The kettle and the statue: hard water and limescale": "Der Wasserkocher und die Statue: hartes Wasser und Kalk",
  "Carbon dioxide enters as well as leaves an open vessel.": "Kohlenstoffdioxid tritt in ein offenes Gefäß ein und verlässt es auch wieder.",
  "Neutral is a moving target: warm water is still just water": "Neutral ist beweglich: Warmes Wasser bleibt Wasser",
  "The safety layer: precise about danger, honest about outcomes": "Die Sicherheitsebene: präzise über Gefahren, ehrlich über Folgen",
  "One thing at a time: the column tells a mixture apart.": "Eins nach dem anderen: Die Säule trennt ein Gemisch.",
  "Rates: two beakers, one variable, one clock.": "Reaktionsgeschwindigkeit: zwei Bechergläser, eine Variable, eine Uhr.",
  "Salt from brine: the oldest chemistry in the world": "Salz aus Sole: die älteste Chemie der Welt",
  "Gas laws in a sealed vessel": "Gasgesetze in einem geschlossenen Gefäß",
  "The marquee: silver nitrate + table salt": "Das Vorzeigeexperiment: Silbernitrat + Kochsalz",
  "Die Spannungsreihe: who takes the electrons, and what a battery is worth": "Die Spannungsreihe: Wer nimmt die Elektronen und was leistet eine Batterie?",
  "The spirit still — and the wall at 96 percent": "Die Spiritusbrennerei — und die Grenze bei 96 Prozent",
  "There and back again: an ester is made, then unmade.": "Hin und zurück: Ein Ester entsteht und wird wieder zerlegt.",
  "Three protons: phosphoric acid meets the burette": "Drei Protonen: Phosphorsäure trifft die Bürette",
  "Titration: strong acid meets strong base": "Titration: starke Säure trifft starke Base",
  "A salt pulse through a water column": "Ein Salzimpuls durch eine Wassersäule",
  "Two roads, one temperature: Hess's law with a thermometer": "Zwei Wege, eine Temperatur: der Satz von Hess mit einem Thermometer",
  Hydrogen: "Wasserstoff", Carbon: "Kohlenstoff", Nitrogen: "Stickstoff", Oxygen: "Sauerstoff",
  Fluorine: "Fluor", Sodium: "Natrium", Silicon: "Silicium", Phosphorus: "Phosphor",
  Sulfur: "Schwefel", Chlorine: "Chlor", Potassium: "Kalium", Calcium: "Calcium",
  Manganese: "Mangan", Iron: "Eisen", Cobalt: "Kobalt", Copper: "Kupfer", Zinc: "Zink",
  Arsenic: "Arsen", Bromine: "Brom", Silver: "Silber", Tin: "Zinn", Antimony: "Antimon",
  Iodine: "Iod", Caesium: "Cäsium", Tungsten: "Wolfram", Gold: "Gold", Mercury: "Quecksilber",
  Lead: "Blei", Bismuth: "Bismut", Astatine: "Astat", Francium: "Francium",
  Protactinium: "Protactinium", Uranium: "Uran", Plutonium: "Plutonium",
  Californium: "Californium", Roentgenium: "Röntgenium"
};

const DE: Record<string, string> = {
  ...DE_TERMS,
  "English": "Englisch",
  "German": "Deutsch",
  "Language": "Sprache",
  "Kerotakis — the bench": "Kerotakis — das Labor",
  "A virtual chemistry laboratory that computes real chemistry — drag reagents onto drawn glassware and watch a real aqueous solver answer. Offline once loaded.": "Ein virtuelles Chemielabor, das echte Chemie berechnet — ziehe Reagenzien auf die Glasgeräte und beobachte die Ergebnisse eines realen wässrigen Lösers. Nach dem Laden offline nutzbar.",
  "a chemistry bench that computes": "ein Chemielabor, das rechnet",
  "Sandbox": "Sandbox",
  "Mission Control": "Missionszentrale",
  "research campus": "Forschungscampus",
  "Choose where to investigate": "Wähle deinen Forschungsort",
  "Finish missions to open new districts. Choose your own route through the campus.": "Schließe Missionen ab, um neue Bereiche zu öffnen. Wähle deinen eigenen Weg durch den Campus.",
  "{count} missions complete": "{count} Missionen abgeschlossen",
  "discoveries": "Entdeckungen",
  "close research map": "Forschungskarte schließen",
  "campus districts": "Campusbereiche",
  "{done} of {total} complete": "{done} von {total} abgeschlossen",
  "complete {count} missions to enter": "{count} Missionen abschließen zum Betreten",
  "complete one mission to enter": "eine Mission abschließen zum Betreten",
  "district complete": "Bereich abgeschlossen",
  "district open": "Bereich geöffnet",
  "district locked": "Bereich gesperrt",
  "mission complete": "Mission abgeschlossen",
  "replay mission": "Mission wiederholen",
  "The route is still being surveyed": "Diese Route wird noch erkundet",
  "Complete {count} missions anywhere in the open districts to unlock this route.": "Schließe {count} Missionen in beliebigen geöffneten Bereichen ab, um diese Route freizuschalten.",
  "Complete one mission anywhere in the open districts to unlock this route.": "Schließe eine Mission in einem beliebigen geöffneten Bereich ab, um diese Route freizuschalten.",
  "Other ways to explore": "Andere Wege zum Entdecken",
  "experiment library": "Experimentbibliothek",
  "open full sandbox": "vollständige Sandbox öffnen",
  "Discovery Hall": "Entdeckerhalle",
  "Start with visible changes, careful observation, and safe habits.": "Beginne mit sichtbaren Veränderungen, genauer Beobachtung und sicheren Gewohnheiten.",
  "Matter Gardens": "Gärten der Stoffe",
  "Follow acids, bases, minerals, and water through the living campus.": "Verfolge Säuren, Basen, Mineralien und Wasser durch den lebendigen Campus.",
  "Energy Yard": "Energiehof",
  "Work with heat, fire, pressure, and the energy hidden in matter.": "Arbeite mit Wärme, Feuer, Druck und der in Stoffen verborgenen Energie.",
  "Electron Works": "Elektronenwerk",
  "Make electrons move, plate metals, and build chemical power.": "Bringe Elektronen in Bewegung, beschichte Metalle und erzeuge chemische Energie.",
  "Systems Dock": "Systemdock",
  "Control rates and separate mixtures with connected apparatus.": "Steuere Reaktionsgeschwindigkeiten und trenne Gemische mit verbundenen Geräten.",
  "open Mission Control": "Missionszentrale öffnen",
  "close Mission Control": "Missionszentrale schließen",
  "Choose your path": "Wähle deinen Weg",
  "Follow a guided investigation or open the whole laboratory.": "Folge einer geführten Untersuchung oder öffne das ganze Labor.",
  "Story missions": "Forschungsmissionen",
  "Experiment library": "Experimentbibliothek",
  "{count} Codex experiments": "{count} Codex-Experimente",
  "Predict first, run real chemistry, then compare the evidence.": "Erst vorhersagen, dann echte Chemie ausführen und die Nachweise vergleichen.",
  "browse experiments": "Experimente durchsuchen",
  "Sandbox lab": "Sandbox-Labor",
  "Your laboratory, your rules": "Dein Labor, deine Regeln",
  "Everything is unlocked. Build, test, and break your own ideas.": "Alles ist freigeschaltet. Baue, teste und hinterfrage deine eigenen Ideen.",
  "exit to sandbox": "zur Sandbox wechseln",
  "you are here": "du bist hier",
  "mission in progress": "Mission läuft",
  "mission journal": "Missionsjournal",
  "leave mission": "Mission verlassen",
  "current lab instruction": "aktuelle Laboranweisung",
  "show a hint": "Hinweis anzeigen",
  "hide hint": "Hinweis ausblenden",
  "hint": "Hinweis",
  "evidence ledger": "Nachweisprotokoll",
  "Results gathered during this mission": "Während dieser Mission gesammelte Ergebnisse",
  "Your engine-backed observations and measurements will collect here.": "Deine vom Rechenmodell bestätigten Beobachtungen und Messungen werden hier gesammelt.",
  "Prepare the requested material": "Bereite den benötigten Stoff vor",
  "Set up another vessel": "Stelle ein weiteres Gefäß bereit",
  "Change how closely you observe": "Ändere die Beobachtungstiefe",
  "Take the next measurement": "Führe die nächste Messung durch",
  "Observe the evidence": "Beobachte die Nachweise",
  "Run the reaction": "Führe die Reaktion durch",
  "Add carefully until the endpoint": "Gib vorsichtig bis zum Endpunkt hinzu",
  "Control the vessel": "Kontrolliere das Gefäß",
  "Separate the mixture": "Trenne das Gemisch",
  "Change the conditions": "Verändere die Bedingungen",
  "Build and test the electrical system": "Baue und prüfe das elektrische System",
  "Carry out the next investigation step": "Führe den nächsten Untersuchungsschritt durch",
  "Check the selected vessel and material before adding it.": "Prüfe vor der Zugabe das ausgewählte Gefäß und den Stoff.",
  "Empty vessels appear in the Prepare zone.": "Leere Gefäße erscheinen im Bereich Vorbereiten.",
  "The register changes detail, never the underlying chemistry.": "Das Register ändert die Detailtiefe, niemals die zugrunde liegende Chemie.",
  "Select the named vessel, then use its observation or measurement tools.": "Wähle das genannte Gefäß und nutze dann seine Beobachtungs- oder Messgeräte.",
  "Use the burette for controlled additions and watch the instrument reading.": "Nutze die Bürette für kontrollierte Zugaben und beobachte den Messwert.",
  "Connected apparatus can be placed from the equipment cabinet.": "Verbundene Aufbauten kannst du aus dem Geräteschrank platzieren.",
  "Condition controls are in the selected vessel's action dock.": "Die Bedingungsregler befinden sich im Aktionsfeld des ausgewählten Gefäßes.",
  "Place the electrical apparatus, then connect the named vessels.": "Platziere die elektrischen Geräte und verbinde dann die genannten Gefäße.",
  "The exact operator instruction remains visible if you want to run it directly.": "Die genaue Operatoranweisung bleibt sichtbar, falls du sie direkt ausführen möchtest.",
  "mission debrief": "Missionsauswertung",
  "discovery recorded": "Entdeckung gespeichert",
  "mission replay complete": "Missionswiederholung abgeschlossen",
  "close mission debrief": "Missionsauswertung schließen",
  "evidence items": "Nachweise",
  "missions complete": "Missionen abgeschlossen",
  "review the evidence": "Nachweise ansehen",
  "keep experimenting": "weiter experimentieren",
  "return to research map": "zur Forschungskarte",
  "This mission was already complete. Your new run remains in the lab notebook.": "Diese Mission war bereits abgeschlossen. Dein neuer Durchlauf bleibt im Laborbuch erhalten.",
  "Matter Gardens and Energy Yard are now open.": "Die Gärten der Stoffe und der Energiehof sind jetzt geöffnet.",
  "Electron Works is now open.": "Das Elektronenwerk ist jetzt geöffnet.",
  "Systems Dock is now open.": "Das Systemdock ist jetzt geöffnet.",
  "Your discovery is now part of the Story research record.": "Deine Entdeckung ist jetzt Teil des Forschungsprotokolls der Story.",
  "mission progress": "Missionsfortschritt",
  "current objective": "aktuelles Ziel",
  "{step} of {total} steps": "Schritt {step} von {total}",
  "continue mission": "Mission fortsetzen",
  "Available missions": "Verfügbare Missionen",
  "launch mission": "Mission starten",
  "Missions are downloading. The sandbox is ready now.": "Missionen werden geladen. Die Sandbox ist schon bereit.",
  "guided mission": "geführte Mission",
  "lab mode": "Labormodus",
  "utilities": "Werkzeuge & Dateien",
  "open utilities": "Werkzeuge und Dateien öffnen",
  "time and history": "Zeit und Verlauf",
  "files and notebook": "Dateien und Laborbuch",
  "explore and study": "Entdecken und untersuchen",
  "supply cabinet": "Materialschrank",
  "choose what goes on the bench": "Wähle, was auf den Labortisch kommt",
  "reagents": "Reagenzien",
  "equipment": "Geräte",
  "cabinet": "Schrank",
  "journal": "Laborbuch",
  "workspace": "Labortisch",
  "lab journal": "Laborbuch",
  "observations and evidence": "Beobachtungen und Nachweise",
  "notebook entries": "Laborbucheinträge",
  "working with vessel v{vessel}": "Arbeiten mit Gefäß v{vessel}",
  "precision tools": "Präzisionsgeräte",
  "controlled addition": "kontrollierte Zugabe",
  "choose more equipment…": "weitere Geräte wählen…",
  "transfer and separation": "Überführen und Trennen",
  "appearance": "Darstellung",
  "light": "Hell",
  "dark": "Dunkel",
  "high contrast": "Hoher Kontrast",
  "quick actions for vessel v{vessel}": "Schnellaktionen für Gefäß v{vessel}",
  "selected": "ausgewählt",
  "heat": "erwärmen",
  "cool": "abkühlen",
  "look": "ansehen",
  "seal": "verschließen",
  "run {action} on {vessel}": "{action} bei {vessel} ausführen",
  "details": "Details",
  "more tools": "weitere Geräte",
  "bench work zones": "Arbeitsbereiche des Labortischs",
  "prepare": "Vorbereiten",
  "react": "Reagieren",
  "analyse": "Analysieren",
  "add here": "hier hinzugeben",
  "pour from {vessel}": "aus {vessel} gießen",
  "transfer target": "Zielgefäß",
  "tools": "Werkzeuge",
  "undo": "rückgängig",
  "save .lab": ".lab speichern",
  "save notes": "Notizen speichern",
  "print": "drucken",
  "open .lab": ".lab öffnen",
  "clear": "leeren",
  "wait 30 s": "30 s warten",
  "burette": "Bürette",
  "filter": "filtrieren",
  "decant": "dekantieren",
  "drain": "ablassen",
  "voltmeter": "Voltmeter",
  "still": "Destille",
  "more apparatus": "weitere Geräte",
  "apparatus…": "Geräte…",
  "curated reaction": "kuratierte Reaktion",
  "column train": "Säulenkette",
  "lessons…": "Lektionen…",
  "start a lesson": "eine Lektion beginnen",
  "more": "weitere",
  "live": "aktiv",
  "shipped results": "mitgelieferte Ergebnisse",
  "starting…": "startet…",
  "elements": "Elemente",
  "toolbox": "Werkzeugkasten",
  "experiments": "Experimente",
  "map": "Karte",
  "install": "installieren",
  "console": "Konsole",
  "A newer bench is downloaded and ready.": "Eine neuere Version des Labors wurde geladen und ist bereit.",
  "reload into it": "neu laden",
  "later": "später",
  "cancel": "abbrechen",
  "pour": "gieße",
  "tap the source vessel": "Quellgefäß antippen",
  "from v{vessel} — now tap the target": "von v{vessel} — jetzt das Ziel antippen",
  "latest reaction equation": "neueste Reaktionsgleichung",
  "panes": "Bereiche",
  "bench": "Labor",
  "shelf": "Regal",
  "notes": "Notizen",
  "print the notebook — or save it as PDF from the print dialog": "Laborbuch drucken — oder im Druckdialog als PDF speichern",
  "let 30 seconds of bench time pass": "30 Sekunden Laborzeit verstreichen lassen",
  "clamp the burette over the selected vessel": "Bürette über dem gewählten Gefäß einspannen",
  "the periodic table, wired to the shelf": "das Periodensystem, mit dem Regal verbunden",
  "named relations: compute with provenance": "benannte Beziehungen: mit Herkunftsnachweis berechnen",
  "codex experiments: predict, run, check": "Kodex-Experimente: vorhersagen, ausführen, prüfen",
  "the concept map: what you have met, what is ready": "Begriffskarte: Bekanntes und Bereites",
  "install the bench — it runs offline, engine and all": "Labor installieren — vollständig offline nutzbar",
  "engine {identity}": "Engine {identity}",
  "{tool}: pick the source vessel, then the target": "{tool}: erst das Quellgefäß, dann das Ziel wählen",

  "choose…": "auswählen…",
  "go": "los",
  "put away": "wegräumen",
  "{apparatus} over v{vessel}": "{apparatus} über v{vessel}",
  "the bench": "das Labor",
  "add a vessel": "ein Gefäß hinzufügen",
  "Drag something in from the shelf, type a command below — or pick a lesson.": "Ziehe etwas aus dem Regal hinein, gib unten einen Befehl ein — oder wähle eine Lektion.",
  "The bench is warming up…": "Das Labor wärmt sich auf…",
  "beaker": "Becherglas",
  "flask": "Kolben",
  "tube": "Reagenzglas",
  "cylinder": "Messzylinder",
  "crucible": "Tiegel",
  "burette over v{vessel}": "Bürette über v{vessel}",
  "titrant": "Titrationsmittel",
  "concentration": "Konzentration",
  "per drop": "je Tropfen",
  "until pH": "bis pH",
  "dripping…": "tropft…",
  "start the drip": "Tropfen starten",
  "save SVG": "SVG speichern",
  "save PNG": "PNG speichern",
  "data": "Daten",
  "not a command": "kein Befehl",
  "command": "Befehl",
  "speak a command — it lands here to read and correct before you run it": "Befehl sprechen — er erscheint hier zum Prüfen und Korrigieren",
  "stop listening": "Zuhören beenden",
  "speak a command": "Befehl sprechen",
  "concept map": "Begriffskarte",
  "{met} of {total} concepts met — filled means run to a green check here": "{met} von {total} Begriffen kennengelernt — gefüllt bedeutet hier erfolgreich ausgeführt",
  "close": "schließen",
  "the codex export has not arrived yet — the map draws itself from it": "Der Kodex-Export ist noch nicht eingetroffen — daraus erstellt sich die Karte.",
  "concept graph": "Begriffsdiagramm",
  "ready": "bereit",
  "locked": "gesperrt",
  "needs: {concepts}": "benötigt: {concepts}",
  "{count} from the codex — each one computed, checked, and yours to break": "{count} aus dem Kodex — jedes berechnet, geprüft und bereit für deine Tests",
  "all": "alle",
  "by concept": "nach Begriff",
  "by curriculum": "nach Lehrplan",
  "filter…": "filtern…",
  "filter experiments": "Experimente filtern",
  "concepts": "Begriffe",
  "these entries name no concepts yet": "Diese Einträge nennen noch keine Begriffe.",
  "taught alongside:": "zusammen vermittelt:",
  "no curriculum placements in this export yet": "Noch keine Lehrplan-Zuordnungen in diesem Export.",
  "placed per: {sources}": "zugeordnet nach: {sources}",
  "nothing matches that filter": "Nichts entspricht diesem Filter.",
  "theory": "Theorie",
  "procedure": "Ablauf",
  "predict & run": "vorhersagen & ausführen",
  "concepts: {concepts}": "Begriffe: {concepts}",
  "models: {models}": "Modelle: {models}",
  "you will need: {apparatus}": "Du brauchst: {apparatus}",
  "commit a prediction first — the reveal only teaches if you have.": "Lege dich zuerst auf eine Vorhersage fest — erst dann ist die Auflösung lehrreich.",
  "running…": "läuft…",
  "run it on the bench": "im Labor ausführen",
  "the chemistry agrees": "die Chemie stimmt überein",
  "not everything checked out": "nicht alles wurde bestätigt",
  "occurred": "aufgetreten",
  "absent": "nicht aufgetreten",
  "expected {range}": "erwartet {range}",
  "your prediction held.": "Deine Vorhersage traf zu.",
  "lab notebook": "Laborbuch",
  "Keyboard": "Tastatur",
  "keyboard shortcuts": "Tastaturkürzel",
  "focus the command bar": "Befehlszeile fokussieren",
  "undo the last step": "letzten Schritt rückgängig machen",
  "redo the step": "Schritt wiederholen",
  "open this help": "diese Hilfe öffnen",
  "Every button and drag also works from the keyboard — vessels are buttons, and everything you do is a command you can read back in the notebook.": "Jede Schaltfläche und jede Ziehbewegung funktioniert auch per Tastatur — Gefäße sind Schaltflächen, und jede Aktion ist ein Befehl, den du im Laborbuch nachlesen kannst.",
  "vessel v{vessel} detail": "Details zu Gefäß v{vessel}",
  "particles": "Teilchen",
  "close inspector": "Inspektor schließen",
  "act on {vessel}": "Aktionen für {vessel}",
  "gas tests on {vessel}": "Gastests für {vessel}",
  "test the gas:": "Gas testen:",
  "heat 10 kJ": "um 10 kJ erhitzen",
  "cool 10 kJ": "um 10 kJ kühlen",
  "stir": "rühren",
  "ignite": "entzünden",
  "seal 500 mL": "mit 500 mL Kopfraum verschließen",
  "open": "öffnen",
  "thermometer": "Thermometer",
  "pH meter": "pH-Meter",
  "balance": "Waage",
  "volume": "Volumen",
  "conductivity": "Leitfähigkeit",
  "pressure gauge": "Manometer",
  "calorimeter": "Kalorimeter",
  "look closely": "genau ansehen",
  "chromatograph": "Chromatograph",
  "instruments for {vessel}": "Instrumente für {vessel}",
  "kit reagents": "Reagenzien des Sets",
  "amount of {name}": "Menge von {name}",
  "lesson {name}": "Lektion {name}",
  "do it": "ausführen",
  "off the script by {count} step": "{count} Schritt vom Ablauf abgewichen",
  "off the script by {count} steps": "{count} Schritte vom Ablauf abgewichen",
  "exploring is allowed": "Erkunden ist erlaubt",
  "return to the script": "zum Ablauf zurückkehren",
  "leave lesson": "Lektion verlassen",
  "positive ion": "positives Ion",
  "negative ion": "negatives Ion",
  "uncharged, dissolved": "ungeladen, gelöst",
  "solvent": "Lösungsmittel",
  "solid": "Feststoff",
  "gas": "Gas",
  "also present, too dilute to draw at this scale:": "ebenfalls vorhanden, in diesem Maßstab zu verdünnt zum Darstellen:",
  "ratios from solved speciation": "Verhältnisse aus berechneter Speziation",
  "ratios from the ideal fallback": "Verhältnisse aus idealer Näherung",
  "periodic table": "Periodensystem",
  "the elements": "die Elemente",
  "tap one to see what the lab has of it": "Element antippen, um den Laborbestand zu sehen",
  "period {period} · group {group} · {block}-block": "Periode {period} · Gruppe {group} · {block}-Block",
  "flame test: {flames}": "Flammenprobe: {flames}",
  "on the shelf, containing {symbol}:": "im Regal, enthält {symbol}:",
  "nothing on the shelf contains {symbol} yet — the registry grows by provenance-carrying tranches, not by wishful entries.": "Noch enthält nichts im Regal {symbol} — das Register wächst in Chargen mit Herkunftsnachweis, nicht durch Wunschdaten.",
  "curated reaction on v{vessel}": "kuratierte Reaktion in v{vessel}",
  "verified family templates the engine can run": "verifizierte Reaktionsfamilien, die die Engine ausführen kann",
  "run": "ausführen",
  "temperature": "Temperatur",
  "ionic strength": "Ionenstärke",
  "close reading": "Messwert schließen",
  "detail level": "Detailstufe",
  "Look": "Ansehen",
  "Measure": "Messen",
  "Model": "Modellieren",
  "reagent shelf": "Reagenzienregal",
  "shelf contents": "Regalinhalt",
  "the kit ({count})": "das Set ({count})",
  "everything": "alles",
  "find a substance…": "Stoff suchen…",
  "find a substance": "Stoff suchen",
  "phase filter": "Phasenfilter",
  "custom amount": "eigene Menge",
  "nothing on the shelf matches": "Nichts im Regal passt.",
  "{count} substances — every one computed, none painted on": "{count} Stoffe — alle berechnet, keiner bloß dargestellt",
  "{shown} of {total} substances": "{shown} von {total} Stoffen",
  "burns {colour}": "Flammenfarbe {colour}",
  "hazards: {hazards}": "Gefahren: {hazards}",
  "hazards unassessed": "Gefahren nicht bewertet",
  "timeline: step {position} of {total}": "Zeitleiste: Schritt {position} von {total}",
  "relation calculator": "Beziehungsrechner",
  "Toolbox": "Werkzeugkasten",
  "named relations, computed by the engine — with sources": "benannte Beziehungen, von der Engine berechnet — mit Quellen",
  "close the toolbox": "Werkzeugkasten schließen",
  "relations": "Beziehungen",
  "the engine has not answered with its relations yet": "Die Engine hat ihre Beziehungen noch nicht geliefert.",
  "arguments": "Argumente",
  "optional": "optional",
  "computing…": "berechnet…",
  "compute": "berechnen",
  "cells in flow order, then where solution enters and collects": "Zellen in Fließrichtung, dann Einlass und Sammelgefäß wählen",
  "cells": "Zellen",
  "inlet": "Einlass",
  "receiver": "Sammelgefäß",
  "steps": "Schritte",
  "run the column": "Säule ausführen",
  "solution": "Lösung",
  "sealed": "verschlossen",
  "pressure-controlled": "druckgeregelt",
  "swept with carrier gas": "mit Trägergas gespült",
  "pH probe": "pH-Sonde",
  "computed": "berechnet",
  "open boundary": "offen",
  "sealed boundary": "verschlossen",
  "pressure controlled boundary": "druckgeregelt",
  "swept boundary": "gespült"
  ,"walk command history": "Befehlsverlauf durchgehen"
  ,"undo (replays the bench)": "rückgängig (Labor wird erneut ausgeführt)"
  ,"redo": "wiederholen"
  ,"this help": "diese Hilfe"
  ,"close panels": "Fenster schließen"
  ,"the particle view could not be drawn: {error}": "Die Teilchenansicht konnte nicht gezeichnet werden: {error}"
  ,"ratios from the inventory — ion pairs and complexes not resolved": "Verhältnisse aus dem Bestand — Ionenpaare und Komplexe nicht aufgelöst"
  ,"pop": "Knallgasprobe"
  ,"splint": "Spanprobe"
  ,"limewater": "Kalkwasserprobe"
  ,"litmus": "Lackmusprobe"
  ,"alkali metal": "Alkalimetall"
  ,"alkaline-earth metal": "Erdalkalimetall"
  ,"transition metal": "Übergangsmetall"
  ,"post-transition metal": "Metall der Borgruppe"
  ,"metalloid": "Halbmetall"
  ,"nonmetal": "Nichtmetall"
  ,"halogen": "Halogen"
  ,"noble gas": "Edelgas"
  ,"lanthanide": "Lanthanoid"
  ,"actinide": "Actinoid"
  ,"properties not yet established": "Eigenschaften noch nicht bestimmt"
  ,"wash bottle": "Spritzflasche"
  ,"add water up to a volume": "mit Wasser auf ein Volumen auffüllen"
  ,"to volume": "auf Volumen"
  ,"evaporating dish": "Abdampfschale"
  ,"boil part of the liquid away": "einen Teil der Flüssigkeit verdampfen"
  ,"fraction": "Anteil"
  ,"electrodes and supply": "Elektroden und Stromquelle"
  ,"pass a current for a time": "für eine bestimmte Zeit Strom leiten"
  ,"current": "Stromstärke"
  ,"for": "für"
  ,"mortar": "Mörser"
  ,"set a solid's particle size": "Korngröße eines Feststoffs einstellen"
  ,"grain": "Korngröße"
  ,"lamp": "Lampe"
  ,"shine light of one wavelength": "Licht einer Wellenlänge einstrahlen"
  ,"wavelength": "Wellenlänge"
  ,"irradiance": "Bestrahlungsstärke"
  ,"piston lid": "Kolbendeckel"
  ,"hold a set pressure over the vessel": "einen festen Druck über dem Gefäß halten"
  ,"pressure": "Druck"
  ,"headspace": "Kopfraum"
  ,"carrier-gas line": "Trägergasleitung"
  ,"purge the headspace with inert gas": "Kopfraum mit Inertgas spülen"
  ,"The bench is live: states nobody pre-computed are solved.": "Das Labor ist aktiv: Nicht vorberechnete Zustände werden gelöst."
  ,"The bench answers from shipped results only — the live aqueous engine is not attached.": "Das Labor antwortet nur mit mitgelieferten Ergebnissen — die aktive wässrige Engine ist nicht verbunden."
  ,"the aqueous engine failed to attach: {reason}": "Die wässrige Engine konnte nicht verbunden werden: {reason}"
  ,"replayed": "erneut ausgeführt"
  ,"restored instantly": "sofort wiederhergestellt"
  ,"restored your last session: {count} step(s) {how}": "Letzte Sitzung wiederhergestellt: {count} Schritt(e), {how}"
  ,"could not restore the last session — starting fresh": "Die letzte Sitzung konnte nicht wiederhergestellt werden — neuer Start."
  ,"the bench is empty again": "Das Labor ist wieder leer."
  ,"the bench refused this operation": "Das Labor hat diesen Vorgang abgelehnt."
  ,"running {name} on this bench": "{name} wird in diesem Labor ausgeführt."
  ,"stopped at {name}:{line} — the rest of the file did not run": "Bei {name}:{line} angehalten — der Rest der Datei wurde nicht ausgeführt."
  ,"{name} finished": "{name} abgeschlossen"
  ,"speaking at {level}": "Detailstufe {level}"
  ,"stepped back to {position} of {total}": "zurück zu Schritt {position} von {total}"
  ,"stepped forward to {position} of {total}": "vor zu Schritt {position} von {total}"
  ,"replay failed, the bench may be out of sync — {reason}": "Wiederholung fehlgeschlagen; das Labor ist möglicherweise nicht synchron — {reason}"
  ,"lesson started: {name}": "Lektion begonnen: {name}"
  ,"lesson finished: {name}": "Lektion abgeschlossen: {name}"
  ,"back on the script.": "Zurück im Ablauf."
  ,"lesson left: {name}": "Lektion verlassen: {name}"
  ,"Kerotakis lab notebook": "Kerotakis-Laborbuch"
  ,"hazard": "Gefahr"
  ,"the bench answered {answer}.": "Das Labor antwortete {answer}."
  ,"Try: {next}": "Versuche: {next}"
  ,"separate solids from liquid": "Feststoffe von Flüssigkeit trennen"
  ,"pour off a chosen fraction": "einen gewählten Anteil abgießen"
  ,"move the lower liquid layer": "die untere Flüssigkeitsschicht ablassen"
  ,"connect two half-cells": "zwei Halbzellen verbinden"
  ,"separate by volatility": "nach Flüchtigkeit trennen"
  ,"active work area": "aktiver Arbeitsbereich"
  ,"Instrument wall": "Instrumentenwand"
  ,"Choose a tool and it appears at the selected work area.": "Wähle ein Werkzeug; es erscheint am ausgewählten Arbeitsplatz."
  ,"Every installed instrument is available in Sandbox.": "Jedes installierte Instrument ist in der Sandbox verfügbar."
  ,"Complete investigations to earn permanent access to more instruments.": "Schließe Untersuchungen ab, um dauerhaft Zugang zu weiteren Instrumenten zu erhalten."
  ,"after one mission": "nach einer Mission"
  ,"after {count} missions": "nach {count} Missionen"
  ,"mission kit": "Missionsset"
  ,"Permanent stock unlocks after one completed mission. Mission kits loan required materials.": "Der dauerhafte Vorrat wird nach einer abgeschlossenen Mission freigeschaltet. Missionssets leihen benötigte Stoffe aus."
  ,"Permanent stock unlocks after {count} completed missions. Mission kits loan required materials.": "Der dauerhafte Vorrat wird nach {count} abgeschlossenen Missionen freigeschaltet. Missionssets leihen benötigte Stoffe aus."
  ,"new permanent equipment": "neues dauerhaftes Gerät"
  ,"place on bench": "auf den Labortisch stellen"
  ,"Concentrate solutions and recover dissolved solids.": "Konzentriere Lösungen und gewinne gelöste Feststoffe zurück."
  ,"Control pressure and headspace above a vessel.": "Kontrolliere Druck und Gasraum über einem Gefäß."
  ,"Drive and measure electrochemical change.": "Treibe elektrochemische Veränderungen an und miss sie."
  ,"Separate liquids through a connected distillation rig.": "Trenne Flüssigkeiten mit einer verbundenen Destillationsapparatur."
  ,"{count} uses left": "{count} Entnahmen übrig"
  ,"one use left": "eine Entnahme übrig"
  ,"This bottle is empty. Mission kits still supply required materials, and permanent stock refills after a new discovery.": "Diese Flasche ist leer. Missionssets enthalten weiterhin benötigte Stoffe; nach einer neuen Entdeckung wird der dauerhafte Vorrat aufgefüllt."
  ,"That material is not yet available. Accept an investigation that supplies it or complete more missions.": "Dieser Stoff ist noch nicht verfügbar. Nimm eine Untersuchung an, die ihn bereitstellt, oder schließe weitere Missionen ab."
  ,"That bottle is empty. Mission kits still supply required materials, and the stockroom refills after a new discovery.": "Diese Flasche ist leer. Missionssets enthalten weiterhin benötigte Stoffe; nach einer neuen Entdeckung wird das Lager aufgefüllt."
  ,"stockroom replenished": "Materiallager aufgefüllt"
  ,"Permanent supplies are ready for the next investigation.": "Die dauerhaften Vorräte stehen für die nächste Untersuchung bereit."
  ,"measure and transform": "messen und verändern"
  ,"on bench": "auf dem Tisch"
  ,"select source": "Quelle wählen"
  ,"move solution through connected cells": "Lösung durch verbundene Zellen bewegen"
  ,"reaction studio": "Reaktionsstudio"
  ,"choose a verified reaction family": "eine geprüfte Reaktionsfamilie wählen"
  ,"{tool} deployed": "{tool} aufgebaut"
  ,"deployed at vessel v{vessel}": "bei Gefäß v{vessel} aufgebaut"
  ,"{zone} work zone": "Arbeitsbereich {zone}"
  ,"set up and measure": "aufbauen und abmessen"
  ,"mix and transform": "mischen und umsetzen"
  ,"measure and compare": "messen und vergleichen"
  ,"move vessel v{vessel}": "Gefäß v{vessel} verschieben"
  ,"move vessel v{vessel} to {zone}": "Gefäß v{vessel} nach {zone} verschieben"
  ,"vessel v{vessel} moved to {zone}": "Gefäß v{vessel} nach {zone} verschoben"
  ,"place vessel here": "Gefäß hier abstellen"
  ,"move": "verschieben"
  ,"vessel v{vessel} placement": "Platzierung von Gefäß v{vessel}"
  ,"My Chemistry Lab": "Mein Chemielabor"
  ,"open world map": "Weltkarte öffnen"
  ,"close world map": "Weltkarte schließen"
  ,"your laboratory": "dein Labor"
  ,"laboratory name": "Laborname"
  ,"rename laboratory": "Labor umbenennen"
  ,"save": "speichern"
  ,"current save": "aktueller Spielstand"
  ,"Story": "Story"
  ,"Story laboratory": "Story-Labor"
  ,"Sandbox hangar": "Sandbox-Hangar"
  ,"Kerotakis Research Campus": "Kerotakis-Forschungscampus"
  ,"Where do you want to work today?": "Wo möchtest du heute arbeiten?"
  ,"Explore guided investigations in Story, or enter a fully unlocked laboratory in Sandbox.": "Erkunde geführte Untersuchungen in der Story oder betrete ein vollständig freigeschaltetes Labor in der Sandbox."
  ,"active now": "jetzt aktiv"
  ,"separate save": "eigener Spielstand"
  ,"The Discovery Wing": "Der Entdeckertrakt"
  ,"Take missions, earn permanent instruments, and follow the chemistry story at your pace.": "Nimm Missionen an, verdiene dauerhafte Geräte und folge der Chemiegeschichte in deinem Tempo."
  ,"{count} missions": "{count} Missionen"
  ,"missions arriving…": "Missionen treffen ein…"
  ,"guided progress": "geführter Fortschritt"
  ,"enter Story": "Story betreten"
  ,"switch to Story": "zur Story wechseln"
  ,"Mission Board": "Missionstafel"
  ,"choose an investigation": "eine Untersuchung wählen"
  ,"Research Library": "Forschungsbibliothek"
  ,"{count} computed experiments": "{count} berechnete Experimente"
  ,"archive syncing…": "Archiv wird synchronisiert…"
  ,"The Open Bench": "Das offene Labor"
  ,"Every reagent and instrument is available. Build freely without changing Story progress.": "Alle Reagenzien und Geräte sind verfügbar. Baue frei, ohne den Story-Fortschritt zu verändern."
  ,"everything unlocked": "alles freigeschaltet"
  ,"free exploration": "freies Erkunden"
  ,"enter Sandbox": "Sandbox betreten"
  ,"switch to Sandbox": "zur Sandbox wechseln"
  ,"Your saves stay separate.": "Deine Spielstände bleiben getrennt."
  ,"Clearing or experimenting in Sandbox never changes your Story laboratory.": "Leeren oder Experimentieren in der Sandbox verändert niemals dein Story-Labor."
};

function detectLocale(): Locale {
  if (typeof window !== "undefined") {
    try {
      const saved = window.localStorage.getItem("kerotakis.locale");
      if (saved === "en" || saved === "de") return saved;
    } catch {
      // Storage may be unavailable in privacy modes; browser language remains enough.
    }
  }
  return typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("de")
    ? "de"
    : "en";
}

class I18n {
  locale = $state<Locale>(detectLocale());

  constructor() {
    this.applyDocumentLanguage();
  }

  setLocale(locale: Locale) {
    this.locale = locale;
    if (typeof window !== "undefined") {
      try {
        window.localStorage.setItem("kerotakis.locale", locale);
      } catch {
        // The live choice still works when persistence is blocked.
      }
    }
    this.applyDocumentLanguage();
  }

  t(message: string, vars: Vars = {}): string {
    const template = this.locale === "de" ? (DE[message] ?? message) : message;
    return template.replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? `{${key}}`));
  }

  private applyDocumentLanguage() {
    if (typeof document === "undefined") return;
    document.documentElement.lang = this.locale;
    document.documentElement.dir = "ltr";
    document.title = this.t("Kerotakis — the bench");
    document
      .querySelector<HTMLMetaElement>('meta[name="description"]')
      ?.setAttribute(
        "content",
        this.t(
          "A virtual chemistry laboratory that computes real chemistry — drag reagents onto drawn glassware and watch a real aqueous solver answer. Offline once loaded.",
        ),
      );
  }
}

export const i18n = new I18n();
export const t = (message: string, vars?: Vars) => i18n.t(message, vars);
