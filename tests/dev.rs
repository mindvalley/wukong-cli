mod common;

#[test]
fn test_wukong_dev_help() {
    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

// #[test]
// fn test_wukong_dev_config_lint_success() {
//     let temp = assert_fs::TempDir::new().unwrap();
//     let main_config_file = temp.child("config/config.exs");
//     main_config_file.touch().unwrap();
//
//     main_config_file
//         .write_str(
//             r#"
// use Mix.Config
//
// System.get_env("API_KEY")
// System.fetch_env("API_SECRET")
// System.fetch_env!("API_TOKEN")
//
// # invalid
// import_config "config/dev.exs"
//
// # valid
// if File.exists?("config/dev.exs") do
//   import_config "config/dev.exs"
// end
//
// # invalid
// if File.exists?("config/a.exs") do
//   import_config "config/b.exs"
// end
//
// # valid
// File.exists?("config/dev.exs") && import_config "config/dev.exs"
// # invalid
// File.exists?("config/a.exs") && import_config "config/b.exs"
//
// test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")
//
// # Use Jason for JSON parsing in Phoenix
// config :phoenix, :json_library, Jason
//     "#,
//         )
//         .unwrap();
//
//     let dev_config_file = temp.child("config/dev.exs");
//     dev_config_file.touch().unwrap();
//
//     dev_config_file
//         .write_str(
//             r#"
// use Mix.Config
//
// System.get_env("API_KEY")
// System.fetch_env("API_SECRET")
// System.fetch_env!("API_TOKEN")
//
// # invalid
// import_config "config/dev.exs"
//
// # valid
// if File.exists?("config/dev.exs") do
//   import_config "config/dev.exs"
// end
//
// # invalid
// if File.exists?("config/a.exs") do
//   import_config "config/b.exs"
// end
//
// # valid
// File.exists?("config/dev.exs") && import_config "config/dev.exs"
// # invalid
// File.exists?("config/a.exs") && import_config "config/b.exs"
//
// test_domain = System.get_env("TEST_DOMAIN", "mv.test.com")
//
// # Use Jason for JSON parsing in Phoenix
// config :phoenix, :json_library, Jason
//     "#,
//         )
//         .unwrap();
//
//     let cmd = common::wukong_raw_command()
//         .arg("dev")
//         .arg("config-lint")
//         .arg(temp.path().to_str().unwrap())
//         .assert()
//         .success();
//
//     let output = cmd.get_output();
//
//     insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
//
//     temp.close().unwrap();
// }
