use account_utils::validator_definitions::{
    SigningDefinition, ValidatorDefinition, ValidatorDefinitions,
};
use clap::ArgMatches;
use std::fs;
use std::path::PathBuf;
use validator_dir::Builder as ValidatorBuilder;

/// Generates validator directories with INSECURE, deterministic keypairs given the range
/// of indices, validator and secret directories.
pub fn generate_validator_dirs(
    indices: &[usize],
    validators_dir: PathBuf,
    secrets_dir: PathBuf,
) -> Result<(), String> {
    if !validators_dir.exists() {
        fs::create_dir_all(&validators_dir)
            .map_err(|e| format!("Unable to create validators dir: {:?}", e))?;
    }

    if !secrets_dir.exists() {
        fs::create_dir_all(&secrets_dir)
            .map_err(|e| format!("Unable to create secrets dir: {:?}", e))?;
    }

    for i in indices {
        println!("Validator {}", i + 1);

        ValidatorBuilder::new(validators_dir.clone())
            .password_dir(secrets_dir.clone())
            .store_withdrawal_keystore(false)
            .insecure_voting_keypair(*i)
            .map_err(|e| format!("Unable to generate keys: {:?}", e))?
            .build()
            .map_err(|e| format!("Unable to build validator: {:?}", e))?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
/// Generates web3signer directories with INSECURE, deterministic keypairs given the range
/// of indices, validator and secret directories.
pub fn generate_web3signer_dirs(
    indices: &[usize],
    // Lighthouse dir where validator_definitions.json is stored.
    validators_dir: PathBuf,
    // keys stored in web3signer
    keys_dir: PathBuf,
    // passwords for web3signer keys
    secrets_dir: PathBuf,
    url: String,
    root_certificate_path: PathBuf,
    client_identity_path: PathBuf,
    client_identity_password: String,
) -> Result<(), String> {
    if !keys_dir.exists() {
        fs::create_dir_all(&keys_dir).map_err(|e| format!("Unable to create keys dir: {:?}", e))?;
    }

    if !secrets_dir.exists() {
        fs::create_dir_all(&secrets_dir)
            .map_err(|e| format!("Unable to create secrets dir: {:?}", e))?;
    }
    eprintln!("Base dir: {:?}", &validators_dir);

    let mut definitions = ValidatorDefinitions::open_or_create(&validators_dir)
        .map_err(|e| format!("Unable to create validator definitions file: {:?}", e))?;

    for i in indices {
        println!("Validator {}", i + 1);

        let builder = ValidatorBuilder::new(keys_dir.clone())
            .password_dir(secrets_dir.clone())
            .store_withdrawal_keystore(false)
            .insecure_voting_keypair(*i)
            .map_err(|e| format!("Unable to generate keys: {:?}", e))?;

        let pubkey = builder.get_public_key().unwrap();

        let signing_definition = SigningDefinition::Web3Signer {
            url: url.clone(),
            root_certificate_path: Some(root_certificate_path.clone()),
            request_timeout_ms: None,
            client_identity_path: Some(client_identity_path.clone()),
            client_identity_password: Some(client_identity_password.clone()),
        };

        let definition = ValidatorDefinition {
            enabled: true,
            voting_public_key: pubkey,
            graffiti: None,
            suggested_fee_recipient: None,
            description: "".to_string(),
            signing_definition,
        };

        definitions.push(definition);

        builder
            .build()
            .map_err(|e| format!("Unable to build validator: {:?}", e))?;
    }

    definitions
        .save(&validators_dir)
        .map_err(|e| format!("Unable to save validator_definitions file: {:?}", e))?;
    convert_validator_dir_to_web3signer_dir(keys_dir, secrets_dir)?;
    Ok(())
}

pub fn convert_validator_dir_to_web3signer_dir(
    keys_dir: PathBuf,
    secrets_dir: PathBuf,
) -> Result<(), String> {
    let keydir_paths =
        fs::read_dir(&keys_dir).map_err(|e| format!("Unable to read keys directory: {:?}", e))?;
    for path in keydir_paths {
        let key_path = path
            .map_err(|e| format!("Unable to read directory: {:?}", e))?
            .path();
        let name = key_path
            .file_name()
            .ok_or_else(|| "Unable to parse file name".to_string())?;
        let mut new_path = keys_dir.join(name);
        let _ = &new_path.set_extension("json");
        fs::rename(key_path.join("voting-keystore.json"), new_path)
            .map_err(|e| format!("Unable to rename key {:?}", e))?;
        fs::remove_dir(key_path).map_err(|e| format!("Unable to remove old path {:?}", e))?;
    }

    let secretsdir_paths = fs::read_dir(&secrets_dir)
        .map_err(|e| format!("Unable to read keys directory: {:?}", e))?;
    for path in secretsdir_paths {
        let secret_path = path
            .map_err(|e| format!("Unable to read directory: {:?}", e))?
            .path();
        let mut new_path = secret_path.clone();
        let _ = new_path.set_extension("txt");
        fs::rename(secret_path, new_path)
            .map_err(|e| format!("Unable to rename secret {:?}", e))?;
    }
    Ok(())
}

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let validator_count: usize = clap_utils::parse_required(matches, "count")?;
    let base_dir: PathBuf = clap_utils::parse_required(matches, "base-dir")?;
    let node_count: Option<usize> = clap_utils::parse_optional(matches, "node-count")?;
    let web3signer_dir: PathBuf = clap_utils::parse_required(matches, "web3signer-dir")?;
    let url: String = clap_utils::parse_required(matches, "web3signer-url")?;
    let root_certificate_path: PathBuf =
        clap_utils::parse_required(matches, "root-certificate-path")?;
    let client_identity_path: PathBuf =
        clap_utils::parse_required(matches, "client-identity-path")?;
    let client_identity_password: String =
        clap_utils::parse_required(matches, "client-identity-password")?;

    if let Some(node_count) = node_count {
        let validators_per_node = validator_count / node_count;
        let validator_range = (0..validator_count).collect::<Vec<_>>();
        let indices_range = validator_range
            .chunks(validators_per_node)
            .collect::<Vec<_>>();

        for (i, indices) in indices_range.iter().enumerate() {
            let validators_dir = base_dir.join(format!("node_{}", i + 1)).join("validators");
            let keys_dir = web3signer_dir.join("keys");
            let secrets_dir = web3signer_dir.join("secrets");
            if i == 0 {
                generate_web3signer_dirs(
                    indices,
                    validators_dir,
                    keys_dir,
                    secrets_dir,
                    url.clone(),
                    root_certificate_path.clone(),
                    client_identity_path.clone(),
                    client_identity_password.clone(),
                )?;
            } else {
                let default_secrets_dir = base_dir.join(format!("node_{}", i + 1)).join("secrets");
                generate_validator_dirs(indices, validators_dir, default_secrets_dir)?;
            }
        }
    } else {
        let validators_dir = base_dir.join("validators");
        let keys_dir = web3signer_dir.join("keys");
        let secrets_dir = web3signer_dir.join("secrets");
        generate_web3signer_dirs(
            (0..validator_count).collect::<Vec<_>>().as_slice(),
            validators_dir,
            keys_dir,
            secrets_dir,
            url,
            root_certificate_path,
            client_identity_path,
            client_identity_password,
        )?;
    }
    Ok(())
}
