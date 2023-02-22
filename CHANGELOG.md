# Changelog

## [0.0.5-beta2](https://github.com/mindvalley/wukong-cli/tree/0.0.5-beta2) (2023-02-22)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.4-beta1...0.0.5-beta2)

**Implemented enhancements:**

-  PXP-467: \[CLI\] Generating a default changelog for the rollback operation \(re-open\) [\#62](https://github.com/mindvalley/wukong-cli/pull/62) ([jk-gan](https://github.com/jk-gan))
- PXP-466: \[Wukong CLI\] Refactor the configuration logic [\#60](https://github.com/mindvalley/wukong-cli/pull/60) ([jk-gan](https://github.com/jk-gan))
- Benchmarking Output Tokenizer [\#56](https://github.com/mindvalley/wukong-cli/pull/56) ([jk-gan](https://github.com/jk-gan))
- PXP-459: \[CLI\] Do discovery on whether or not we can do text tokenisation on CLI outputs [\#54](https://github.com/mindvalley/wukong-cli/pull/54) ([jk-gan](https://github.com/jk-gan))
- PXP-445: \[CLI\] Printing DEBUG log from the CLI [\#52](https://github.com/mindvalley/wukong-cli/pull/52) ([jk-gan](https://github.com/jk-gan))
- PXP-444: \[CLI\] Support --verbose flag and WUKONG\_LOG flag [\#51](https://github.com/mindvalley/wukong-cli/pull/51) ([jk-gan](https://github.com/jk-gan))
- Add config file for development environment [\#50](https://github.com/mindvalley/wukong-cli/pull/50) ([jk-gan](https://github.com/jk-gan))
- PXP-423: \[CLI\] Display the build artifact name when listing deployments [\#44](https://github.com/mindvalley/wukong-cli/pull/44) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version to v0.0.5-beta2 [\#66](https://github.com/mindvalley/wukong-cli/pull/66) ([onimsha](https://github.com/onimsha))
- PXP-481: \[Wukong CLI\] Telemetry panic when the `~/.config/wukong` path is not exists  [\#65](https://github.com/mindvalley/wukong-cli/pull/65) ([jk-gan](https://github.com/jk-gan))
- Bump version to v0.0.5-beta1 [\#64](https://github.com/mindvalley/wukong-cli/pull/64) ([onimsha](https://github.com/onimsha))
- PXP-468: \[CLI\] Use the new field to determine what is the current deployed artifact [\#63](https://github.com/mindvalley/wukong-cli/pull/63) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump openssl-src from 111.24.0+1.1.1s to 111.25.0+1.1.1t [\#59](https://github.com/mindvalley/wukong-cli/pull/59) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump tokio from 1.23.1 to 1.24.2 [\#58](https://github.com/mindvalley/wukong-cli/pull/58) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-463: \[CLI\] Implement the rollback command [\#57](https://github.com/mindvalley/wukong-cli/pull/57) ([jk-gan](https://github.com/jk-gan))
- PXP-434: \[CLI\] Implement the application info command [\#55](https://github.com/mindvalley/wukong-cli/pull/55) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump tokio from 1.23.0 to 1.23.1 [\#53](https://github.com/mindvalley/wukong-cli/pull/53) ([dependabot[bot]](https://github.com/apps/dependabot))
- Correct the command to check Wukong CLI version [\#49](https://github.com/mindvalley/wukong-cli/pull/49) ([onimsha](https://github.com/onimsha))
- Add CHANGELOG [\#48](https://github.com/mindvalley/wukong-cli/pull/48) ([onimsha](https://github.com/onimsha))
- PXP-426: \[CLI\] Show which build artifact is the current deployed one when selecting artifact [\#46](https://github.com/mindvalley/wukong-cli/pull/46) ([jk-gan](https://github.com/jk-gan))

## [0.0.4-beta1](https://github.com/mindvalley/wukong-cli/tree/0.0.4-beta1) (2022-12-09)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.3-alpha2...0.0.4-beta1)

**Implemented enhancements:**

- PXP-436: \[Wukong CLI\] Display message to tell user that there are a deployment with same arguments is running [\#42](https://github.com/mindvalley/wukong-cli/pull/42) ([jk-gan](https://github.com/jk-gan))
- PXP-417: \[Wukong CLI\] Don't display selection if it's not available for an application [\#40](https://github.com/mindvalley/wukong-cli/pull/40) ([jk-gan](https://github.com/jk-gan))
- PXP-416: \[Wukong CLI\] Allowing to override the api\_url and okta\_client\_id in the config [\#35](https://github.com/mindvalley/wukong-cli/pull/35) ([jk-gan](https://github.com/jk-gan))
- PXP-418: \[Wukong CLI\] Asking user to continue to deploy when selecting the same build as current deployed ref. [\#34](https://github.com/mindvalley/wukong-cli/pull/34) ([jk-gan](https://github.com/jk-gan))
- PXP-412: \[Wukong CLI\] Remove all the unwrap function  [\#33](https://github.com/mindvalley/wukong-cli/pull/33) ([jk-gan](https://github.com/jk-gan))
- PXP-411: \[CLI\] Support render more than 1 commit in the build artifacts listing step [\#30](https://github.com/mindvalley/wukong-cli/pull/30) ([jk-gan](https://github.com/jk-gan))
- PXP-397: \[CLI\] Support editing the CHANGELOG & Post to Slack [\#28](https://github.com/mindvalley/wukong-cli/pull/28) ([jk-gan](https://github.com/jk-gan))

**Fixed bugs:**

- PXP-419: \[Wukong CLI\] Cursor disappear when terminating the interactive session in the middle. [\#39](https://github.com/mindvalley/wukong-cli/pull/39) ([jk-gan](https://github.com/jk-gan))

**Security fixes:**

- Fix the security issue CVE-2020-26235 [\#45](https://github.com/mindvalley/wukong-cli/pull/45) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version [\#47](https://github.com/mindvalley/wukong-cli/pull/47) ([onimsha](https://github.com/onimsha))
- PXP-405: \[Wukong CLI\] Implement Telemetry metrics [\#43](https://github.com/mindvalley/wukong-cli/pull/43) ([jk-gan](https://github.com/jk-gan))
- PXP-428: \[CLI\] Display the Trigger User when listing deployments [\#41](https://github.com/mindvalley/wukong-cli/pull/41) ([jk-gan](https://github.com/jk-gan))
- PXP-424: \[CLI\] Use Short Commit Hash rather than full commit hash [\#38](https://github.com/mindvalley/wukong-cli/pull/38) ([jk-gan](https://github.com/jk-gan))
- Update README.md [\#37](https://github.com/mindvalley/wukong-cli/pull/37) ([njausteve](https://github.com/njausteve))
- Fix clippy errors [\#36](https://github.com/mindvalley/wukong-cli/pull/36) ([jk-gan](https://github.com/jk-gan))

## [0.0.3-alpha2](https://github.com/mindvalley/wukong-cli/tree/0.0.3-alpha2) (2022-10-27)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.3-alpha1...0.0.3-alpha2)

**Implemented enhancements:**

- PXP-393: \[CLI\] Makes CHANGELOG paginable using keyboard [\#27](https://github.com/mindvalley/wukong-cli/pull/27) ([jk-gan](https://github.com/jk-gan))
- PXP-389: Display better error message when cannot detect working folder [\#25](https://github.com/mindvalley/wukong-cli/pull/25) ([jk-gan](https://github.com/jk-gan))
- Rename command completions to completion [\#23](https://github.com/mindvalley/wukong-cli/pull/23) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version [\#31](https://github.com/mindvalley/wukong-cli/pull/31) ([onimsha](https://github.com/onimsha))
- Add KNOWN ISSUE section [\#29](https://github.com/mindvalley/wukong-cli/pull/29) ([onimsha](https://github.com/onimsha))
- PXP-390: \[CLI\] Improve the output of the ci-status when running against branch without PR [\#26](https://github.com/mindvalley/wukong-cli/pull/26) ([jk-gan](https://github.com/jk-gan))

## [0.0.3-alpha1](https://github.com/mindvalley/wukong-cli/tree/0.0.3-alpha1) (2022-10-14)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.3-alpha...0.0.3-alpha1)

**Merged pull requests:**

- refactor: add newline to changelog [\#22](https://github.com/mindvalley/wukong-cli/pull/22) ([jk-gan](https://github.com/jk-gan))

## [0.0.3-alpha](https://github.com/mindvalley/wukong-cli/tree/0.0.3-alpha) (2022-10-13)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.1-alpha...0.0.3-alpha)

## [0.0.1-alpha](https://github.com/mindvalley/wukong-cli/tree/0.0.1-alpha) (2022-10-12)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.2-alpha...0.0.1-alpha)

## [0.0.2-alpha](https://github.com/mindvalley/wukong-cli/tree/0.0.2-alpha) (2022-10-12)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.1-dev...0.0.2-alpha)

**Merged pull requests:**

- Upgrade to Clap v4 [\#21](https://github.com/mindvalley/wukong-cli/pull/21) ([jk-gan](https://github.com/jk-gan))
- PXP-347: Implement the Deployment API Client [\#20](https://github.com/mindvalley/wukong-cli/pull/20) ([jk-gan](https://github.com/jk-gan))
- PXP-378: Establish a release procedure [\#8](https://github.com/mindvalley/wukong-cli/pull/8) ([jk-gan](https://github.com/jk-gan))

## [0.0.1-dev](https://github.com/mindvalley/wukong-cli/tree/0.0.1-dev) (2022-10-05)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/cc9aa01a08af06b2fab283180cfceb340d44561b...0.0.1-dev)

**Implemented enhancements:**

- PXP-342: Implement client interface and helper [\#2](https://github.com/mindvalley/wukong-cli/pull/2) ([jk-gan](https://github.com/jk-gan))
- Setup project [\#1](https://github.com/mindvalley/wukong-cli/pull/1) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- chore\(brew\): update sha256 hash [\#17](https://github.com/mindvalley/wukong-cli/pull/17) ([jk-gan](https://github.com/jk-gan))
- chore\(brew\): update sha256 values [\#14](https://github.com/mindvalley/wukong-cli/pull/14) ([jk-gan](https://github.com/jk-gan))
- Fix the homebrew strategy file [\#13](https://github.com/mindvalley/wukong-cli/pull/13) ([jk-gan](https://github.com/jk-gan))
- Fix homebrew download strategy [\#12](https://github.com/mindvalley/wukong-cli/pull/12) ([jk-gan](https://github.com/jk-gan))
- Fix initialize method [\#11](https://github.com/mindvalley/wukong-cli/pull/11) ([jk-gan](https://github.com/jk-gan))
- feat: add homebrew formula [\#10](https://github.com/mindvalley/wukong-cli/pull/10) ([jk-gan](https://github.com/jk-gan))
- Use id\_token for api call [\#9](https://github.com/mindvalley/wukong-cli/pull/9) ([jk-gan](https://github.com/jk-gan))
- PXP-371: CLI should support autocomplete for the commands [\#7](https://github.com/mindvalley/wukong-cli/pull/7) ([jk-gan](https://github.com/jk-gan))
- PXP-367: Fetch the list of available applications from API [\#6](https://github.com/mindvalley/wukong-cli/pull/6) ([jk-gan](https://github.com/jk-gan))
- PXP-350: Implements Authentication via Okta [\#5](https://github.com/mindvalley/wukong-cli/pull/5) ([jk-gan](https://github.com/jk-gan))
- PXP-343: Implement the API Client for Pipeline commands [\#4](https://github.com/mindvalley/wukong-cli/pull/4) ([jk-gan](https://github.com/jk-gan))
- PXP-349: Global Flags and CLI Config [\#3](https://github.com/mindvalley/wukong-cli/pull/3) ([jk-gan](https://github.com/jk-gan))



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
