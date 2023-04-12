mod common;

use std::env;

use aion::*;
use assert_fs::prelude::{FileTouch, FileWriteStr, PathChild};
use httpmock::{Method::GET, MockServer};
use wukong::services::vault::client::VaultClient;

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
fn test_wukong_dev_config_diff_success() {
    let server = MockServer::start();

    let secret_api_resp = r#"
        {
          "data": {
            "data": {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5"
            },
            "metadata": {
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {
                "owner": "xxx",
                "mission_critical": "false"
              },
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }
          }
        }"#;

    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    let verify_token_mock = server.mock(|when, then| {
        when.method(GET)
            .path_contains(VaultClient::VERIFY_TOKEN)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    });

    let secret_data_mock = server.mock(|when, then| {
        when.method(GET).path_contains(VaultClient::FETCH_SECRETS);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    });

    let wk_temp = assert_fs::TempDir::new().unwrap();
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"
                    okta_client_id = "valid-okta-client-id"

                    [vault]
                    api_token = "valid_vault_api_token"

                    [auth]
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

    let elixir_temp = assert_fs::TempDir::new().unwrap();
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

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#a.secret.exs
                import_config "config/a.secret.exs"

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#b.secret.exs
                if File.exists?("config/b.secret.exs") do
                  import_config "config/b.secret.exs"
                end

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

    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

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

    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}

#[test]
fn test_wukong_dev_config_diff_when_secret_key_not_found_from_bunker() {
    let server = MockServer::start();

    let secret_api_resp = r#"
        {
          "data": {
            "data": {
              "a.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo"
            },
            "metadata": {
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {
                "owner": "xxx",
                "mission_critical": "false"
              },
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }
          }
        }"#;

    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    let verify_token_mock = server.mock(|when, then| {
        when.method(GET)
            .path_contains(VaultClient::VERIFY_TOKEN)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    });

    let secret_data_mock = server.mock(|when, then| {
        when.method(GET).path_contains(VaultClient::FETCH_SECRETS);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    });

    let wk_temp = assert_fs::TempDir::new().unwrap();
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"
                    okta_client_id = "valid-okta-client-id"

                    [vault]
                    api_token = "valid_vault_api_token"

                    [auth]
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

    let elixir_temp = assert_fs::TempDir::new().unwrap();
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

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#a.secret.exs
                import_config "config/a.secret.exs"

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#b.secret.exs
                if File.exists?("config/b.secret.exs") do
                  import_config "config/b.secret.exs"
                end

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

    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

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

    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}

#[test]
fn test_wukong_dev_config_diff_when_secret_file_not_found() {
    let server = MockServer::start();

    let secret_api_resp = r#"
        {
          "data": {
            "data": {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo"
            },
            "metadata": {
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {
                "owner": "xxx",
                "mission_critical": "false"
              },
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }
          }
        }"#;

    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    let verify_token_mock = server.mock(|when, then| {
        when.method(GET)
            .path_contains(VaultClient::VERIFY_TOKEN)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    });

    let secret_data_mock = server.mock(|when, then| {
        when.method(GET).path_contains(VaultClient::FETCH_SECRETS);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    });

    let wk_temp = assert_fs::TempDir::new().unwrap();
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"
                    okta_client_id = "valid-okta-client-id"

                    [vault]
                    api_token = "valid_vault_api_token"

                    [auth]
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

    let elixir_temp = assert_fs::TempDir::new().unwrap();
    let dev_config_file = elixir_temp.child("config/dev.exs");
    dev_config_file.touch().unwrap();

    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

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

    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}

#[test]
fn test_wukong_dev_config_diff_when_no_changes_found() {
    let server = MockServer::start();

    let secret_api_resp = r#"
        {
          "data": {
            "data": {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo"
            },
            "metadata": {
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {
                "owner": "xxx",
                "mission_critical": "false"
              },
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }
          }
        }"#;

    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    let verify_token_mock = server.mock(|when, then| {
        when.method(GET)
            .path_contains(VaultClient::VERIFY_TOKEN)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    });

    let secret_data_mock = server.mock(|when, then| {
        when.method(GET).path_contains(VaultClient::FETCH_SECRETS);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    });

    let wk_temp = assert_fs::TempDir::new().unwrap();
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"
                    okta_client_id = "valid-okta-client-id"

                    [vault]
                    api_token = "valid_vault_api_token"

                    [auth]
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

    let elixir_temp = assert_fs::TempDir::new().unwrap();
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

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#a.secret.exs
                import_config "config/a.secret.exs"

                # wukong.mindvalley.dev/config-secrets-location: vault:secret/mv/tech/app/dev#b.secret.exs
                if File.exists?("config/b.secret.exs") do
                  import_config "config/b.secret.exs"
                end

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

    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

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

    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}
#[test]
fn test_wukong_dev_config_diff_without_dev_config() {
    let server = MockServer::start();

    let secret_api_resp = r#"
        {
          "data": {
            "data": {
              "b.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5",
              "c.secret.exs": "use Mix.Config\n\nconfig :application, Application.Repo,\n  adapter: Ecto.Adapters.Postgres,\n  username: System.get_env(\"DB_USER\"),\n  password: System.get_env(\"DB_PASS\"),\n  database: \"application_dev\",\n  hostname: \"localhost\",\n  pool_size: 100,\n  queue_target: 5"
            },
            "metadata": {
              "created_time": "2015-02-22T02:24:06.945319214Z",
              "custom_metadata": {
                "owner": "xxx",
                "mission_critical": "false"
              },
              "deletion_time": "",
              "destroyed": false,
              "version": 2
            }
          }
        }"#;

    let verify_token_api_resp = r#"
        {
          "data": {
            "expire_time": "2019-12-10T10:10:10.000000Z",
            "issue_time": "2019-10-10T10:10:10.000000Z"
            }
        }"#;

    let verify_token_mock = server.mock(|when, then| {
        when.method(GET)
            .path_contains(VaultClient::VERIFY_TOKEN)
            .header("X-Vault-Token", "valid_vault_api_token");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(verify_token_api_resp);
    });

    let secret_data_mock = server.mock(|when, then| {
        when.method(GET).path_contains(VaultClient::FETCH_SECRETS);
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(secret_api_resp);
    });

    let wk_temp = assert_fs::TempDir::new().unwrap();
    let wk_config_file = wk_temp.child("config.toml");
    wk_config_file.touch().unwrap();

    wk_config_file
        .write_str(
            format!(
                r#"
                    [core]
                    application = "valid-application"
                    wukong_api_url = "{}"
                    okta_client_id = "valid-okta-client-id"

                    [vault]
                    api_token = "valid_vault_api_token"

                    [auth]
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

    let elixir_temp = assert_fs::TempDir::new().unwrap();
    env::set_current_dir(elixir_temp.path().to_str().unwrap()).unwrap();

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

    wk_temp.close().unwrap();
    elixir_temp.close().unwrap();
}
