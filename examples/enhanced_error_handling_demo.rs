/// Example demonstrating enhanced GraphQL error handling capabilities
/// This example shows various error types, propagation, formatting, and best practices
use graphql_rs::domain::value_objects::{
    ErrorFormatter, ErrorPropagation, ExecutionResult, GraphQLError, PathSegment, ValidationResult,
};
use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 GraphQL Enhanced Error Handling Demo");
    println!("==========================================");

    // 1. Demonstrate different error types
    demo_error_types();

    // 2. Demonstrate error enhancement
    demo_error_enhancement();

    // 3. Demonstrate error propagation
    demo_error_propagation();

    // 4. Demonstrate error formatting
    demo_error_formatting();

    // 5. Demonstrate execution results
    demo_execution_results();

    // 6. Demonstrate validation results
    demo_validation_results();

    // 7. Real-world scenario
    demo_real_world_scenario().await;

    println!("\n✅ Enhanced Error Handling Demo Complete!");
    Ok(())
}

fn demo_error_types() {
    println!("\n🔍 1. Different Error Types:");
    println!("   ----------------------------");

    // Client errors
    let validation_error = GraphQLError::validation_error("Invalid query syntax".to_string());
    let field_error = GraphQLError::field_not_found("unknownField", "Query");
    let auth_error = GraphQLError::auth_error("Invalid authentication token".to_string());

    // Server errors
    let execution_error = GraphQLError::execution_error("Database connection failed".to_string());
    let rate_limit_error = GraphQLError::rate_limit_error();

    println!(
        "   ✓ Validation Error: {} (Code: {})",
        validation_error.message,
        validation_error.error_code().unwrap_or("None")
    );
    println!(
        "   ✓ Field Error: {} (Client: {})",
        field_error.message,
        field_error.is_client_error()
    );
    println!(
        "   ✓ Auth Error: {} (Code: {})",
        auth_error.message,
        auth_error.error_code().unwrap_or("None")
    );
    println!(
        "   ✓ Execution Error: {} (Server: {})",
        execution_error.message,
        execution_error.is_server_error()
    );
    println!(
        "   ✓ Rate Limit Error: {} (Code: {})",
        rate_limit_error.message,
        rate_limit_error.error_code().unwrap_or("None")
    );
}

fn demo_error_enhancement() {
    println!("\n🔧 2. Error Enhancement:");
    println!("   ----------------------");

    let error = GraphQLError::field_not_found("name", "User")
        .with_location(2, 15)
        .with_path(vec![
            PathSegment::Field("user".to_string()),
            PathSegment::Index(0),
            PathSegment::Field("name".to_string()),
        ])
        .with_timestamp()
        .with_request_id("req-12345")
        .with_extension("suggestion", json!("Did you mean 'username'?"));

    println!(
        "   ✓ Error with location: Line {}, Column {}",
        error.locations[0].line, error.locations[0].column
    );
    println!("   ✓ Error with path: {:?}", error.path);
    println!("   ✓ Error with extensions: request_id, timestamp, suggestion");

    // Demonstrate user message masking
    let server_error =
        GraphQLError::execution_error("Database password leaked: secret123".to_string());
    println!(
        "   ✓ Server error (dev): {}",
        server_error.user_message(false)
    );
    println!(
        "   ✓ Server error (prod): {}",
        server_error.user_message(true)
    );
}

fn demo_error_propagation() {
    println!("\n📡 3. Error Propagation:");
    println!("   -----------------------");

    // Non-nullable field error (bubbles up)
    let error = GraphQLError::execution_error("Service unavailable".to_string());
    let path = vec![
        PathSegment::Field("user".to_string()),
        PathSegment::Field("id".to_string()),
    ];

    let (_propagated_error, should_bubble) = ErrorPropagation::propagate_field_error(
        error.clone(),
        &path,
        false, // non-nullable
    );

    println!(
        "   ✓ Non-nullable field error bubbles up: {}",
        should_bubble
    );

    // Nullable field error (doesn't bubble)
    let (_, should_bubble_nullable) = ErrorPropagation::propagate_field_error(
        error, &path, true, // nullable
    );

    println!(
        "   ✓ Nullable field error bubbles up: {}",
        should_bubble_nullable
    );

    // Missing required field
    let missing_error = ErrorPropagation::missing_required_field(
        "email",
        &[PathSegment::Field("user".to_string())],
    );
    println!("   ✓ Missing required field: {}", missing_error.message);

    // Null in non-nullable
    let null_error =
        ErrorPropagation::null_in_non_nullable("id", &[PathSegment::Field("user".to_string())]);
    println!("   ✓ Null in non-nullable: {}", null_error.message);
}

fn demo_error_formatting() {
    println!("\n🎨 4. Error Formatting:");
    println!("   ----------------------");

    let error = GraphQLError::execution_error("Internal service failure".to_string())
        .with_location(3, 7)
        .with_path(vec![
            PathSegment::Field("users".to_string()),
            PathSegment::Index(1),
        ])
        .with_extension("stackTrace", json!(["service.rs:42", "handler.rs:15"]));

    // Development formatting (full details)
    let _dev_format = ErrorFormatter::format_development(&error);
    println!("   ✓ Development format includes: message, locations, path, stackTrace");

    // Production formatting (sanitized)
    let _prod_format = ErrorFormatter::format_production(&error);
    println!("   ✓ Production format masks sensitive info");

    // Error summary
    let errors = vec![
        GraphQLError::validation_error("Error 1".to_string()),
        GraphQLError::field_not_found("field", "Type"),
        GraphQLError::execution_error("Error 2".to_string()),
    ];

    let summary = ErrorFormatter::format_error_summary(&errors);
    println!(
        "   ✓ Error summary: {} total, {} client, {} server",
        summary["total"], summary["client_errors"], summary["server_errors"]
    );
}

fn demo_execution_results() {
    println!("\n⚡ 5. Execution Results:");
    println!("   ----------------------");

    // Success result
    let success =
        ExecutionResult::success(json!({"user": {"name": "John", "email": "john@example.com"}}))
            .with_timing(125)
            .with_tracing("trace-abc123");

    println!(
        "   ✓ Success result: is_success={}, has_errors={}",
        success.is_success(),
        success.has_errors()
    );

    // Error result
    let error_result =
        ExecutionResult::single_error(GraphQLError::field_not_found("unknownField", "Query"));

    println!(
        "   ✓ Error result: is_success={}, has_errors={}",
        error_result.is_success(),
        error_result.has_errors()
    );

    // Partial result (data + errors)
    let partial = ExecutionResult::partial(
        json!({"user": {"name": "John", "email": null}}),
        vec![GraphQLError::execution_error(
            "Email service unavailable".to_string(),
        )],
    );

    println!(
        "   ✓ Partial result: has_data={}, has_errors={}",
        partial.data.is_some(),
        partial.has_errors()
    );

    // Result with mixed error types
    let mixed = ExecutionResult::success(json!({}))
        .with_error(GraphQLError::validation_error("Client error".to_string()))
        .with_error(GraphQLError::execution_error("Server error".to_string()));

    println!(
        "   ✓ Mixed errors: client_errors={}, server_errors={}",
        mixed.has_client_errors(),
        mixed.has_server_errors()
    );

    // Sanitized result for production
    let _sanitized = mixed.sanitized(true);
    println!("   ✓ Sanitized result masks server error messages");
}

fn demo_validation_results() {
    println!("\n✅ 6. Validation Results:");
    println!("   -------------------------");

    // Valid result
    let valid = ValidationResult::valid();
    println!("   ✓ Valid result: {}", valid.is_valid());

    // Invalid with field error
    let field_invalid = ValidationResult::field_error("name", "Field is required".to_string());
    println!(
        "   ✓ Field error has path: {}",
        field_invalid.errors().unwrap()[0].path.is_some()
    );

    // Type error
    let type_invalid = ValidationResult::type_error("User", "Invalid type definition".to_string());
    println!(
        "   ✓ Type error has type extension: {}",
        type_invalid.errors().unwrap()[0].extensions.is_some()
    );

    // Combine multiple results
    let combined = ValidationResult::combine(vec![
        ValidationResult::valid(),
        ValidationResult::invalid("Error 1".to_string()),
        ValidationResult::field_error("field", "Error 2".to_string()),
    ]);

    if let Some(errors) = combined.errors() {
        println!("   ✓ Combined result has {} errors", errors.len());
    }

    // Convert to execution result
    if let Some(exec_result) = combined.to_execution_result() {
        println!(
            "   ✓ Converted to execution result with {} errors",
            exec_result.errors.len()
        );
    }
}

async fn demo_real_world_scenario() {
    println!("\n🌍 7. Real-World Scenario:");
    println!("   --------------------------");

    // Simulate a GraphQL query execution with various errors
    let mut result = ExecutionResult::success(json!({}));

    // Add some validation errors during parsing
    result = result.with_error(
        GraphQLError::validation_error("Unknown directive '@unknownDirective'".to_string())
            .with_location(1, 20),
    );

    // Add a field resolution error
    result = result.with_error(
        GraphQLError::field_not_found("profile", "User")
            .with_location(2, 8)
            .with_path(vec![PathSegment::Field("user".to_string())])
            .with_extension("suggestion", json!("Did you mean 'userProfile'?")),
    );

    // Add an execution error from a resolver
    result = result.with_error(
        GraphQLError::execution_error("External API timeout".to_string())
            .with_path(vec![
                PathSegment::Field("user".to_string()),
                PathSegment::Field("posts".to_string()),
                PathSegment::Index(0),
            ])
            .with_timestamp()
            .with_request_id("req-xyz789"),
    );

    // Add execution metadata
    result = result.with_timing(450).with_tracing("trace-real-world-123");

    println!("   ✓ Complex query result:");
    println!("     - Total errors: {}", result.errors.len());
    println!("     - Client errors: {}", result.has_client_errors());
    println!("     - Server errors: {}", result.has_server_errors());
    println!("     - Has timing info: {}", result.extensions.is_some());

    // Show different formatting approaches
    println!("\n   📊 Error Analysis:");
    let summary = ErrorFormatter::format_error_summary(&result.errors);
    println!(
        "     - Validation errors: {}",
        summary["by_code"]
            .get("VALIDATION_ERROR")
            .unwrap_or(&json!(0))
    );
    println!(
        "     - Field errors: {}",
        summary["by_code"]
            .get("FIELD_NOT_FOUND")
            .unwrap_or(&json!(0))
    );
    println!(
        "     - Execution errors: {}",
        summary["by_code"]
            .get("EXECUTION_ERROR")
            .unwrap_or(&json!(0))
    );

    // Show sanitized version for production
    let _sanitized = result.sanitized(true);
    println!("\n   🔒 Production Response (sanitized):");
    println!("     - Client errors shown as-is");
    println!("     - Server errors masked for security");
    println!("     - Timing and tracing info preserved");
}
