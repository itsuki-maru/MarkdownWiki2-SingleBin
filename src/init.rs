use serde::Deserialize;
use serde::Serialize;
use dirs::home_dir;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;
use super::scheme::ApplicationInitSetup;


pub fn get_application_user_setup_path() -> PathBuf {
    let home_dir = home_dir().expect("User home directory get error.");
    let setup_file_dir = home_dir.join(".markdown-wiki2-single");
    if !setup_file_dir.exists() {
        fs::create_dir(&setup_file_dir).expect("Directory `~/.markdown-wiki2-single` create error.");
        let images_dir = &setup_file_dir.join("images");
        fs::create_dir(images_dir).expect("Directory `~/.markdown-wiki2-single/images` create error.");
        read_or_create_json_env(setup_file_dir.clone());
    }
    setup_file_dir
}

pub fn read_or_create_json_env(setup_file_dir: PathBuf) -> ApplicationInitSetup {
    let setup_recover_path = setup_file_dir.clone();
    let env_json_path = setup_file_dir.join("markdown-wiki2-single.env.json");

    // JSONデータが存在しない場合の処理
    if !env_json_path.exists() {
        let default_env = create_default_env(setup_file_dir);
        let _ = write_to_json_file(env_json_path.clone(), &default_env.clone());
    }

    let load_json_data: ApplicationInitSetup = match read_to_json_data(&env_json_path) {
        Ok(load_json_env) => load_json_env,
        Err(_) => {
            let result = fs::remove_file(env_json_path);
            match result {
                Ok(_) => read_or_create_json_env(setup_recover_path),
                Err(_) => panic!("Json env load error.")
            }
            
        },
    };
    load_json_data
}

pub fn create_default_env(
    application_user_setting_dir: PathBuf,
) -> ApplicationInitSetup {
    // ユーザー入力から取得
    let app_title = prompt("Enter application title", "MarkdownWiki2-SingleBin");
    let database_path = application_user_setting_dir.join("markdown-wiki2.sqlite");
    let database_url = format!("sqlite:{}", &database_path.to_string_lossy());
    let image_file_path = application_user_setting_dir.join("images");
    let upload_file_path = application_user_setting_dir.join("images");
    let failed_account_lock = prompt("Enter failed account lock", "15");
    let next_challenge_minutes = prompt("Enter next challenge minutes", "5");
    let challenge_limit_time_failed_count = prompt("Enter challenge limit time failed count", "5");
    let admin_username = prompt("Enter administrator name", "administrator");
    let admin_passwotd = prompt("Enter administrator password", "P@ssw0rd");
    let access_token_exp_minutes = prompt("Enter access token exp(minutes)", "60");
    let refresh_token_exp_minutes = prompt("Enter refresh token exp(minutes)", "1440");
    let secret_key = Uuid::new_v4().to_string();
    let rust_log = "markdown_wiki2_single=info,tower_http=info".to_string();
    let allow_user_create_account = "false".to_string();

    ApplicationInitSetup {
        app_title: app_title.clone(),
        sqlite_database_path: database_path,
        database_url: database_url,
        image_file_path: image_file_path.to_string_lossy().into_owned(),
        upload_file_path: upload_file_path.to_string_lossy().into_owned(),
        failed_account_lock: failed_account_lock,
        next_challenge_minutes: next_challenge_minutes,
        challenge_limit_time_failed_count: challenge_limit_time_failed_count,
        admin_username: admin_username,
        admin_passwotd: admin_passwotd,
        access_token_exp_minutes: access_token_exp_minutes,
        refresh_token_exp_minutes: refresh_token_exp_minutes,
        secret_key: secret_key,
        cache_control: "max-age=3600".to_string(),
        service_name: app_title,
        rust_log: rust_log,
        allow_user_create_account: allow_user_create_account,
        allow_origins: format!("http://localhost:5173"),
    }
}

fn write_to_json_file<T: Serialize>(file_path: PathBuf, data: &T) -> io::Result<()> {
    let file = fs::File::create(file_path).expect("`markdown-wiki2-single.env.json` fs create error.");
    serde_json::to_writer_pretty(file, data).expect("`markdown-wiki2-single.env.json` write error.");
    Ok(())
}

fn read_to_json_data<T: for<'de> Deserialize<'de>>(file_path: &PathBuf) -> io::Result<T> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

fn prompt(prompt_text: &str, default: &str) -> String {
    print!("{} [{}]: ", prompt_text, default);
    io::stdout().flush().unwrap(); // プロンプトを即時表示

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        default.to_string() // 空文字の場合はデフォルト値を返す
    } else {
        input.to_string()
    }
}