
use serde_json::Value;
use jsonschema::{JSONSchema, Draft};

#[derive(Debug)]
pub enum ServiceError {
    Validation(ValidationError),
    Internal(String),
}

impl From<ValidationError> for ServiceError {
    fn from(err: ValidationError) -> Self {
        ServiceError::Validation(err)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    MissingRequiredArgument(String),
    SchemaError(String),
    InvalidArgument(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequiredArgument(field) => write!(f, "Missing required argument: {}", field),
            ValidationError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            ValidationError::InvalidArgument(msg) => write!(f, "Invalid argument(s): {}", msg),
        }
    }
}

fn validate_script_arguments(
    schema_val: &Value,
    args: &Value,
) -> Result<(), ValidationError> {
    if schema_val.get("type").and_then(Value::as_str) == Some("object") && !args.is_object() {
        let is_schema_effectively_empty = schema_val.get("properties").and_then(Value::as_object).map_or(true, |p| p.is_empty()) &&
                                          schema_val.get("required").map_or(true, |r| r.as_array().map_or(true, |ra| ra.is_empty()));
        if args.is_null() && is_schema_effectively_empty {
            // Null args for an empty object schema is acceptable.
        } else {
            return Err(ValidationError::InvalidArgument(
                format!("Arguments should be a JSON object, but received: {}", args.to_string())
            ));
        }
    }

    if let Some(schema_obj) = schema_val.as_object() {
        if let Some(required_arr_val) = schema_obj.get("required") {
            if let Some(required_arr) = required_arr_val.as_array() {
                let input_args_obj = match args.as_object() {
                    Some(obj) => obj,
                    None => {
                        if !required_arr.is_empty() {
                             return Err(ValidationError::InvalidArgument(
                                format!("Arguments should be a JSON object to satisfy required fields: {:?}, but received: {}", required_arr, args.to_string())
                            ));
                        }
                        return Ok(());
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

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(schema_val)
        .map_err(|e| ValidationError::SchemaError(format!("Failed to compile schema: {}", e)))?;
    let validation_result = compiled_schema.validate(args);
    if let Err(errors) = validation_result {
        let error_messages: Vec<String> = errors
            .map(|e| {
                let path = e.instance_path.to_string();
                let specific_error = match &e.kind {
                    jsonschema::error::ValidationErrorKind::Required { property } => {
                        format!("property '{}' is required", property.as_str().unwrap_or_default())
                    }
                    jsonschema::error::ValidationErrorKind::Type { expected } => {
                        format!("has incorrect type, expected {:?}", expected)
                    }
                    _ => e.to_string(),
                };
                if path.is_empty() {
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
// --- End of duplicated definitions ---


// Placeholder for JobDetails and JobOutcome structures
#[derive(Debug, Clone)]
pub struct JobDetails {
    pub job_id: String,
    pub script_path: String, // Or however the script is identified
    pub arguments: Value,    // The arguments for the script
                             // ... other job-related fields
}

pub struct JobOutcome {
    pub job_id: String,
    pub result: Value,
    // ... other outcome fields
}

// Hypothetical function for fetching script schema
async fn get_script_schema_for_job(_script_path: &str) -> Result<Value, ServiceError> {
    // Fetch schema, e.g., from DB or cache
    Ok(serde_json::json!({ // Example schema
        "type": "object",
        "properties": { "data": { "type": "string" } },
        "required": ["data"]
    }))
}

// Hypothetical function for executing a job (e.g., from `wmill script run` API)
pub async fn execute_job_with_validation(
    job_details: JobDetails,
) -> Result<JobOutcome, ServiceError> {
    // 1. Fetch the script's JSON schema based on job_details
    let schema = get_script_schema_for_job(&job_details.script_path).await?;

    // 2. Validate arguments against the schema
    validate_script_arguments(&schema, &job_details.arguments)?;

    // 3. If validation passes, proceed to actual job execution
    // log::info!("Arguments validated successfully for job {}. Proceeding to execution.", job_details.job_id);
    // let outcome = execute_actual_job(job_details).await?; // Actual execution logic

    Ok(JobOutcome {
        job_id: job_details.job_id.clone(),
        result: serde_json::json!({
            "message": "Job arguments validated successfully. Job would execute.",
            "validated_args": job_details.arguments
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn job_test_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string" },
                "payload": { "type": "object" }
            },
            "required": ["task_id"]
        })
    }

    #[test]
    fn job_valid_args() {
        let schema = job_test_schema();
        let args = json!({ "task_id": "task123", "payload": { "data": "example" } });
        assert!(validate_script_arguments(&schema, &args).is_ok());
    }

    #[test]
    fn job_missing_required_arg() {
        let schema = job_test_schema();
        let args = json!({ "payload": { "data": "example" } }); // "task_id" is missing
        match validate_script_arguments(&schema, &args) {
            Err(ValidationError::MissingRequiredArgument(field)) => assert_eq!(field, "task_id"),
            res => panic!("Expected MissingRequiredArgument for 'task_id', got {:?}", res),
        }
    }
}
