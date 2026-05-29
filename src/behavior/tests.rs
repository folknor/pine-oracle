use super::*;

    #[test]
    fn function_lookup_carries_behavior_when_present() {
        let b = lookup("input").expect("input must exist");
        match b {
            Behavior::Function(f) => {
                assert_eq!(f.name, "input");
                let beh = f.behavior.expect("input is polymorphic per behavior.json");
                assert!(beh.polymorphic.is_polymorphic());
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn non_polymorphic_function_has_static_polymorphic_field() {
        let b = lookup("plot").expect("plot must exist");
        match b {
            Behavior::Function(f) => {
                if let Some(beh) = f.behavior {
                    assert!(!beh.polymorphic.is_polymorphic());
                }
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn variable_lookup_carries_qualifier() {
        let b = lookup("close").expect("close must exist");
        match b {
            Behavior::Variable(v) => {
                assert_eq!(v.qualifier, "series");
                assert!(v.ty.contains("float"));
            }
            other => panic!("expected Variable, got {other:?}"),
        }
    }

    #[test]
    fn constant_lookup_works() {
        let b = lookup("color.red").or_else(|| lookup("adjustment.none"));
        assert!(b.is_some(), "at least one known constant should be found");
    }

    #[test]
    fn keyword_lookup_works() {
        let b = lookup("var").expect("var keyword must exist");
        assert!(matches!(b, Behavior::Keyword(_)));
    }

    #[test]
    fn snapshot_reports_vendored_pine_data_metadata() {
        let snapshot = snapshot();
        assert_eq!(snapshot.version, "6");
        chrono::DateTime::parse_from_rfc3339(&snapshot.generated_at)
            .expect("generated_at must be RFC3339");
        assert_eq!(snapshot.function_count, 475);
        assert!(snapshot.variable_count > 0);
        assert!(snapshot.constant_count > 0);
        assert!(snapshot.keyword_count > 0);
        assert!(snapshot.function_behavior_count > 0);
    }

    #[test]
    fn behavior_list_filters_by_kind_and_grep() {
        let entries = list(Some("function"), Some("plotshape")).expect("list");
        assert!(entries.iter().any(|entry| entry.name == "plotshape"
            && entry.kind == BehaviorKind::Function
            && entry.detail.contains("plotshape(")));
        assert!(
            entries
                .iter()
                .all(|entry| entry.kind == BehaviorKind::Function)
        );
    }

    #[test]
    fn behavior_grep_does_not_match_kind_names() {
        let entries = list(None, Some("variable")).expect("list");
        assert!(
            !entries
                .iter()
                .any(|entry| entry.kind == BehaviorKind::Variable && entry.name == "close"),
            "grep should not match every variable entry by kind name"
        );
    }

    #[test]
    fn behavior_kind_filter_is_case_insensitive() {
        let entries = list(Some("Function"), Some("plotshape")).expect("list");
        assert!(entries.iter().any(|entry| entry.name == "plotshape"));
        assert!(
            entries
                .iter()
                .all(|entry| entry.kind == BehaviorKind::Function)
        );
    }

    #[test]
    fn behavior_kind_catalog_reports_all_kinds() {
        let kinds = kind_catalog();
        assert_eq!(
            kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(),
            vec![
                BehaviorKind::Function,
                BehaviorKind::Variable,
                BehaviorKind::Constant,
                BehaviorKind::Keyword
            ]
        );
        assert!(kinds.iter().all(|kind| kind.count > 0));
    }

    #[test]
    fn invalid_behavior_kind_errors() {
        let err = list(Some("functionsish"), None).expect_err("must reject");
        assert!(err.to_string().contains("unknown behavior kind"));
    }

    #[test]
    fn syntax_builtins_include_pine_data_functions() {
        let builtins = syntax_builtins();
        let signatures = builtins
            .function_signatures("math.sqrt")
            .expect("math.sqrt signature");
        assert!(!signatures.is_empty());
        assert!(builtins.variables.contains_key("close"));
        assert!(builtins.constants.contains_key("color.red"));
        assert!(builtins.polymorphism.contains_key("nz"));
    }

    #[test]
    fn syntax_builtins_add_doc_union_overloads_to_runtime_functions() {
        let builtins = syntax_builtins();
        let signatures = builtins
            .function_signatures("plotshape")
            .expect("plotshape signature");
        assert!(
            signatures.iter().any(|signature| {
                signature.params.first().is_some_and(|param| {
                    param.name == "series" && param.value_type == ValueType::Bool
                })
            }),
            "plotshape should accept bool series per pine-data, got {signatures:?}"
        );
    }

    #[test]
    fn syntax_builtins_skip_generic_placeholder_function_names() {
        let builtins = syntax_builtins();
        assert!(builtins.function_signatures("array.new<type>").is_none());
    }

    #[test]
    fn unknown_polymorphism_strategy_is_not_inserted() {
        let entry = RawBehaviorEntry {
            polymorphic: PolymorphicField::Dynamic(PolymorphicDetail {
                return_type_param: Some("source".to_string()),
                strategy: Some("future-strategy".to_string()),
                observed_mappings: HashMap::new(),
                allowed_types: Vec::new(),
            }),
            argument_ordering: None,
            observed_return_types: Vec::new(),
            reason: None,
        };
        assert_eq!(syntax_polymorphism_rule(&entry), None);
    }

    #[test]
    fn parses_common_type_strings_for_syntax() {
        assert_eq!(parse_value_type("series<int>").describe(), "int");
        assert_eq!(parse_value_type("series int/float").describe(), "float");
        assert_eq!(parse_value_type("array<float>").describe(), "array<float>");
        assert_eq!(
            parse_value_type("map<string, float>").describe(),
            "map<string, float>"
        );
        assert_eq!(parse_value_type("chart.point").describe(), "chart.point");
    }

    #[test]
    fn parses_top_level_param_unions_into_alternatives() {
        let alternatives = parse_param_value_type_alternatives("series int/float/bool");
        assert_eq!(
            alternatives,
            vec![ValueType::Int, ValueType::Float, ValueType::Bool]
        );
    }

    #[test]
    fn unknown_name_is_none() {
        assert!(lookup("definitely-not-a-pine-symbol").is_none());
    }

    #[test]
    fn alertcondition_top_level_only_flag_is_parsed() {
        // alertcondition is one of the 14 functions with topLevelOnly:true in
        // functions.json. The serde rename must fire or this returns false.
        let b = lookup("alertcondition").expect("alertcondition must exist");
        match b {
            Behavior::Function(f) => {
                assert!(
                    f.flags.top_level_only,
                    "alertcondition should have top_level_only=true (topLevelOnly in JSON)"
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn array_from_variadic_flags_are_parsed() {
        // array.from has variadic:true and minArgs:1 in functions.json.
        let b = lookup("array.from").expect("array.from must exist");
        match b {
            Behavior::Function(f) => {
                assert!(f.flags.variadic, "array.from should be variadic");
                assert_eq!(f.flags.min_args, Some(1), "array.from minArgs should be 1");
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn lookup_is_case_insensitive() {
        // Variables are stored with their original casing (e.g. "close").
        // Querying with "CLOSE" must return the same entry.
        let lower = lookup("close").expect("close must exist");
        let upper = lookup("CLOSE").expect("CLOSE must return same entry");
        match (lower, upper) {
            (Behavior::Variable(a), Behavior::Variable(b)) => {
                assert_eq!(
                    a.name, b.name,
                    "both lookups must resolve to the same symbol"
                );
            }
            other => panic!("expected two Variables, got {other:?}"),
        }
    }

    #[test]
    fn lookup_case_insensitive_function() {
        let lower = lookup("plot").expect("plot must exist");
        let mixed = lookup("PLOT").expect("PLOT case-insensitive lookup must work");
        match (lower, mixed) {
            (Behavior::Function(a), Behavior::Function(b)) => {
                assert_eq!(a.name, b.name);
            }
            other => panic!("expected two Functions, got {other:?}"),
        }
    }

    // Regression: pine-data switched from `example: string` to `examples: string[]`.
    // If upstream regresses, `#[serde(default)]` would silently drop examples.
    // This test pins that `alert` -- which ships at least one example in the
    // canonical v6 data -- continues to surface it.
    #[test]
    fn alert_has_at_least_one_example() {
        let b = lookup("alert").expect("alert must exist in pine-data");
        match b {
            Behavior::Function(f) => {
                assert!(
                    !f.examples.is_empty(),
                    "alert must have at least one example (upstream schema regression guard)"
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    // Generic-placeholder fallback: `array.new<type>` is stored under that exact
    // name in pine-data. A bare `array.new` query must resolve via the fallback.
    #[test]
    fn lookup_generic_placeholder_array_new() {
        let b =
            lookup("array.new").expect("array.new must resolve via generic-placeholder fallback");
        match b {
            Behavior::Function(f) => {
                assert!(
                    f.name.starts_with("array.new"),
                    "resolved name must start with array.new, got: {}",
                    f.name
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    // map.new<type,type> has a two-type-param suffix; the fallback must prefer
    // the `<type,type>` probe over `<type>` when both might match.
    #[test]
    fn lookup_generic_placeholder_map_new() {
        let b = lookup("map.new").expect("map.new must resolve via generic-placeholder fallback");
        match b {
            Behavior::Function(f) => {
                assert!(
                    f.name.starts_with("map.new"),
                    "resolved name must start with map.new, got: {}",
                    f.name
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    // Verify that `Behavior::kind()` returns the correct discriminant.
    #[test]
    fn behavior_kind_accessor_round_trips() {
        assert_eq!(
            lookup("plot").unwrap().kind(),
            BehaviorKind::Function,
            "plot is a function"
        );
        assert_eq!(
            lookup("close").unwrap().kind(),
            BehaviorKind::Variable,
            "close is a variable"
        );
        assert_eq!(
            lookup("color.red").unwrap().kind(),
            BehaviorKind::Constant,
            "color.red is a constant"
        );
    }

    // `Behavior::is_polymorphic` must return true for a known poly function.
    // `input` carries a Dynamic polymorphic field in function-behavior.json.
    #[test]
    fn is_polymorphic_true_for_input() {
        let b = lookup("input").expect("input must exist");
        assert!(
            b.is_polymorphic(),
            "input should be polymorphic per behavior metadata"
        );
    }

    // Non-polymorphic functions must return false.
    #[test]
    fn is_polymorphic_false_for_alert() {
        let b = lookup("alert").expect("alert must exist");
        assert!(!b.is_polymorphic(), "alert should not be polymorphic");
    }

    // Variables and non-functions are never polymorphic.
    #[test]
    fn is_polymorphic_false_for_variable() {
        let b = lookup("close").expect("close must exist");
        assert!(!b.is_polymorphic(), "variables are never polymorphic");
    }
