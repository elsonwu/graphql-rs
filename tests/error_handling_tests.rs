/// Comprehensive tests for enhanced GraphQL error handling
/// This test suite covers error creation, propagation, formatting, and edge cases
use graphql_rs::domain::value_objects::{
    ErrorFormatter, ErrorPropagation, ExecutionResult, GraphQLError, GraphQLResult, PathSegment,
    SourceLocation, ValidationResult,
};
use serde_json::json;

#[cfg(test)]
mod error_creation_tests {
    use super::*;

    #[test]
    fn test_basic_error_creation() {
        let error = GraphQLError::new("Test error".to_string());
        assert_eq!(error.message, "Test error");
        assert!(error.locations.is_empty());
        assert!(error.path.is_none());
        assert!(error.extensions.is_none());
    }

    #[test]
    fn test_validation_error_with_code() {
        let error = GraphQLError::validation_error("Invalid field".to_string());
        assert_eq!(error.message, "Invalid field");
        assert_eq!(error.error_code(), Some("VALIDATION_ERROR"));
    }

    #[test]
    fn test_field_not_found_error() {
        let error = GraphQLError::field_not_found("name", "User");
        assert_eq!(error.message, "Field 'name' not found on type 'User'");
        assert_eq!(error.error_code(), Some("FIELD_NOT_FOUND"));
        assert!(error.is_client_error());
    }

    #[test]
    fn test_auth_error() {
        let error = GraphQLError::auth_error("Invalid token".to_string());
        assert_eq!(error.error_code(), Some("AUTH_ERROR"));
        assert!(error.is_client_error());
    }

    #[test]
    fn test_execution_error() {
        let error = GraphQLError::execution_error("Database connection failed".to_string());
        assert_eq!(error.error_code(), Some("EXECUTION_ERROR"));
        assert!(error.is_server_error());
    }

    #[test]
    fn test_rate_limit_error() {
        let error = GraphQLError::rate_limit_error();
        assert_eq!(error.message, "Rate limit exceeded");
        assert_eq!(error.error_code(), Some("RATE_LIMIT_EXCEEDED"));
    }
}

#[cfg(test)]
mod error_enhancement_tests {
    use super::*;

    #[test]
    fn test_error_with_location() {
        let error = GraphQLError::new("Test".to_string()).with_location(2, 5);

        assert_eq!(error.locations.len(), 1);
        assert_eq!(error.locations[0].line, 2);
        assert_eq!(error.locations[0].column, 5);
    }

    #[test]
    fn test_error_with_multiple_locations() {
        let locations = vec![
            SourceLocation { line: 1, column: 3 },
            SourceLocation { line: 4, column: 7 },
        ];

        let error = GraphQLError::new("Test".to_string()).with_locations(locations.clone());

        assert_eq!(error.locations.len(), 2);
        assert_eq!(error.locations, locations);
    }

    #[test]
    fn test_error_with_path() {
        let path = vec![
            PathSegment::Field("user".to_string()),
            PathSegment::Index(0),
            PathSegment::Field("name".to_string()),
        ];

        let error = GraphQLError::new("Test".to_string()).with_path(path.clone());

        assert_eq!(error.path, Some(path));
    }

    #[test]
    fn test_error_with_extensions() {
        let error = GraphQLError::new("Test".to_string())
            .with_extension("customField", json!("customValue"))
            .with_timestamp()
            .with_request_id("req-123");

        let extensions = error.extensions.unwrap();
        assert_eq!(extensions.get("customField").unwrap(), "customValue");
        assert!(extensions.contains_key("timestamp"));
        assert_eq!(extensions.get("requestId").unwrap(), "req-123");
    }

    #[test]
    fn test_user_message_masking() {
        let client_error = GraphQLError::field_not_found("name", "User");
        assert_eq!(client_error.user_message(true), client_error.message);

        let server_error = GraphQLError::execution_error("Database crashed".to_string());
        assert_eq!(
            server_error.user_message(true),
            "Internal server error occurred"
        );
        assert_eq!(server_error.user_message(false), "Database crashed");
    }
}

#[cfg(test)]
mod execution_result_tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let data = json!({"name": "John"});
        let result = ExecutionResult::success(data.clone());

        assert_eq!(result.data, Some(data));
        assert!(result.errors.is_empty());
        assert!(result.is_success());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_error_result() {
        let errors = vec![GraphQLError::validation_error("Invalid".to_string())];
        let result = ExecutionResult::error(errors.clone());

        assert_eq!(result.data, None);
        assert_eq!(result.errors, errors);
        assert!(!result.is_success());
        assert!(result.has_errors());
    }

    #[test]
    fn test_single_error_result() {
        let error = GraphQLError::field_not_found("name", "User");
        let result = ExecutionResult::single_error(error.clone());

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], error);
    }

    #[test]
    fn test_partial_result() {
        let data = json!({"name": "John", "age": null});
        let errors = vec![GraphQLError::execution_error(
            "Age service unavailable".to_string(),
        )];
        let result = ExecutionResult::partial(data.clone(), errors.clone());

        assert_eq!(result.data, Some(data));
        assert_eq!(result.errors, errors);
        assert!(result.has_errors());
        assert!(!result.is_success());
    }

    #[test]
    fn test_with_error_chaining() {
        let result = ExecutionResult::success(json!({"test": true}))
            .with_error(GraphQLError::validation_error("Warning".to_string()));

        assert!(result.data.is_some());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_errors_by_code() {
        let errors = vec![
            GraphQLError::validation_error("Error 1".to_string()),
            GraphQLError::field_not_found("field", "Type"),
            GraphQLError::validation_error("Error 2".to_string()),
        ];

        let result = ExecutionResult::error(errors);
        let validation_errors = result.errors_by_code("VALIDATION_ERROR");
        let field_errors = result.errors_by_code("FIELD_NOT_FOUND");

        assert_eq!(validation_errors.len(), 2);
        assert_eq!(field_errors.len(), 1);
    }

    #[test]
    fn test_client_server_error_detection() {
        let result = ExecutionResult::success(json!({}))
            .with_error(GraphQLError::validation_error("Client error".to_string()))
            .with_error(GraphQLError::execution_error("Server error".to_string()));

        assert!(result.has_client_errors());
        assert!(result.has_server_errors());
    }

    #[test]
    fn test_result_sanitization() {
        let result = ExecutionResult::success(json!({"data": "test"}))
            .with_error(GraphQLError::execution_error(
                "Sensitive server info".to_string(),
            ))
            .with_error(GraphQLError::validation_error("Client error".to_string()));

        let sanitized = result.sanitized(true);

        // Server error should be masked
        assert!(sanitized.errors[0]
            .message
            .contains("Internal server error"));
        // Client error should remain unchanged
        assert_eq!(sanitized.errors[1].message, "Client error");
    }
}

#[cfg(test)]
mod validation_result_tests {
    use super::*;

    #[test]
    fn test_validation_result_states() {
        let valid = ValidationResult::valid();
        let invalid = ValidationResult::invalid("Error".to_string());
        let pending = ValidationResult::Pending;

        assert!(valid.is_valid());
        assert!(!valid.is_invalid());
        assert!(!valid.is_pending());

        assert!(!invalid.is_valid());
        assert!(invalid.is_invalid());
        assert!(!invalid.is_pending());

        assert!(!pending.is_valid());
        assert!(!pending.is_invalid());
        assert!(pending.is_pending());
    }

    #[test]
    fn test_field_error() {
        let result = ValidationResult::field_error("name", "Required field".to_string());

        if let ValidationResult::Invalid(errors) = result {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].message, "Required field");
            assert_eq!(
                errors[0].path,
                Some(vec![PathSegment::Field("name".to_string())])
            );
        } else {
            panic!("Expected Invalid result");
        }
    }

    #[test]
    fn test_type_error() {
        let result = ValidationResult::type_error("User", "Invalid type definition".to_string());

        if let ValidationResult::Invalid(errors) = result {
            assert_eq!(errors.len(), 1);
            let extensions = errors[0].extensions.as_ref().unwrap();
            assert_eq!(extensions.get("type").unwrap(), "User");
        } else {
            panic!("Expected Invalid result");
        }
    }

    #[test]
    fn test_combine_validation_results() {
        let results = vec![
            ValidationResult::valid(),
            ValidationResult::invalid("Error 1".to_string()),
            ValidationResult::valid(),
            ValidationResult::invalid("Error 2".to_string()),
        ];

        let combined = ValidationResult::combine(results);

        if let ValidationResult::Invalid(errors) = combined {
            assert_eq!(errors.len(), 2);
        } else {
            panic!("Expected combined Invalid result");
        }
    }

    #[test]
    fn test_combine_with_pending() {
        let results = vec![
            ValidationResult::valid(),
            ValidationResult::Pending,
            ValidationResult::invalid("Error".to_string()),
        ];

        let combined = ValidationResult::combine(results);
        assert!(combined.is_pending());
    }

    #[test]
    fn test_with_error_chaining() {
        let result = ValidationResult::valid()
            .with_error(GraphQLError::validation_error("New error".to_string()));

        if let ValidationResult::Invalid(errors) = result {
            assert_eq!(errors.len(), 1);
        } else {
            panic!("Expected Invalid result");
        }
    }

    #[test]
    fn test_to_execution_result() {
        let validation_error = ValidationResult::invalid("Test error".to_string());
        let execution_result = validation_error.to_execution_result().unwrap();

        assert!(execution_result.data.is_none());
        assert_eq!(execution_result.errors.len(), 1);
        assert_eq!(execution_result.errors[0].message, "Test error");
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    #[test]
    fn test_propagate_nullable_field_error() {
        let error = GraphQLError::field_not_found("name", "User");
        let path = vec![PathSegment::Field("user".to_string())];

        let (propagated_error, should_bubble) = ErrorPropagation::propagate_field_error(
            error.clone(),
            &path,
            true, // nullable
        );

        assert_eq!(propagated_error.path, Some(path));
        assert!(!should_bubble); // Nullable fields don't bubble
    }

    #[test]
    fn test_propagate_non_nullable_field_error() {
        let error = GraphQLError::execution_error("Service failed".to_string());
        let path = vec![PathSegment::Field("user".to_string())];

        let (propagated_error, should_bubble) = ErrorPropagation::propagate_field_error(
            error.clone(),
            &path,
            false, // non-nullable
        );

        assert_eq!(propagated_error.path, Some(path));
        assert!(should_bubble); // Non-nullable fields bubble up
    }

    #[test]
    fn test_collect_field_errors() {
        let errors = vec![
            (GraphQLError::validation_error("Error 1".to_string()), false),
            (GraphQLError::execution_error("Error 2".to_string()), true),
            (GraphQLError::field_not_found("field", "Type"), false),
        ];

        let (collected_errors, should_bubble) = ErrorPropagation::collect_field_errors(errors);

        assert_eq!(collected_errors.len(), 3);
        assert!(should_bubble); // One error had bubble=true
    }

    #[test]
    fn test_missing_required_field() {
        let parent_path = vec![PathSegment::Field("user".to_string())];
        let error = ErrorPropagation::missing_required_field("id", &parent_path);

        assert!(error.message.contains("Required field 'id' is missing"));
        assert_eq!(
            error.path,
            Some(vec![
                PathSegment::Field("user".to_string()),
                PathSegment::Field("id".to_string()),
            ])
        );
    }

    #[test]
    fn test_null_in_non_nullable() {
        let parent_path = vec![PathSegment::Field("user".to_string())];
        let error = ErrorPropagation::null_in_non_nullable("name", &parent_path);

        assert!(error
            .message
            .contains("Cannot return null for non-nullable field 'name'"));
        assert_eq!(
            error.path,
            Some(vec![
                PathSegment::Field("user".to_string()),
                PathSegment::Field("name".to_string()),
            ])
        );
    }
}

#[cfg(test)]
mod error_formatting_tests {
    use super::*;

    #[test]
    fn test_development_formatting() {
        let error = GraphQLError::execution_error("Test error".to_string())
            .with_location(2, 5)
            .with_path(vec![PathSegment::Field("test".to_string())])
            .with_extension("stackTrace", json!(["line1", "line2"]));

        let formatted = ErrorFormatter::format_development(&error);

        assert_eq!(formatted["message"], "Test error");
        assert_eq!(formatted["locations"][0]["line"], 2);
        assert_eq!(formatted["path"][0], "test");
        assert!(formatted["stackTrace"].is_array());
    }

    #[test]
    fn test_production_formatting() {
        let server_error = GraphQLError::execution_error("Sensitive info".to_string())
            .with_location(2, 5)
            .with_timestamp();

        let formatted = ErrorFormatter::format_production(&server_error);

        assert_eq!(formatted["message"], "Internal server error occurred");
        assert!(formatted["locations"].is_null()); // Hidden for server errors
        assert!(formatted["extensions"]["code"].is_string());
        assert!(formatted["extensions"]["timestamp"].is_number());
    }

    #[test]
    fn test_error_summary() {
        let errors = vec![
            GraphQLError::validation_error("Error 1".to_string()),
            GraphQLError::validation_error("Error 2".to_string()),
            GraphQLError::execution_error("Error 3".to_string()),
            GraphQLError::field_not_found("field", "Type"),
        ];

        let summary = ErrorFormatter::format_error_summary(&errors);

        assert_eq!(summary["total"], 4);
        assert_eq!(summary["client_errors"], 3);
        assert_eq!(summary["server_errors"], 1);

        let by_code = &summary["by_code"];
        assert_eq!(by_code["VALIDATION_ERROR"], 2);
        assert_eq!(by_code["EXECUTION_ERROR"], 1);
        assert_eq!(by_code["FIELD_NOT_FOUND"], 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_error_workflow() {
        // Start with validation
        let mut validation_errors = vec![
            GraphQLError::field_not_found("unknownField", "Query"),
            GraphQLError::validation_error("Missing required argument".to_string()),
        ];

        // Add location info
        validation_errors[0] = validation_errors[0].clone().with_location(1, 10);
        validation_errors[1] = validation_errors[1].clone().with_location(1, 25);

        let validation_result = ValidationResult::invalid_with_errors(validation_errors);

        // Convert to execution result
        let execution_result = validation_result.to_execution_result().unwrap();

        // Add execution timing
        let final_result = execution_result.with_timing(150);

        // Verify the complete workflow
        assert!(final_result.has_errors());
        assert!(final_result.has_client_errors());
        assert!(!final_result.has_server_errors());
        assert_eq!(final_result.errors.len(), 2);

        let timing_ext = final_result.extensions.unwrap();
        assert_eq!(timing_ext["timing"]["duration_ms"], 150);
    }

    #[test]
    fn test_graphql_result_type_alias() {
        fn sample_operation() -> GraphQLResult<String> {
            Err(GraphQLError::validation_error("Sample error".to_string()))
        }

        match sample_operation() {
            Ok(_) => panic!("Should have failed"),
            Err(error) => {
                assert_eq!(error.error_code(), Some("VALIDATION_ERROR"));
            },
        }
    }
}
