use serde_json::Value;
use serde_json::Value as JsonValue;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

pub fn split_slug(slug: &str) -> Option<(String, String)> {
    // Attempt to split the slug into two parts based on the '/'
    match slug.split_once('/') {
        Some((org_id, name)) => Some((org_id.to_string(), name.to_string())),
        None => None, // Return None if the slug does not contain a '/'
    }
}

pub fn get_token_from_file_storage() -> String {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            println!("Failed to locate home directory. Exiting.");
            exit(1);
        }
    };

    // Construct the path to the auth.json file
    let auth_file_path: PathBuf = [home_dir.to_str().unwrap(), ".ginger-society", "auth.json"]
        .iter()
        .collect();

    // Read the token from the file
    let mut file = match File::open(&auth_file_path) {
        Ok(f) => f,
        Err(_) => {
            println!("Failed to open {}. Exiting.", auth_file_path.display());
            exit(1);
        }
    };
    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents) {
        println!("Failed to read the auth.json file. Exiting.");
        exit(1);
    }

    let json: Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => {
            println!("Failed to parse auth.json as JSON. Exiting.");
            exit(1);
        }
    };

    let token = match json.get("API_TOKEN").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            println!("API_TOKEN not found in auth.json. Exiting.");
            exit(1);
        }
    };

    token
}

pub fn get_package_json_info() -> Option<(String, String, String, String, Vec<String>)> {
    let mut file = File::open("package.json").expect("Failed to open package.json");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read package.json");

    let package_json: JsonValue =
        serde_json::from_str(&content).expect("Failed to parse package.json");

    let name = package_json.get("name")?.as_str()?.to_string();
    let version = package_json.get("version")?.as_str()?.to_string();
    let description = package_json.get("description")?.as_str()?.to_string();

    // Extract organization and package name
    let (organization, package_name) = if name.starts_with('@') {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() == 2 {
            (
                parts[0].trim_start_matches('@').to_string(),
                parts[1].to_string(),
            )
        } else {
            println!("The package name should be of format @scope/pkg-name");
            exit(1);
        }
    } else {
        println!("The package name should be of format @scope/pkg-name");
        exit(1);
    };

    // Internal dependencies logic
    let prefix = format!("@{}/", organization);
    let mut internal_dependencies = Vec::new();

    if let Some(dependencies) = package_json.get("dependencies").and_then(|d| d.as_object()) {
        for (key, _) in dependencies {
            if key.starts_with(&prefix) {
                internal_dependencies.push(key.clone());
            }
        }
    }

    if let Some(dev_dependencies) = package_json
        .get("devDependencies")
        .and_then(|d| d.as_object())
    {
        for (key, _) in dev_dependencies {
            if key.starts_with(&prefix) {
                internal_dependencies.push(key.clone());
            }
        }
    }

    Some((
        package_name,
        version,
        description,
        organization,
        internal_dependencies,
    ))
}
