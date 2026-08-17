//! XSD validation tests for 3MF files produced by threemf2
//!
//! This crate validates that XML output from threemf2 conforms to the official
//! 3MF Consortium XSD schemas.

// Validation utilities for XSD testing
pub mod validation {
    use fastxml::schema::SchemaBuilder;
    use fastxml::schema::Validator;
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    /// Extract model XML from a 3MF package (ZIP archive)
    pub fn extract_model_xml(package_bytes: &[u8]) -> Result<String, String> {
        let cursor = Cursor::new(package_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| format!("Failed to open ZIP archive: {}", e))?;

        // Find and extract the 3D model file
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

            let name = file.name();
            if name.ends_with(".model") || name.contains("3dmodel") {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read model content: {}", e))?;
                return Ok(content);
            }
        }

        Err("No .model file found in 3MF package".to_string())
    }

    pub fn validate_against_xsd(xml: &str, contents: &[(&str, &[u8])]) -> Result<(), String> {
        let mut builder = SchemaBuilder::new();
        for (uri, content) in contents {
            builder = builder.add(*uri, *content);
        }
        let schema = builder
            .resolve()
            .map_err(|e| format!("Schema compilation failed: {:?}", e))?;

        let report = Validator::from(xml)
            .schema(schema)
            .run()
            .map_err(|e| format!("Validation execution failed: {:?}", e))?;

        if report.is_valid() {
            Ok(())
        } else {
            let errors: Vec<String> = report.errors().iter().map(|e| e.to_string()).collect();
            Err(errors.join("\n"))
        }
    }

    /// Panic with detailed error message on validation failure
    pub fn validate_or_panic(xml: &str, contents: &[(&str, &[u8])], case_name: &str) {
        match validate_against_xsd(xml, contents) {
            Ok(()) => (),
            Err(e) => {
                panic!(
                    "XSD validation failed for {}:\n{}\n\nXML Content:\n{}",
                    case_name, e, xml
                );
            }
        }
    }
}
