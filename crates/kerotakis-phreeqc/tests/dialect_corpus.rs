#![cfg(all(
    feature = "engine",
    feature = "my-basic",
))]

use kerotakis_phreeqc::{databases, Phreeqc};

#[test]
fn kinetics_rate_with_arithmetic_and_save() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "RATES\n\
             SimpleRate\n\
             -start\n\
             10 rate = PARM(1) * M * TIME\n\
             20 IF rate > M THEN rate = M\n\
             30 SAVE rate\n\
             -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             KINETICS 1\n\
                 SimpleRate\n\
                     -formula H2O 0\n\
                     -m 1\n\
                     -m0 1\n\
                     -parms 0.1\n\
                     -steps 1 second\n\
             END\n",
        )
        .unwrap();
}

#[test]
fn for_next_loop_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 ForLoop\n\
                 -start\n\
                 10 total = 0\n\
                 20 FOR i = 1 TO 5\n\
                 30   total = total + i\n\
                 40 NEXT i\n\
                 50 SAVE total\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values ForLoop\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_ForLoop").expect("calculated value");
    assert!((value - 15.0).abs() < 1e-10, "sum(1..5) = {value}");
}

#[test]
fn while_wend_loop_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 WhileLoop\n\
                 -start\n\
                 10 n = 1\n\
                 20 WHILE n < 100\n\
                 30   n = n * 2\n\
                 40 WEND\n\
                 50 SAVE n\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values WhileLoop\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_WhileLoop").expect("calculated value");
    assert!((value - 128.0).abs() < 1e-10, "2^7 = {value}");
}

#[test]
fn for_loop_supports_descending_steps_and_captures_its_limit() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DescendingFor\n\
                 -start\n\
                 10 lower = 3\n\
                 20 total = 0\n\
                 30 FOR i = 5 TO lower STEP -1\n\
                 40   lower = 100\n\
                 50   total = total + i\n\
                 60 NEXT i\n\
                 70 SAVE total\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values DescendingFor\n\
             END\n",
        )
        .unwrap();
    assert_eq!(engine.last_value("V_DescendingFor"), Some(12.0));
}

#[test]
fn dim_uses_inclusive_bounds_and_accepts_multiple_arrays() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 DimCompatibility\n\
                 -start\n\
                 10 n = 2\n\
                 20 DIM a(n), marker$(1)\n\
                 30 a(0) = 1\n\
                 40 a(2) = 3\n\
                 50 marker$(1) = \"ok\"\n\
                 60 SAVE a(0) + a(2) + LEN(marker$(1))\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values DimCompatibility\n\
             END\n",
        )
        .unwrap();
    assert_eq!(engine.last_value("V_DimCompatibility"), Some(6.0));
}

#[test]
fn instr_is_one_based_and_accepts_an_optional_start() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 InstrCompatibility\n\
                 -start\n\
                 10 first = INSTR(\"banana\", \"na\")\n\
                 20 second = INSTR(4, \"banana\", \"na\")\n\
                 30 missing = INSTR(\"banana\", \"zz\")\n\
                 40 SAVE first * 100 + second * 10 + missing\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values InstrCompatibility\n\
             END\n",
        )
        .unwrap();
    assert_eq!(engine.last_value("V_InstrCompatibility"), Some(350.0));
}

#[test]
fn gosub_return_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 GosubTest\n\
                 -start\n\
                 10 x = 5\n\
                 20 GOSUB 100\n\
                 30 SAVE x\n\
                 40 GOTO 200\n\
                 100 x = x * x\n\
                 110 RETURN\n\
                 200 REM done\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values GosubTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_GosubTest").expect("calculated value");
    assert!((value - 25.0).abs() < 1e-10, "5^2 = {value}");
}

#[test]
fn if_then_else_compiles_and_runs() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "CALCULATE_VALUES\n\
                 IfTest\n\
                 -start\n\
                 10 x = 10\n\
                 20 IF x > 5 THEN y = 1 ELSE y = 0\n\
                 30 SAVE y\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values IfTest\n\
             END\n",
        )
        .unwrap();
    let value = engine.last_value("V_IfTest").expect("calculated value");
    assert!((value - 1.0).abs() < 1e-10, "IF 10>5 THEN 1 = {value}");
}

#[test]
fn string_operations_in_punch() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings marker\n\
                 10 a$ = \"Hello\"\n\
                 20 b$ = \" World\"\n\
                 30 PUNCH a$ + b$\n\
             END\n",
        )
        .unwrap();
    let rows = engine.selected_output();
    let values = rows.last().expect("selected-output values");
    assert_eq!(values[0], "Hello World");
}

#[test]
fn runtime_scalar_names_do_not_capture_string_variables() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings marker conductivity\n\
                 10 sc$ = \"semicolon\"\n\
                 20 PUNCH sc$, SC\n\
             END\n",
        )
        .unwrap();
    let values = engine.selected_output().last().unwrap().clone();
    assert_eq!(values[0], "semicolon");
    assert!(values[1].parse::<f64>().unwrap().is_finite());
}

#[test]
fn unsupported_function_fails_at_compile_time() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let error = engine
        .run(
            "CALCULATE_VALUES\n\
                 BadFunc\n\
                 -start\n\
                 10 SAVE KEROTAKIS_UNKNOWN_FUNCTION(1)\n\
                 -end\n\
             SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
                 -calculate_values BadFunc\n\
             END\n",
        )
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("MY-BASIC compatibility"),
        "unregistered function should fail: {error}"
    );
}

#[test]
fn user_graph_exposes_renderer_neutral_series_and_points() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             REACTION_TEMPERATURE 1\n\
                 25 26 in 2 steps\n\
             USER_GRAPH 7 Compatibility graph\n\
                 -headings Temperature Double_T Offset_T pH\n\
                 -chart_title \"Renderer-neutral graph\"\n\
                 -axis_titles \"Temperature\" \"Primary\" \"Secondary\"\n\
                 -initial_solutions false\n\
                 10 GRAPH_X TC\n\
                 20 GRAPH_Y TC * 2, TC + 1\n\
                 30 GRAPH_SY -LA(\"H+\")\n\
             END\n",
        )
        .unwrap();

    let graphs = engine.user_graph_data().unwrap();
    assert_eq!(graphs.charts.len(), 1);
    let chart = &graphs.charts[0];
    assert_eq!(chart.user_number, 7);
    assert_eq!(chart.title, "Renderer-neutral graph");
    assert_eq!(chart.axis_titles, ["Temperature", "Primary", "Secondary"]);
    assert_eq!(chart.series.len(), 3);
    assert_eq!(chart.series[0].id, "Double_T");
    assert_eq!(chart.series[1].id, "Offset_T");
    assert_eq!(chart.series[2].id, "pH");
    assert_eq!(chart.series[2].y_axis, 2);
    assert_eq!(chart.series[0].points, [[25.0, 50.0], [26.0, 52.0]]);
    assert_eq!(chart.series[1].points, [[25.0, 26.0], [26.0, 27.0]]);
    assert_eq!(chart.series[2].points.len(), 2);
    assert!(chart.series[2]
        .points
        .iter()
        .all(|point| point[1].is_finite() && point[1] > 6.0 && point[1] < 8.0));
}

#[test]
fn conditional_plot_xy_keeps_its_source_line_style() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             REACTION 1\n\
                 H2O 1e-6\n\
             USER_GRAPH 3\n\
                 -headings Kept\n\
                 -initial_solutions false\n\
                 10 IF 1 = 1 THEN GOTO 30\n\
                 20 PLOT_XY 1, 1, color = Red, symbol = Square\n\
                 30 PLOT_XY 2, 3, color = Blue, symbol = Circle\n\
             END\n",
        )
        .unwrap();
    let graphs = engine.user_graph_data().unwrap();
    let series = &graphs.charts[0].series[0];
    assert_eq!(series.color, "Blue");
    assert_eq!(series.symbol, "Circle");
    assert_eq!(series.points, [[2.0, 3.0]]);
}

#[test]
fn extended_native_callback_families_return_finite_values() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Na 0.01\n\
                 Cl 0.01\n\
             GAS_PHASE 1\n\
                 -fixed_pressure\n\
                 CO2(g) 0.0004\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings dhp a0 bdot dc gam lg lkp lks vm phi tsc alk aphi cb dha dhav dhb eps iter kap kt mu osm pe pot pressure qbrn rho rho0 sc\n\
                 10 PUNCH DELTA_H_PHASE(\"Calcite\"), DH_A0(\"Na+\"), DH_BDOT(\"Na+\"), DIFF_C(\"Na+\")\n\
                 20 PUNCH GAMMA(\"Na+\"), LG(\"Na+\"), LK_PHASE(\"Calcite\"), LK_SPECIES(\"Na+\")\n\
                 30 PUNCH PHASE_VM(\"Calcite\"), PR_PHI(\"CO2(g)\"), T_SC(\"Na+\")\n\
                 40 PUNCH ALK, APHI, CHARGE_BALANCE, DH_A, DH_Av, DH_B, EPS_R, ITERATIONS, KAPPA, KIN_TIME\n\
                 50 PUNCH MU, OSMOTIC, PERCENT_ERROR, POT_V, PRESSURE, QBrn, RHO, RHO_0, SC\n\
             END\n",
        )
        .unwrap();
    let rows = engine.selected_output();
    let values = rows.last().unwrap();
    assert_eq!(values.len(), 31);
    assert_eq!(values.last().unwrap(), "");
    for (index, value) in values[..30].iter().enumerate() {
        let parsed: f64 = value
            .parse()
            .unwrap_or_else(|_| panic!("callback column {index} was not numeric: {value:?}"));
        assert!(parsed.is_finite(), "callback column {index} was {parsed}");
    }
}

#[test]
fn persistent_strings_and_remaining_scalar_helpers_work() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "TITLE Adapter helpers\n\
             SOLUTION 1\n\
                 pH 7\n\
                 Na 0.01\n\
                 Cl 0.01\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings stored left right unit title visc diff por jt jc\n\
                 10 PUT$(\"saved\", 2, 3)\n\
                 20 d = SETDIFF_C(\"Na+\", 1e-9)\n\
                 30 PUNCH GET$(2, 3), LTRIM(\"  left  \"), RTRIM(\"  right  \"), ISO_UNIT(\"13C\"), TITLE\n\
                 40 PUNCH F_VISC(\"Na+\"), DIFF_C(\"Na+\"), GET_POR(1), MCD_JTOT(\"Na+\"), MCD_JCONC(\"Na+\")\n\
             END\n",
        )
        .unwrap();
    let rows = engine.selected_output();
    let values = rows.last().unwrap();
    assert_eq!(
        &values[..5],
        ["saved", "left", "right", "unknown", "Adapter helpers"]
    );
    for value in &values[5..10] {
        assert!(value.parse::<f64>().unwrap().is_finite(), "{value:?}");
    }
}

#[test]
fn native_multi_output_callbacks_fill_basic_arrays() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1 Example solution\n\
                 units mol/kgw\n\
                 pH 7\n\
                 Na 0.01\n\
                 Cl 0.01\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings pf pn pe pc st sn stype smoles sf sfn se sen eq elt frac sys dl description\n\
                 10 pf$ = PHASE_FORMULA$(\"Calcite\", pn, pe$, pc)\n\
                 20 system_amount = SYS(\"elements\", sn, sname$, stype$, smoles)\n\
                 30 sf$ = SPECIES_FORMULA$(\"Na+\", sfn, se$, stoich)\n\
                 40 seq$ = SPECIES_EQUATION$(\"Na+\", sen, seqn$, seqc)\n\
                 50 frac = EQ_FRAC(\"Na+\", eq, elt$)\n\
                 60 dl = DEBYE_LENGTH\n\
                 70 PUNCH pf$, pn, pe$(1), pc(1), system_amount, sn, stype$(1), smoles(1)\n\
                 80 PUNCH sf$, sfn, se$(1), sen, eq, elt$, frac, system_amount, dl, DESCRIPTION\n\
             END\n",
        )
        .unwrap();
    let rows = engine.selected_output();
    let values = rows.last().unwrap();
    assert_eq!(values[0], "CaCO3");
    assert!(values[1].parse::<f64>().unwrap() >= 2.0);
    assert!(!values[2].is_empty());
    assert!(values[3].parse::<f64>().unwrap().is_finite());
    assert!(values[5].parse::<f64>().unwrap() >= 2.0);
    assert!(!values[6].is_empty());
    assert!(values[7].parse::<f64>().unwrap().is_finite());
    assert_eq!(values[8], "aq");
    assert!(values[9].parse::<f64>().unwrap() >= 2.0);
    assert!(!values[10].is_empty());
    assert!(values[11].parse::<f64>().unwrap() >= 1.0);
    assert!(values[15].parse::<f64>().unwrap().is_finite());
    assert!(values[16].parse::<f64>().unwrap() > 0.0);
    assert_eq!(values[17], "Example solution");
    assert_eq!(values[18], "");
}

#[test]
fn phreeqc_string_format_and_line_control_helpers_work() {
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(
            "SOLUTION 1\n\
                 pH 7\n\
             SELECTED_OUTPUT\n\
                 -reset false\n\
             USER_PUNCH\n\
                 -headings scientific fixed marker\n\
                 10 PUNCH STR_E$(12.5, 10, 2), STR_F$(12.5, 8, 2), \"done\"\n\
             END\n",
        )
        .unwrap();
    let values = engine.selected_output().last().unwrap().clone();
    assert_eq!(values[0].trim(), "1.25e+01");
    assert_eq!(values[1].trim(), "12.50");
    assert_eq!(values[2], "done");
}
