mod common;
use aion::*;
use assert_fs::{
    fixture::ChildPath,
    prelude::{FileTouch, FileWriteStr, PathChild},
};
use httpmock::{Method::GET, MockServer};
use serial_test::serial;
use std::env;

const VERIFY_TOKEN_URL: &str = "/v1/auth/token/lookup-self";
const FETCH_SECRETS_URL: &str = "/v1/secret/data";

fn setup() -> (assert_fs::TempDir, assert_fs::TempDir) {
    let wk_temp = assert_fs::TempDir::new().unwrap();
    let elixir_temp = assert_fs::TempDir::new().unwrap();

    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

    (wk_temp, elixir_temp)
}

fn teardown(wk_temp: assert_fs::TempDir, elixir_temp: assert_fs::TempDir) {
    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}

fn verify_token_mock(server: &MockServer) -> httpmock::Mock {
    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    server.mock(|when, then| {
        when.method(GET)
            .path_contains(VERIFY_TOKEN_URL)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    })
}

fn get_secret_mock<'a>(server: &'a MockServer, custom_data: Option<&'a str>) -> httpmock::Mock<'a> {
    let data = custom_data.unwrap_or(r#"
        {
          "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
          "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
          "dotenv": "test=true"
        }"#);

    let secret_api_resp = format!(
        r#"{{
          "data": {{
            "data": {},
            "metadata": {{
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {{
                "owner": "xxx",
                "mission_critical": "false"
              }},
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }}
          }}
        }}"#,
        data
    );

    server.mock(|when, then| {
        when.method(GET).path_contains(FETCH_SECRETS_URL);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    })
}

fn mock_wukong_config(wk_temp: &assert_fs::TempDir, server_url: String) -> ChildPath {
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"

                    [auth.vault]
                    api_token = "valid_vault_api_token"
                    expiry_time = "2027-06-09T08:51:19.032792+00:00"

                    [auth.okta]
                    client_id = "valid-okta-client-id"
                    account = "test@email.com"
                    subject = "subject"
                    id_token = "id_token"
                    access_token = "access_token"
                    expiry_time = "{}"
                    refresh_token = "refresh_token"
                "#,
                server_url,
                2.days().from_now().to_rfc3339()
            )
            .as_str(),
        )
        .unwrap();

    wk_config_file
}

#[test]
fn test_wukong_dev_config_help() {
    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_for_wukong_toml_success() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let verify_token_mock = verify_token_mock(&server);
    let secret_data_mock = get_secret_mock(&server, None);
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    let wukong_toml_config_file = elixir_temp.child(".wukong.toml");
    wukong_toml_config_file.touch().unwrap();
    wukong_toml_config_file
        .write_str(
            r#"
[[secrets]]

[secrets.dotenv]
provider = "bunker"
kind = "generic"
src = "vault:secret/mv/tech/app/dev#dotenv"
dst = ".env"
    "#,
        )
        .unwrap();

    let created_env_file = elixir_temp.child(".env");
    created_env_file.touch().unwrap();
    created_env_file.write_str("test=false").unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_for_dev_exs_success() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let verify_token_mock = verify_token_mock(&server);
    let secret_data_mock = get_secret_mock(&server, None);
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();
    let dev_config_secret_file_c = elixir_temp.child("config/c.secret.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
                use Mix.Config

                System.get_env("API_KEY")
                System.fetch_env("API_SECRET")
                System.fetch_env!("API_TOKEN")

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#c.secret.exs
                File.exists?("c.secret.exs") && import_config "c.secret.exs"

                test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

                # Use Jason for JSON parsing in Phoenix
                config :phoenix, :json_library, Jason
            "#,
        )
        .unwrap();

    dev_config_secret_file_c
        .write_str(
            r#"
use Mix.Config

# Configure your database.
config :academy, Academy.Repo,
  adapter: Ecto.Adapters.Postgres,
  username: "postgres",
  passwords: "",
  database: "academy_core_devss",
  pool_size: 10
            "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_when_secret_key_not_found_from_bunker() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let verify_token_mock = verify_token_mock(&server);
    let secret_data_mock = get_secret_mock(
        &server,
        Some(
            r#"
        {
          "a.secret.exs": "test"
        }"#,
        ),
    );
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();
    let dev_config_secret_file = elixir_temp.child("config/c.secret.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
                use Mix.Config

                System.get_env("API_KEY")
                System.fetch_env("API_SECRET")
                System.fetch_env!("API_TOKEN")

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#c.secret.exs
                File.exists?("c.secret.exs") && import_config "c.secret.exs"

                test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

                # Use Jason for JSON parsing in Phoenix
                config :phoenix, :json_library, Jason
            "#,
        )
        .unwrap();

    dev_config_secret_file
        .write_str(
            r#"use Mix.Config

config :application, Application.Repo"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_when_secret_file_not_found() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let verify_token_mock = verify_token_mock(&server);
    let secret_data_mock = get_secret_mock(
        &server,
        Some(
            r#"
        {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo"
            }"#,
        ),
    );
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());
    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
                use Mix.Config

                System.get_env("API_KEY")
                System.fetch_env("API_SECRET")
                System.fetch_env!("API_TOKEN")

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#c.secret.exs
                File.exists?("c.secret.exs") && import_config "c.secret.exs"

                test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

                # Use Jason for JSON parsing in Phoenix
                config :phoenix, :json_library, Jason
            "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (wk_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
        (elixir_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    });

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_when_no_changes_found() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let verify_token_mock = verify_token_mock(&server);
    let secret_data_mock = get_secret_mock(
        &server,
        Some(
            r#"
        {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo"
            }"#,
        ),
    );
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();
    let dev_config_secret_file = elixir_temp.child("config/c.secret.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
                use Mix.Config

                System.get_env("API_KEY")
                System.fetch_env("API_SECRET")
                System.fetch_env!("API_TOKEN")

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#c.secret.exs
                File.exists?("c.secret.exs") && import_config "c.secret.exs"

                test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

                # Use Jason for JSON parsing in Phoenix
                config :phoenix, :json_library, Jason
            "#,
        )
        .unwrap();

    dev_config_secret_file
        .write_str(
            r#"use Mix.Config

config :application, Application.Repo"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_diff_when_config_not_found() {
    let server = MockServer::start();
    let (wk_temp, elixir_temp) = setup();
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("diff")
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_pull_for_wukong_toml_success() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let secret_data_mock = get_secret_mock(
        &server,
        Some(
            r#"
            {
              "dotenv": "test=true"
            }"#,
        ),
    );
    let verify_token_mock = verify_token_mock(&server);
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
[core]
application = "valid-application"
wukong_api_url = "{}"

[auth.vault]
api_token = "valid_vault_api_token"
expiry_time = "2027-06-09T08:51:19.032792+00:00"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "{}"
refresh_token = "refresh_token"
    "#,
                server.base_url(),
                2.days().from_now().to_rfc3339()
            )
            .as_str(),
        )
        .unwrap();

    let wukong_toml_config_file = elixir_temp.child(".wukong.toml");
    wukong_toml_config_file.touch().unwrap();

    wukong_toml_config_file
        .write_str(
            r#"
[[secrets]]

[secrets.dotenv]
provider = "bunker"
kind = "generic"
src = "vault:secret/mv/tech/app/dev#dotenv"
dst = ".env"
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("pull")
        .arg(elixir_temp.path().to_str().unwrap())
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (elixir_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    });

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_pull_for_dev_exs_success() {
    let (wk_temp, elixir_temp) = setup();
    let server = MockServer::start();

    let secret_data_mock = get_secret_mock(
        &server,
        Some(
            r#"
        {
              "a.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5"
            }"#,
        ),
    );
    let verify_token_mock = verify_token_mock(&server);
    let wk_config_file = mock_wukong_config(&wk_temp, server.base_url());

    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
[core]
application = "valid-application"
wukong_api_url = "{}"

[auth.vault]
api_token = "valid_vault_api_token"
expiry_time = "2027-06-09T08:51:19.032792+00:00"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "{}"
refresh_token = "refresh_token"
    "#,
                server.base_url(),
                2.days().from_now().to_rfc3339()
            )
            .as_str(),
        )
        .unwrap();

    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
use Mix.Config

System.get_env("API_KEY")
System.fetch_env("API_SECRET")
System.fetch_env!("API_TOKEN")

# wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#a.secret.exs
import_config "config/a.secret.exs"

# wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#b.secret.exs
if File.exists?("config/b.secret.exs") do
  import_config "config/b.secret.exs"
end

# wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#c.secret.exs
File.exists?("config/c.secret.exs") && import_config "config/c.secret.exs"

test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("pull")
        .arg(elixir_temp.path().to_str().unwrap())
        .env("WUKONG_DEV_CONFIG_FILE", wk_config_file.path())
        .env("WUKONG_DEV_VAULT_API_URL", server.base_url())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (elixir_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    });

    verify_token_mock.assert();
    secret_data_mock.assert();

    teardown(wk_temp, elixir_temp)
}

#[test]
#[serial]
fn test_wukong_dev_config_lint_on_main_config_success() {
    let wk_temp = assert_fs::TempDir::new().unwrap();
    let main_config_file = wk_temp.child("config/config.exs");
    main_config_file.touch().unwrap();

    main_config_file
        .write_str(
            r#"
use Mix.Config

System.get_env("API_KEY")
System.fetch_env("API_SECRET")
System.fetch_env!("API_TOKEN")

# invalid
import_config "config/dev.exs"

# valid
if File.exists?("config/dev.exs") do
  import_config "config/dev.exs"
end

# invalid
if File.exists?("config/a.exs") do
  import_config "config/b.exs"
end

# valid
File.exists?("config/dev.exs") && import_config "config/dev.exs"
# invalid
File.exists?("config/a.exs") && import_config "config/b.exs"

test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("lint")
        .arg(wk_temp.path().to_str().unwrap())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (wk_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
        (r"\d+(.\d+)ms|\d+(.\d+)s", "[DURATION]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    });

    wk_temp.close().unwrap();
}

#[test]
#[serial]
fn test_wukong_dev_config_lint_on_dev_config_success() {
    let wk_temp = assert_fs::TempDir::new().unwrap();
    let dev_config_file = wk_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();

    dev_config_file
        .write_str(
            r#"
use Mix.Config

System.get_env("API_KEY")
System.fetch_env("API_SECRET")
System.fetch_env!("API_TOKEN")

# invalid
import_config "config/dev.exs"

# valid
if File.exists?("config/dev.exs") do
  import_config "config/dev.exs"
end

# invalid
if File.exists?("config/a.exs") do
  import_config "config/b.exs"
end

# valid
File.exists?("config/dev.exs") && import_config "config/dev.exs"
# invalid
File.exists?("config/a.exs") && import_config "config/b.exs"

test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("config")
        .arg("lint")
        .arg(wk_temp.path().to_str().unwrap())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (wk_temp.path().to_str().unwrap().to_string().as_str(), "[TEMP_DIR]"),
        (r"\d+(.\d+)ms|\d+(.\d+)s", "[DURATION]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    });

    wk_temp.close().unwrap();
}
