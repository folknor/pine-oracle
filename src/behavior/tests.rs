use super::*;

    #[test]
    fn function_lookup_carries_polymorphism_flag_when_present() {
        let b = lookup("input").expect("input must exist");
        match b {
            Behavior::Function(f) => {
                assert_eq!(f.name, "input");
                assert!(
                    f.flags.is_polymorphic(),
                    "input carries a polymorphic flag in functions.json"
                );
                assert_eq!(f.flags.polymorphic.as_deref(), Some("input"));
                assert_eq!(f.flags.return_type_param.as_deref(), Some("defval"));
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn non_polymorphic_function_has_no_polymorphic_flag() {
        let b = lookup("plot").expect("plot must exist");
        match b {
            Behavior::Function(f) => {
                assert!(!f.flags.is_polymorphic(), "plot is not polymorphic");
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
        assert!(snapshot.type_count > 0);
        assert!(snapshot.annotation_count > 0);
        assert!(snapshot.polymorphic_function_count > 0);
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
    fn polymorphism_rule_maps_known_flag_values() {
        let identity = FunctionFlags {
            polymorphic: Some("input".to_string()),
            ..FunctionFlags::default()
        };
        assert_eq!(
            syntax_polymorphism_rule(&identity),
            Some(PolymorphismRule::Identity)
        );

        let numeric = FunctionFlags {
            polymorphic: Some("numeric".to_string()),
            ..FunctionFlags::default()
        };
        assert_eq!(
            syntax_polymorphism_rule(&numeric),
            Some(PolymorphismRule::Numeric)
        );

        let element = FunctionFlags {
            polymorphic: Some("element".to_string()),
            ..FunctionFlags::default()
        };
        assert_eq!(
            syntax_polymorphism_rule(&element),
            Some(PolymorphismRule::CollectionElement)
        );
    }

    // return-follows-source functions (ta.valuewhen, ta.change, ...) carry only
    // `returnTypeParam` and no `polymorphic` flag; they map to Identity.
    #[test]
    fn return_type_param_only_maps_to_identity() {
        let flags = FunctionFlags {
            return_type_param: Some("source".to_string()),
            ..FunctionFlags::default()
        };
        assert_eq!(
            syntax_polymorphism_rule(&flags),
            Some(PolymorphismRule::Identity)
        );
    }

    // An unknown future `polymorphic` value must not install an inert rule.
    #[test]
    fn unknown_polymorphism_value_is_not_inserted() {
        let flags = FunctionFlags {
            polymorphic: Some("future-strategy".to_string()),
            ..FunctionFlags::default()
        };
        assert_eq!(syntax_polymorphism_rule(&flags), None);
    }

    // Monomorphic functions install no polymorphism rule.
    #[test]
    fn monomorphic_function_has_no_rule() {
        assert_eq!(syntax_polymorphism_rule(&FunctionFlags::default()), None);
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
    // `input` carries `polymorphic: "input"` in the functions.json flags.
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

    // --- new upstream catalogs: types + annotations ---

    // chart.point is an object type carrying index/time/price fields.
    #[test]
    fn type_lookup_carries_fields() {
        let b = lookup("chart.point").expect("chart.point type must exist");
        match b {
            Behavior::Type(t) => {
                assert_eq!(t.name, "chart.point");
                assert_eq!(t.namespace.as_deref(), Some("chart"));
                assert_eq!(t.classification, "object");
                let field_names: Vec<&str> = t.fields.iter().map(|f| f.name.as_str()).collect();
                assert!(field_names.contains(&"index"));
                assert!(field_names.contains(&"time"));
                assert!(field_names.contains(&"price"));
            }
            other => panic!("expected Type, got {other:?}"),
        }
    }

    // Container type names (array / matrix / map) are not shadowed by a cast
    // function, so a bare `lookup` resolves them to their Type entry. The
    // primitive type names (int, float, ...) ARE shadowed by their cast
    // functions, so they surface only via `--list --kind type` and search.
    #[test]
    fn container_type_lookup_works() {
        let b = lookup("matrix").expect("matrix type must exist");
        assert_eq!(b.kind(), BehaviorKind::Type);
        if let Behavior::Type(t) = b {
            assert_eq!(t.classification, "container");
        }
    }

    // Primitive type names are reachable through the type listing even though a
    // bare `lookup` resolves to the cast function of the same name.
    #[test]
    fn primitive_types_are_listable() {
        let entries = list(Some("type"), None).expect("list");
        assert!(
            entries
                .iter()
                .any(|e| e.name == "int" && e.detail == "primitive"),
            "int must appear in the type listing as a primitive"
        );
    }

    // @version= is the canonical compiler annotation; lookup must surface it.
    #[test]
    fn annotation_lookup_works() {
        let b = lookup("@version=").expect("@version= annotation must exist");
        match b {
            Behavior::Annotation(a) => {
                assert_eq!(a.name, "@version=");
                assert!(!a.description.is_empty());
            }
            other => panic!("expected Annotation, got {other:?}"),
        }
    }

    #[test]
    fn behavior_kind_catalog_includes_types_and_annotations() {
        let kinds = kind_catalog();
        assert_eq!(
            kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(),
            vec![
                BehaviorKind::Function,
                BehaviorKind::Variable,
                BehaviorKind::Constant,
                BehaviorKind::Keyword,
                BehaviorKind::Type,
                BehaviorKind::Annotation,
                BehaviorKind::Operator,
            ]
        );
        assert!(kinds.iter().all(|kind| kind.count > 0));
    }

    // keywords.json is mid-migration from a bare string[] to objects with
    // prose. The untagged RawKeywordEntry must accept both in the same array so
    // the binary works before and after the schema lands.
    #[test]
    fn keywords_accept_bare_and_object_shapes() {
        let json = r#"["for", {"name":"switch","remarks":"only one block runs","seeAlso":["if","?:"]}]"#;
        let entries: Vec<RawKeywordEntry> = serde_json::from_str(json).expect("must parse mixed");
        let kws: Vec<RawKeyword> = entries
            .into_iter()
            .map(RawKeywordEntry::into_keyword)
            .collect();
        // Bare string -> name only, no prose.
        assert_eq!(kws[0].name, "for");
        assert!(kws[0].remarks.is_none());
        assert!(kws[0].see_also.is_empty());
        // Object -> name + prose sub-sections.
        assert_eq!(kws[1].name, "switch");
        assert_eq!(kws[1].remarks.as_deref(), Some("only one block runs"));
        assert_eq!(kws[1].see_also, vec!["if", "?:"]);
    }

    // Against the real vendored keywords.json (now object-form), a keyword must
    // surface its prose. `switch` carries description, remarks, see-also, and a
    // returns sentence.
    #[test]
    fn keyword_carries_prose_from_vendored_data() {
        let b = lookup("switch").expect("switch keyword must exist");
        match b {
            Behavior::Keyword(k) => {
                assert_eq!(k.name, "switch");
                assert!(!k.description.is_empty(), "switch has a description");
                assert!(k.remarks.is_some(), "switch has remarks");
                assert!(
                    k.see_also.iter().any(|s| s == "if"),
                    "switch see-also includes if; got {:?}",
                    k.see_also
                );
                assert!(k.returns_description.is_some());
            }
            other => panic!("expected Keyword, got {other:?}"),
        }
    }

    // Operators are a first-class catalog now: lookup must resolve a symbol and
    // carry its prose sub-sections.
    #[test]
    fn operator_lookup_works() {
        let b = lookup("-").expect("operator - must exist");
        match b {
            Behavior::Operator(o) => {
                assert_eq!(o.name, "-");
                assert!(!o.description.is_empty());
                assert!(
                    o.returns_description.is_some(),
                    "the `-` operator carries a prose Returns sentence"
                );
            }
            other => panic!("expected Operator, got {other:?}"),
        }
    }

    // A function carries the new prose sub-sections straight from pine-data.
    #[test]
    fn function_carries_prose_subsections() {
        let b = lookup("ta.sma").expect("ta.sma must exist");
        match b {
            Behavior::Function(f) => {
                assert_eq!(
                    f.remarks.as_deref(),
                    Some("na values in the source series are ignored.")
                );
                assert!(f.see_also.iter().any(|s| s == "ta.ema"));
                assert!(f.returns_description.is_some());
                // Per-argument prose now comes from pine-data directly.
                let source = f
                    .parameters
                    .iter()
                    .find(|p| p.name == "source")
                    .expect("source param");
                assert_eq!(source.description, "Series of values to process.");
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn list_filters_to_annotation_kind() {
        let entries = list(Some("annotation"), None).expect("list");
        assert!(!entries.is_empty(), "expected annotation entries");
        assert!(
            entries
                .iter()
                .all(|entry| entry.kind == BehaviorKind::Annotation)
        );
        assert!(entries.iter().any(|entry| entry.name == "@param"));
    }

    // --- new upstream function richness ---

    // request.quandl is the one deprecated function in the v6 reference.
    #[test]
    fn deprecated_function_carries_note() {
        let b = lookup("request.quandl").expect("request.quandl must exist");
        match b {
            Behavior::Function(f) => {
                assert!(
                    f.deprecated.is_some(),
                    "request.quandl must carry a deprecation note"
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    // Overloaded functions expose their per-overload signatures.
    #[test]
    fn input_exposes_overloads() {
        let b = lookup("input").expect("input must exist");
        match b {
            Behavior::Function(f) => {
                assert!(
                    f.overloads.len() > 1,
                    "input is overloaded across return types, got {}",
                    f.overloads.len()
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    // Enumerated params expose their allowedValues set.
    #[test]
    fn enumerated_param_exposes_allowed_values() {
        let b = lookup("input").expect("input must exist");
        match b {
            Behavior::Function(f) => {
                let display = f
                    .parameters
                    .iter()
                    .find(|p| p.name == "display")
                    .expect("input has a display param");
                assert!(
                    display.allowed_values.iter().any(|v| v == "display.all"),
                    "display param must enumerate display.all, got {:?}",
                    display.allowed_values
                );
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }
