use clap::ArgMatches;
use eth2_keystore::{Keystore, PlainText};
use std::fs::read;
use std::path::PathBuf;
use sdkms::api_model::*;
use sdkms:: {Error as SdkmsError, SdkmsClient};
use rand::prelude::*;
const MY_API_KEY : &'static str= "OTA5NzMxZjAtYzliNy00NTg5LWI0MTEtYjhiZjlhZjExNmQ2OmN0NEM0bVExQjFTZUlfYlcyNVk4X3FnaURnd0JMN2lVUkROOFowUGVzX1BQN3BFSVVjX1lKZ3RJTGMwcWZtdUxLNTFSdlVMVUNKeGhCR1ZSdjN4ek13";

fn load_keystore(keystore_path: &PathBuf)-> Result<Keystore, String>{
    let keystore = Keystore::from_json_file(keystore_path);
    keystore.map_err(|e| format!("Failed to parse json {:?}", e))
}

fn import_to_dsm(key: Vec<u8>)->Result<(), SdkmsError>{
    println!{"Importing to DSM..."};
    let client = SdkmsClient::builder()
    .with_api_endpoint("https://apps.sdkms.fortanix.com")
    .with_api_key(MY_API_KEY)
    .build()?;

    let sobject_req = SobjectRequest{
        name: Some(format!("lighthouse-{}", random_name(8))),
        description: Some(format!("BLS keys imported from lighthouse")),
        obj_type: Some(ObjectType::Secret),
        key_ops: Some(
            KeyOperations::APPMANAGEABLE | KeyOperations::EXPORT,
        ),
        value: Some(key.into()),
        ..Default::default()
    };

    
    let sobject = client.import_sobject(&sobject_req)?;
    println!("Created sobject: \n{}", sobject_to_string(&sobject));
    Ok(())
}
fn sobject_to_string(s: &Sobject) -> String {
    format!(
        "key-id {}\nName {}\nCreated-at {}",
        s.kid.map_or("?".to_owned(), |kid| kid.to_string()),
        s.name.as_ref().map(String::as_str).unwrap_or_default(),
        s.created_at.to_utc_datetime().unwrap(),
    )
}

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let keystore_path :PathBuf = clap_utils::parse_required(matches, "keystore_path")?;
    let password_path :PathBuf = clap_utils::parse_required(matches, "ks_password_path")?;
    let keystore = load_keystore(&keystore_path)?;
    let password: PlainText = read(password_path)
        .map_err(|_| "Unable to parse password")?
        .into();
    println!{"Running put keystore"};
    let keypair = keystore.decrypt_keypair(password.as_ref());
    let raw_bytes:Vec<u8> = keypair.unwrap().sk.serialize().as_bytes().into();
    let _ = import_to_dsm(raw_bytes).map_err(|e| format!("Sdkms import error {} ", e));
    Ok(())
}

fn random_name(size: usize) -> String {
    let char_set = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    let mut s = String::with_capacity(size);
    let mut rng = thread_rng();
    for _ in 0..size {
        let r = rng.gen_range(0..char_set.len()-1);
        s.push_str(&char_set[r..r + 1]);
    }
    s
}
