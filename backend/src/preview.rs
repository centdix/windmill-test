
use serde_json::Value;
use jsonschema::{JSONSchema, Draft};

// Placeholder for Error type, assuming it's defined elsewhere (e.g., windmill_common::error::Error)
// For this example, let's define a simple local Error that can wrap ValidationError
// In a real project, these error types and the validation function should be in a shared module.
#[derive(Debug)]
pub enum ServiceError {
    Validation(ValidationError),
    Internal(String),
    // ... other error variants
}

impl From<ValidationError> for ServiceError {
    fn from(err: ValidationError) -> Self {
        ServiceError::Validation(err)
    }
}

// Custom error type for validation failures
#[derive(Debug)]
pub enum ValidationError {
    MissingRequiredArgument(String),
    // InvalidType(String, String), // field_name, expected_type - Handled by InvalidArgument
    SchemaError(String),
    InvalidArgument(String), // General validation error from jsonschema or custom logic
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequiredArgument(field) => write!(f, "Missing required argument: {}", field),
            // ValidationError::InvalidType(field, expected) => write!(f, "Argument '{}' has invalid type, expected {}", field, expected),
            ValidationError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            ValidationError::InvalidArgument(msg) => write!(f, "Invalid argument(s): {}", msg),
        }
    }
}


// The core validation logic
// This function should ideally be in a shared module (e.g., windmill_common::validation)
fn validate_script_arguments(
    schema_val: &Value,
    args: &Value,
) -> Result<(), ValidationError> {
    // Handle cases where args might be `null` but schema expects an object.
    // If schema expects an object and args is not an object:
    if schema_val.get("type").and_then(Value::as_str) == Some("object") && !args.is_object() {
        // Check if it's an empty schema (no properties, no required fields) and args is null.
        // In this specific case, `null` can be treated as a valid empty object.
        let is_schema_effectively_empty = schema_val.get("properties").and_then(Value::as_object).map_or(true, |p| p.is_empty()) &&
                                          schema_val.get("required").map_or(true, |r| r.as_array().map_or(true, |ra| ra.is_empty()));
        if args.is_null() && is_schema_effectively_empty {
            // Null args for an empty object schema is acceptable.
        } else {
            // Otherwise, if schema expects an object and args is not one (and not the special null case), it's an error.
            return Err(ValidationError::InvalidArgument(
                format!("Arguments should be a JSON object, but received: {}", args.to_string())
            ));
        }
    }

    // 1. Custom check for required fields.
    // This specifically addresses the issue where "default": null might bypass standard required checks.
    if let Some(schema_obj) = schema_val.as_object() {
        if let Some(required_arr_val) = schema_obj.get("required") {
            if let Some(required_arr) = required_arr_val.as_array() {
                // This check assumes `args` is an object if there are required fields.
                // The above check handles if `args` is not an object when schema type is "object".
                let input_args_obj = match args.as_object() {
                    Some(obj) => obj,
                    None => {
                        // If args is not an object but there are required fields, it's an error.
                        // This path should ideally be caught by the initial object check if schema type is "object".
                        // If schema type isn't "object" but has "required", that's an unusual schema.
                        if !required_arr.is_empty() {
                             return Err(ValidationError::InvalidArgument(
                                format!("Arguments should be a JSON object to satisfy required fields: {:?}, but received: {}", required_arr, args.to_string())
                            ));
                        }
                        // If no required fields, proceed (jsonschema will validate type later)
                        return Ok(()); // Or let jsonschema handle it if args is not an object but schema allows other types.
                    }
                };

                for required_field_val in required_arr {
                    if let Some(required_field_str) = required_field_val.as_str() {
                        if !input_args_obj.contains_key(required_field_str) {
                            return Err(ValidationError::MissingRequiredArgument(
                                required_field_str.to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    // 2. Perform standard JSON schema validation for types, formats, etc.
    // Compile the schema. Using Draft7 as a common default.
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7) // Ensure this matches the draft used by your schemas
        .compile(schema_val)
        .map_err(|e| ValidationError::SchemaError(format!("Failed to compile schema: {}", e)))?;

    let validation_result = compiled_schema.validate(args);
    if let Err(errors) = validation_result {
        let error_messages: Vec<String> = errors
            .map(|e| {
                // Make error messages more user-friendly
                let path = e.instance_path.to_string();
                let specific_error = match &e.kind {
                    jsonschema::error::ValidationErrorKind::Required { property } => {
                        // This should ideally be caught by our manual check above,
                        // but jsonschema might report it for nested objects.
                        format!("property '{}' is required", property.as_str().unwrap_or_default())
                    }
                    jsonschema::error::ValidationErrorKind::Type { expected } => {
                        format!("has incorrect type, expected {:?}", expected)
                    }
                    // Add more specific messages for other error kinds if needed
                    _ => e.to_string(), // Fallback to default jsonschema error string
                };
                if path.is_empty() { // Error on the root instance
                    format!("Input data {}", specific_error)
                } else {
                    format!("Field '{}' {}", path, specific_error)
                }
            })
            .collect();
        return Err(ValidationError::InvalidArgument(error_messages.join("; ")));
    }

    Ok(())
}

// Hypothetical function for running a script preview (e.g., from UI "Test" button)
// This function's signature and body are illustrative.
// It needs access to a way to fetch script schema, e.g., from a database or cache.
async fn get_script_schema_from_path(_script_path: &str) -> Result<Value, ServiceError> {
    // In a real scenario, this would fetch the script's JSON schema.
    // For this example, let's return a placeholder schema.
    Ok(serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer", "minimum": 0 }
        },
        "required": ["name"]
    }))
}

pub async fn run_script_preview_with_validation(
    script_path: String,
    script_args: Value,
) -> Result<Value, ServiceError> {
    // 1. Fetch the script's JSON schema
    let schema = get_script_schema_from_path(&script_path).await?;

    // 2. Validate arguments against the schema
    validate_script_arguments(&schema, &script_args)?;

    // 3. If validation passes, proceed to execute the script preview
    // The actual execution logic is omitted here.
    // log::info!("Arguments validated successfully for script {}. Proceeding to execution.", script_path);
    // let result = execute_actual_preview(&script_path, script_args).await?;

    Ok(serde_json::json!({
        "message": "Script arguments validated successfully. Preview would run.",
        "validated_args": script_args
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "email": { "type": "string", "format": "email" },
                "age": { "type": "integer", "default": 30 },
                "address": {
                    "type": "object",
                    "properties": {
                        "street": { "type": "string" },
                        "city": { "type": "string" }
                    },
                    "required": ["street", "city"]
                }
            },
            "required": ["name", "email"]
        })
    }

    #[test]
    fn valid_args() {
        let schema = test_schema();
        let args = json!({"name": "John Doe", "email": "john.doe@example.com", "age": 35});
        assert!(validate_script_arguments(&schema, &args).is_ok());
    }

    #[test]
    fn missing_required_arg() {
        let schema = test_schema();
        let args = json!({"name": "John Doe"}); // "email" is missing
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::MissingRequiredArgument(field)) => assert_eq!(field, "email"),
            res => panic!("Expected MissingRequiredArgument for 'email', got {:?}", res),
        }
    }

    #[test]
    fn invalid_type_for_arg() {
        let schema = test_schema();
        let args = json!({"name": "John Doe", "email": "john.doe@example.com", "age": "thirty-five"});
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::InvalidArgument(msg)) => {
                assert!(msg.contains("Field '/age' has incorrect type, expected Integer"));
            }
            res => panic!("Expected InvalidArgument for type mismatch on 'age', got {:?}", res),
        }
    }
    
    #[test]
    fn invalid_format_for_arg() {
        let schema = test_schema();
        let args = json!({"name": "John Doe", "email": "not-an-email"});
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::InvalidArgument(msg)) => {
                assert!(msg.contains("Field '/email' \"not-an-email\" is not a \"email\""));
            }
            res => panic!("Expected InvalidArgument for format mismatch on 'email', got {:?}", res),
        }
    }

    #[test]
    fn empty_args_when_required_present() {
        let schema = test_schema();
        let args = json!({});
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::MissingRequiredArgument(field)) => assert!(field == "name" || field == "email"),
            res => panic!("Expected MissingRequiredArgument, got {:?}", res),
        }
    }

    #[test]
    fn null_args_when_object_expected_with_required() {
        let schema = test_schema(); // Expects an object with required fields
        let args = serde_json::Value::Null;
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::InvalidArgument(msg)) => {
                 assert!(msg.contains("Arguments should be a JSON object, but received: null"));
            }
            res => panic!("Expected InvalidArgument for null args, got {:?}", res),
        }
    }

    #[test]
    fn valid_null_args_for_empty_object_schema() {
        let schema = json!({"type": "object", "properties": {}, "required": []});
        let args = serde_json::Value::Null;
        assert!(validate_script_arguments(&schema, &args).is_ok());
    }
    
    #[test]
    fn valid_empty_object_args_for_empty_object_schema() {
        let schema = json!({"type": "object", "properties": {}, "required": []});
        let args = json!({});
        assert!(validate_script_arguments(&schema, &args).is_ok());
    }

    #[test]
    fn nested_required_field_missing() {
        let schema = test_schema();
        let args = json!({
            "name": "Jane Doe",
            "email": "jane.doe@example.com",
            "address": { "street": "123 Main St" } // "city" is missing
        });
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::InvalidArgument(msg)) => {
                assert!(msg.contains("Field '/address' property 'city' is required"));
            }
            res => panic!("Expected InvalidArgument for missing nested 'city', got {:?}", res),
        }
    }
}
