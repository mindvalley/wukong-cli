# Changelog

## [Unreleased](https://github.com/mindvalley/wukong-cli/tree/HEAD)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/2.1.0...HEAD)

**Merged pull requests:**

- Update documentation about the breaking changes in version 2.1.0. [\#212](https://github.com/mindvalley/wukong-cli/pull/212) ([onimsha](https://github.com/onimsha))

## [2.1.0](https://github.com/mindvalley/wukong-cli/tree/2.1.0) (2024-03-21)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/2.0.2...2.1.0)

**Implemented enhancements:**

- chore: cleanup dependencies and use workspace dependencies for share … [\#210](https://github.com/mindvalley/wukong-cli/pull/210) ([jk-gan](https://github.com/jk-gan))
- PXP-685: \[CLI\] Improve the warning when CloudSQL integration is not enabled [\#209](https://github.com/mindvalley/wukong-cli/pull/209) ([jk-gan](https://github.com/jk-gan))

**Security fixes:**

- deps: upgrade dependencies to patch security warnings [\#207](https://github.com/mindvalley/wukong-cli/pull/207) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version and dependencies [\#211](https://github.com/mindvalley/wukong-cli/pull/211) ([onimsha](https://github.com/onimsha))
- chore: add telemetry for deployment status command [\#208](https://github.com/mindvalley/wukong-cli/pull/208) ([jk-gan](https://github.com/jk-gan))
- PXP-671: \[CLI\] Implement the CloudSQL integration to the deployment status command [\#206](https://github.com/mindvalley/wukong-cli/pull/206) ([jk-gan](https://github.com/jk-gan))
- PXP-678: \[CLI\] Integrate application init name validation [\#205](https://github.com/mindvalley/wukong-cli/pull/205) ([mfauzaan](https://github.com/mfauzaan))
- chore\(deps\): bump mio from 0.8.10 to 0.8.11 [\#204](https://github.com/mindvalley/wukong-cli/pull/204) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-676: \[CLI\] Skip workflow step on application init if empty [\#203](https://github.com/mindvalley/wukong-cli/pull/203) ([mfauzaan](https://github.com/mfauzaan))
- PXP-660 feat: add cloudsql database metrics panel [\#202](https://github.com/mindvalley/wukong-cli/pull/202) ([Fadhil](https://github.com/Fadhil))
- PXP-675: \[CLI\] Make canary flag enabled by default [\#201](https://github.com/mindvalley/wukong-cli/pull/201) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump mio from 0.8.8 to 0.8.11 in /sdk [\#200](https://github.com/mindvalley/wukong-cli/pull/200) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-672: \[CLI\] Include .yaml file types in application init workflow list [\#199](https://github.com/mindvalley/wukong-cli/pull/199) ([mfauzaan](https://github.com/mfauzaan))
- PXP-670: \[CLI\] Implement the AppSignal integration to the deployment status command [\#198](https://github.com/mindvalley/wukong-cli/pull/198) ([jk-gan](https://github.com/jk-gan))
- Fix/handle broken google cloud welcome screen on tui [\#197](https://github.com/mindvalley/wukong-cli/pull/197) ([mfauzaan](https://github.com/mfauzaan))
- PXP-650: \[CLI\] Develop a panel on the TUI to display AppSignal data [\#196](https://github.com/mindvalley/wukong-cli/pull/196) ([jk-gan](https://github.com/jk-gan))
- chore\(github\): update github actions [\#195](https://github.com/mindvalley/wukong-cli/pull/195) ([leylmordor](https://github.com/leylmordor))
- Fix: Apply new clippy changes for for rust version \(1.26.0\) [\#194](https://github.com/mindvalley/wukong-cli/pull/194) ([mfauzaan](https://github.com/mfauzaan))
- PXP-664 refactor: replace global init with inquire library [\#193](https://github.com/mindvalley/wukong-cli/pull/193) ([mfauzaan](https://github.com/mfauzaan))
- PXP-644 feat: add application init command [\#192](https://github.com/mindvalley/wukong-cli/pull/192) ([mfauzaan](https://github.com/mfauzaan))
- PXP-659 feat: add cloud sql instances and metric clients from google protobuf… [\#191](https://github.com/mindvalley/wukong-cli/pull/191) ([Fadhil](https://github.com/Fadhil))
- chore\(deps\): bump h2 from 0.3.19 to 0.3.24 in /sdk [\#187](https://github.com/mindvalley/wukong-cli/pull/187) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-638: \[CLI\] Implement new config [\#183](https://github.com/mindvalley/wukong-cli/pull/183) ([mfauzaan](https://github.com/mfauzaan))

## [2.0.2](https://github.com/mindvalley/wukong-cli/tree/2.0.2) (2024-01-22)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/2.0.1...2.0.2)

**Implemented enhancements:**

- PXP-633: Better error handling if there is failure on JSON decode  [\#180](https://github.com/mindvalley/wukong-cli/pull/180) ([mfauzaan](https://github.com/mfauzaan))

**Fixed bugs:**

- PXP-639: \[CLI\] Ctrl-e stopped working on the latest codebase [\#175](https://github.com/mindvalley/wukong-cli/pull/175) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Add artifact download path to release workflow [\#190](https://github.com/mindvalley/wukong-cli/pull/190) ([onimsha](https://github.com/onimsha))
- Fix a typo in the download action [\#189](https://github.com/mindvalley/wukong-cli/pull/189) ([onimsha](https://github.com/onimsha))
- Update upload-artifact action to v4 [\#188](https://github.com/mindvalley/wukong-cli/pull/188) ([onimsha](https://github.com/onimsha))
- Update tests and dependencies [\#186](https://github.com/mindvalley/wukong-cli/pull/186) ([onimsha](https://github.com/onimsha))
- PXP-663: \[CLI\] Depreciate old error handling [\#185](https://github.com/mindvalley/wukong-cli/pull/185) ([mfauzaan](https://github.com/mfauzaan))
- PXP-662 fix: pipeline status request does not need builds [\#184](https://github.com/mindvalley/wukong-cli/pull/184) ([Fadhil](https://github.com/Fadhil))
- PXP-638: \[CLI\] Reached API limits on Google Logging [\#182](https://github.com/mindvalley/wukong-cli/pull/182) ([jk-gan](https://github.com/jk-gan))
- PXP-641: Handle event sender panic [\#181](https://github.com/mindvalley/wukong-cli/pull/181) ([mfauzaan](https://github.com/mfauzaan))
- PXP-636: \[CLI\] Support a hotkey to force reload the TUI [\#179](https://github.com/mindvalley/wukong-cli/pull/179) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump openssl from 0.10.59 to 0.10.60 [\#178](https://github.com/mindvalley/wukong-cli/pull/178) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump openssl from 0.10.55 to 0.10.60 in /sdk [\#177](https://github.com/mindvalley/wukong-cli/pull/177) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-640: \[CLI\] Log panel panic when the start index out of bound [\#176](https://github.com/mindvalley/wukong-cli/pull/176) ([jk-gan](https://github.com/jk-gan))
- PXP-635: \[CLI\] Terminate the TUI when there is a panic on TUI [\#174](https://github.com/mindvalley/wukong-cli/pull/174) ([mfauzaan](https://github.com/mfauzaan))
- PXP-634:  \[CLI\] Add welcome before showing TUI panels  [\#173](https://github.com/mindvalley/wukong-cli/pull/173) ([mfauzaan](https://github.com/mfauzaan))
- Update README to add installation steps [\#172](https://github.com/mindvalley/wukong-cli/pull/172) ([onimsha](https://github.com/onimsha))
- Update CHANGELOG [\#171](https://github.com/mindvalley/wukong-cli/pull/171) ([onimsha](https://github.com/onimsha))
- PXP-617: \[CLI\] Logs Panel: Support line wrapping [\#170](https://github.com/mindvalley/wukong-cli/pull/170) ([jk-gan](https://github.com/jk-gan))

## [2.0.1](https://github.com/mindvalley/wukong-cli/tree/2.0.1) (2023-11-06)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/2.0.0-alpha2...2.0.1)

**Fixed bugs:**

- PXP-631: \[CLI\] The header layout is broken on big screen [\#166](https://github.com/mindvalley/wukong-cli/pull/166) ([jk-gan](https://github.com/jk-gan))
- PXP-630: \[CLI\]: fix ctrl+e panic [\#165](https://github.com/mindvalley/wukong-cli/pull/165) ([mfauzaan](https://github.com/mfauzaan))
- PXP-627: \[CLI\] Key binding is incorrect for the logs panel [\#157](https://github.com/mindvalley/wukong-cli/pull/157) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump CLI version to 2.0.1 [\#169](https://github.com/mindvalley/wukong-cli/pull/169) ([onimsha](https://github.com/onimsha))
- Bump version of the cli [\#168](https://github.com/mindvalley/wukong-cli/pull/168) ([onimsha](https://github.com/onimsha))
- PXP-632: \[CLI\] Okta tokens did not being updated after refresh [\#167](https://github.com/mindvalley/wukong-cli/pull/167) ([jk-gan](https://github.com/jk-gan))
- PXP-628: \[CLI\]: Display new release info banner [\#164](https://github.com/mindvalley/wukong-cli/pull/164) ([mfauzaan](https://github.com/mfauzaan))
- PXP-628: \[CLI\]: Collect debug logs to a file [\#163](https://github.com/mindvalley/wukong-cli/pull/163) ([mfauzaan](https://github.com/mfauzaan))
- PXP-599: \[CLI\] Logs panel: Support time filtering \(Reopen to base on the main branch\) [\#162](https://github.com/mindvalley/wukong-cli/pull/162) ([jk-gan](https://github.com/jk-gan))
- PXP-625: \[CLI\] Report the Wukong version to telemetry [\#161](https://github.com/mindvalley/wukong-cli/pull/161) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump rustix from 0.37.20 to 0.37.25 in /sdk [\#159](https://github.com/mindvalley/wukong-cli/pull/159) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-610: \[CLI\] Panel Expand [\#158](https://github.com/mindvalley/wukong-cli/pull/158) ([jk-gan](https://github.com/jk-gan))
- PXP-626: \[CLI\] Remove the tick-based canvas refresh [\#155](https://github.com/mindvalley/wukong-cli/pull/155) ([jk-gan](https://github.com/jk-gan))

## [2.0.0-alpha2](https://github.com/mindvalley/wukong-cli/tree/2.0.0-alpha2) (2023-10-17)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/2.0.0-alpha1...2.0.0-alpha2)

**Implemented enhancements:**

- PXP-583: \[CLI\] New Error Codes handling [\#144](https://github.com/mindvalley/wukong-cli/pull/144) ([jk-gan](https://github.com/jk-gan))

**Fixed bugs:**

- PXP-616: \[CLI\] Log Entries is not reset after a new namespace or a new version is selected [\#152](https://github.com/mindvalley/wukong-cli/pull/152) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version and update latest dependencies [\#156](https://github.com/mindvalley/wukong-cli/pull/156) ([onimsha](https://github.com/onimsha))
- chore\(deps\): bump webpki from 0.22.1 to 0.22.2 [\#154](https://github.com/mindvalley/wukong-cli/pull/154) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump webpki from 0.22.0 to 0.22.2 in /sdk [\#153](https://github.com/mindvalley/wukong-cli/pull/153) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-597: \[CLI\] Logs panel: Support search / filter for logs [\#151](https://github.com/mindvalley/wukong-cli/pull/151) ([jk-gan](https://github.com/jk-gan))
- PXP-608: Make panels selectable [\#150](https://github.com/mindvalley/wukong-cli/pull/150) ([mfauzaan](https://github.com/mfauzaan))
- PXP-611: add error logs filter on tui [\#149](https://github.com/mindvalley/wukong-cli/pull/149) ([mfauzaan](https://github.com/mfauzaan))
- PXP-598: Add log tailing hotkey [\#148](https://github.com/mindvalley/wukong-cli/pull/148) ([mfauzaan](https://github.com/mfauzaan))
- PXP-613: Update API calls order [\#147](https://github.com/mindvalley/wukong-cli/pull/147) ([mfauzaan](https://github.com/mfauzaan))
- PXP-600: Display data from different version using hotkey [\#146](https://github.com/mindvalley/wukong-cli/pull/146) ([mfauzaan](https://github.com/mfauzaan))
- PXP-596: Optimize the logs rendering on tui [\#145](https://github.com/mindvalley/wukong-cli/pull/145) ([mfauzaan](https://github.com/mfauzaan))
- Update CHANGELOG [\#143](https://github.com/mindvalley/wukong-cli/pull/143) ([onimsha](https://github.com/onimsha))

## [2.0.0-alpha1](https://github.com/mindvalley/wukong-cli/tree/2.0.0-alpha1) (2023-09-06)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.2.0...2.0.0-alpha1)

**Implemented enhancements:**

- PXP-594: Find a solution to support a canary api in the CLI [\#139](https://github.com/mindvalley/wukong-cli/pull/139) ([jk-gan](https://github.com/jk-gan))
- PXP-579: \[CLI\] Implement the logs section [\#137](https://github.com/mindvalley/wukong-cli/pull/137) ([jk-gan](https://github.com/jk-gan))
- PXP-581: \[CLI\] Implement the deployment section [\#134](https://github.com/mindvalley/wukong-cli/pull/134) ([jk-gan](https://github.com/jk-gan))
- PXP-580: \[CLI\] Implement the build artifact section [\#133](https://github.com/mindvalley/wukong-cli/pull/133) ([jk-gan](https://github.com/jk-gan))

**Fixed bugs:**

- fix: fix errors from the latest dependencies [\#142](https://github.com/mindvalley/wukong-cli/pull/142) ([jk-gan](https://github.com/jk-gan))
- PXP-604: handle permission error for the TUI [\#140](https://github.com/mindvalley/wukong-cli/pull/140) ([mfauzaan](https://github.com/mfauzaan))

**Merged pull requests:**

- Bump version and update to latest dependencies [\#141](https://github.com/mindvalley/wukong-cli/pull/141) ([onimsha](https://github.com/onimsha))
- PXP-595: integrate github pipeline while fetching builds  [\#138](https://github.com/mindvalley/wukong-cli/pull/138) ([mfauzaan](https://github.com/mfauzaan))
- chore\(deps\): bump rustls-webpki from 0.100.1 to 0.100.2 in /sdk [\#136](https://github.com/mindvalley/wukong-cli/pull/136) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump rustls-webpki from 0.100.1 to 0.100.2 [\#135](https://github.com/mindvalley/wukong-cli/pull/135) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-584: \[CLI\] Implement the hotkeys [\#132](https://github.com/mindvalley/wukong-cli/pull/132) ([jk-gan](https://github.com/jk-gan))
- PXP-578: \[CLI\] Implement the headers section [\#131](https://github.com/mindvalley/wukong-cli/pull/131) ([jk-gan](https://github.com/jk-gan))
- PXP-577: \[CLI\] Implement the command [\#130](https://github.com/mindvalley/wukong-cli/pull/130) ([jk-gan](https://github.com/jk-gan))
- PXP-559: \[CLI\] Update the Github Actions workflows [\#129](https://github.com/mindvalley/wukong-cli/pull/129) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump openssl from 0.10.54 to 0.10.55 in /sdk [\#128](https://github.com/mindvalley/wukong-cli/pull/128) ([dependabot[bot]](https://github.com/apps/dependabot))

## [1.2.0](https://github.com/mindvalley/wukong-cli/tree/1.2.0) (2023-08-03)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.1.1...1.2.0)

**Implemented enhancements:**

- PXP-555: Refactoring code [\#126](https://github.com/mindvalley/wukong-cli/pull/126) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- PXP-572: Handle blue/green deployment failure on deployment [\#127](https://github.com/mindvalley/wukong-cli/pull/127) ([mfauzaan](https://github.com/mfauzaan))

## [1.1.1](https://github.com/mindvalley/wukong-cli/tree/1.1.1) (2023-07-21)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.1.0...1.1.1)

**Implemented enhancements:**

- PXP-571: \[CLI\] Send telemetry data for new command groups [\#124](https://github.com/mindvalley/wukong-cli/pull/124) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump to version 1.1.1 [\#125](https://github.com/mindvalley/wukong-cli/pull/125) ([onimsha](https://github.com/onimsha))
- Update CHANGELOG [\#123](https://github.com/mindvalley/wukong-cli/pull/123) ([onimsha](https://github.com/onimsha))

## [1.1.0](https://github.com/mindvalley/wukong-cli/tree/1.1.0) (2023-07-20)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.0.3...1.1.0)

**Implemented enhancements:**

- PXP-567: \[CLI\] Crash when entering interactive session too long [\#119](https://github.com/mindvalley/wukong-cli/pull/119) ([jk-gan](https://github.com/jk-gan))
- PXP-568: \[CLI\] Make the application instances connect interactive [\#118](https://github.com/mindvalley/wukong-cli/pull/118) ([mfauzaan](https://github.com/mfauzaan))

**Merged pull requests:**

- Update README.md [\#122](https://github.com/mindvalley/wukong-cli/pull/122) ([onimsha](https://github.com/onimsha))
- Bump version to 1.1.0 [\#121](https://github.com/mindvalley/wukong-cli/pull/121) ([onimsha](https://github.com/onimsha))

## [1.0.3](https://github.com/mindvalley/wukong-cli/tree/1.0.3) (2023-07-11)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.0.2...1.0.3)

**Fixed bugs:**

- PXP-564: \[CLI\] Polling GraphQLquery for livebook status instead of subscription [\#117](https://github.com/mindvalley/wukong-cli/pull/117) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version and update dependencies [\#120](https://github.com/mindvalley/wukong-cli/pull/120) ([onimsha](https://github.com/onimsha))

## [1.0.2](https://github.com/mindvalley/wukong-cli/tree/1.0.2) (2023-07-03)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.0.1...1.0.2)

**Merged pull requests:**

- chore/bump version 1.0.2 [\#116](https://github.com/mindvalley/wukong-cli/pull/116) ([onimsha](https://github.com/onimsha))
- Update elixir\_working\_with\_secrets\_in\_bunker.md [\#115](https://github.com/mindvalley/wukong-cli/pull/115) ([onimsha](https://github.com/onimsha))
- chore\(deps\): bump openssl from 0.10.54 to 0.10.55 [\#114](https://github.com/mindvalley/wukong-cli/pull/114) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-548: \[CLI\] Program does not exit on Ctrl-C key press [\#112](https://github.com/mindvalley/wukong-cli/pull/112) ([jk-gan](https://github.com/jk-gan))
- chore/add more guides [\#111](https://github.com/mindvalley/wukong-cli/pull/111) ([onimsha](https://github.com/onimsha))
- chore/update changelog [\#110](https://github.com/mindvalley/wukong-cli/pull/110) ([onimsha](https://github.com/onimsha))
- PXP-549: \[API\] Google Cloud Oauth2 does not have the resfresh token [\#109](https://github.com/mindvalley/wukong-cli/pull/109) ([jk-gan](https://github.com/jk-gan))

## [1.0.1](https://github.com/mindvalley/wukong-cli/tree/1.0.1) (2023-06-12)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/1.0.0...1.0.1)

**Implemented enhancements:**

- PXP-531: \[CLI\] Support Generic secrets  [\#104](https://github.com/mindvalley/wukong-cli/pull/104) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- Bump version to 1.0.1 [\#108](https://github.com/mindvalley/wukong-cli/pull/108) ([onimsha](https://github.com/onimsha))
- PXP-547: \[CLI\] The cli is not working when the refresh token is expired. [\#107](https://github.com/mindvalley/wukong-cli/pull/107) ([jk-gan](https://github.com/jk-gan))
- Update CHANGELOG [\#106](https://github.com/mindvalley/wukong-cli/pull/106) ([onimsha](https://github.com/onimsha))
- Fix the reqwest error to stop the retrying during connection check [\#105](https://github.com/mindvalley/wukong-cli/pull/105) ([jk-gan](https://github.com/jk-gan))
- PXP-538: Renew auth token [\#103](https://github.com/mindvalley/wukong-cli/pull/103) ([mfauzaan](https://github.com/mfauzaan))

## [1.0.0](https://github.com/mindvalley/wukong-cli/tree/1.0.0) (2023-06-02)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.7-rc2...1.0.0)

**Merged pull requests:**

- Bump version to 1.0.0 [\#102](https://github.com/mindvalley/wukong-cli/pull/102) ([onimsha](https://github.com/onimsha))
- PXP-511: \[CLI\] Develop the connect-instance command [\#101](https://github.com/mindvalley/wukong-cli/pull/101) ([jk-gan](https://github.com/jk-gan))
- PXP-530: \[CLI\] Support PATCH API for updating secrets in Bunker [\#100](https://github.com/mindvalley/wukong-cli/pull/100) ([mfauzaan](https://github.com/mfauzaan))
- PXP-510: \[CLI\] Develop the list-instances command [\#99](https://github.com/mindvalley/wukong-cli/pull/99) ([jk-gan](https://github.com/jk-gan))
- PXP-503: \[CLI\] Implement the url-mode flag [\#97](https://github.com/mindvalley/wukong-cli/pull/97) ([jk-gan](https://github.com/jk-gan))
- PXP-501: \[CLI\] Implement the include & exclude flags [\#96](https://github.com/mindvalley/wukong-cli/pull/96) ([jk-gan](https://github.com/jk-gan))
- Add a guide for using the Vault integration feature [\#95](https://github.com/mindvalley/wukong-cli/pull/95) ([onimsha](https://github.com/onimsha))
- Update changelogs [\#94](https://github.com/mindvalley/wukong-cli/pull/94) ([onimsha](https://github.com/onimsha))
- PXP-500: \[CLI\] Implement the logs command [\#91](https://github.com/mindvalley/wukong-cli/pull/91) ([jk-gan](https://github.com/jk-gan))

## [0.0.7-rc2](https://github.com/mindvalley/wukong-cli/tree/0.0.7-rc2) (2023-04-20)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.7-rc1...0.0.7-rc2)

**Implemented enhancements:**

- PXP-519: \[CLI\] Change the linter rule to accept any path declared in the File.exists/2 function [\#92](https://github.com/mindvalley/wukong-cli/pull/92) ([mfauzaan](https://github.com/mfauzaan))
- PXP-517: \[CLI\] Rename "dev config-lint" command to "dev config lint" [\#90](https://github.com/mindvalley/wukong-cli/pull/90) ([mfauzaan](https://github.com/mfauzaan))
- PXP-514: \[CLI\] Rename 'dev config-synthesizer' to 'dev config pull' [\#87](https://github.com/mindvalley/wukong-cli/pull/87) ([mfauzaan](https://github.com/mfauzaan))

**Fixed bugs:**

- PXP-516: \[CLI\] Fix multiple annotations support  [\#88](https://github.com/mindvalley/wukong-cli/pull/88) ([mfauzaan](https://github.com/mfauzaan))

**Merged pull requests:**

- Bump version before release [\#93](https://github.com/mindvalley/wukong-cli/pull/93) ([onimsha](https://github.com/onimsha))
- chore\(deps\): bump h2 from 0.3.16 to 0.3.17 [\#89](https://github.com/mindvalley/wukong-cli/pull/89) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-515: \[CLI\] Develop the config secrets diff command [\#86](https://github.com/mindvalley/wukong-cli/pull/86) ([mfauzaan](https://github.com/mfauzaan))
- Update CHANGELOG [\#84](https://github.com/mindvalley/wukong-cli/pull/84) ([onimsha](https://github.com/onimsha))
- PXP-477: \[CLI\] Develop the config secrets push command [\#81](https://github.com/mindvalley/wukong-cli/pull/81) ([mfauzaan](https://github.com/mfauzaan))

## [0.0.7-rc1](https://github.com/mindvalley/wukong-cli/tree/0.0.7-rc1) (2023-04-06)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.6-rc1...0.0.7-rc1)

**Merged pull requests:**

- Bump version to 0.0.7-rc1 [\#83](https://github.com/mindvalley/wukong-cli/pull/83) ([onimsha](https://github.com/onimsha))
- PXP-476: \[CLI\] Develop the config synthesizer command [\#80](https://github.com/mindvalley/wukong-cli/pull/80) ([jk-gan](https://github.com/jk-gan))
- chore\(deps\): bump openssl from 0.10.45 to 0.10.48 [\#79](https://github.com/mindvalley/wukong-cli/pull/79) ([dependabot[bot]](https://github.com/apps/dependabot))
- PXP-475: \[Wukong CLI\] Refactor auth logic [\#78](https://github.com/mindvalley/wukong-cli/pull/78) ([mfauzaan](https://github.com/mfauzaan))
- PXP-489 retry request on query timeout [\#77](https://github.com/mindvalley/wukong-cli/pull/77) ([Fadhil](https://github.com/Fadhil))
- PXP-475: \[Wukong CLI\] Integrate vault client [\#76](https://github.com/mindvalley/wukong-cli/pull/76) ([mfauzaan](https://github.com/mfauzaan))

## [0.0.6-rc1](https://github.com/mindvalley/wukong-cli/tree/0.0.6-rc1) (2023-03-15)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.5-beta2...0.0.6-rc1)

**Implemented enhancements:**

- PXP-493: \[Wukong CLI\] Fix the text alignment when listing build artifacts [\#70](https://github.com/mindvalley/wukong-cli/pull/70) ([jk-gan](https://github.com/jk-gan))

**Fixed bugs:**

- fix: show refresh token error using debug output [\#75](https://github.com/mindvalley/wukong-cli/pull/75) ([jk-gan](https://github.com/jk-gan))
- fix: remove unused step for macos [\#74](https://github.com/mindvalley/wukong-cli/pull/74) ([jk-gan](https://github.com/jk-gan))

**Merged pull requests:**

- PXP-494: \[Wukong CLI\] Improving the debug log for Okta refresh token flow [\#73](https://github.com/mindvalley/wukong-cli/pull/73) ([jk-gan](https://github.com/jk-gan))
- Bumping version to 0.0.6-rc1 [\#72](https://github.com/mindvalley/wukong-cli/pull/72) ([onimsha](https://github.com/onimsha))
- Fix clippy rule [\#71](https://github.com/mindvalley/wukong-cli/pull/71) ([onimsha](https://github.com/onimsha))
- PXP-474: \[CLI\] Develop the ability to read annotations in dev config [\#69](https://github.com/mindvalley/wukong-cli/pull/69) ([jk-gan](https://github.com/jk-gan))
- PXP-488: \[CLI\] Implement the Elixir linter  [\#68](https://github.com/mindvalley/wukong-cli/pull/68) ([jk-gan](https://github.com/jk-gan))
- Update CHANGELOG and README [\#67](https://github.com/mindvalley/wukong-cli/pull/67) ([onimsha](https://github.com/onimsha))

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

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.2-alpha...0.0.3-alpha)

## [0.0.2-alpha](https://github.com/mindvalley/wukong-cli/tree/0.0.2-alpha) (2022-10-12)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.1-alpha...0.0.2-alpha)

## [0.0.1-alpha](https://github.com/mindvalley/wukong-cli/tree/0.0.1-alpha) (2022-10-12)

[Full Changelog](https://github.com/mindvalley/wukong-cli/compare/0.0.1-dev...0.0.1-alpha)

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
