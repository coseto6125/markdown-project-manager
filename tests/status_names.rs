use markdown_project_manager::model::Status;

#[test]
fn test_status_name_round_trips_for_every_variant() {
    for status in Status::ALL {
        assert_eq!(status.as_str().parse::<Status>(), Ok(status));
    }
}

#[test]
fn test_status_all_has_no_duplicate_variant_or_name() {
    for (i, status) in Status::ALL.iter().enumerate() {
        for other in &Status::ALL[i + 1..] {
            assert_ne!(status, other);
            assert_ne!(status.as_str(), other.as_str());
        }
    }
}

#[test]
fn test_status_parse_of_an_unknown_name_lists_the_valid_names() {
    let err = "nope".parse::<Status>().unwrap_err();
    assert!(err.contains("`nope`"), "{err}");
    for status in Status::ALL {
        assert!(err.contains(status.as_str()), "{err}");
    }
}

#[test]
fn test_status_parse_is_case_and_space_exact() {
    assert!("Open".parse::<Status>().is_err());
    assert!(" open".parse::<Status>().is_err());
    assert!("".parse::<Status>().is_err());
}
