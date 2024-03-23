# UPGRADE FROM 2.0.X SERIES TO 2.1.0.

## BREAKING CHANGES

In the latest version of Wukong CLI, we've made a significant change to how the CLI operates. Now, the CLI requires to be run inside a working folder that contains the `.wukong.toml` configuration file.

### Why this change?

This change allows us to ensure that all the necessary configurations are in place before the CLI operations are performed. It helps in maintaining consistency and avoiding errors that might occur due to missing or incorrect configurations.

Previously, we allow the Wukong CLI to run abitrary commands in anywhere inside the terminal, for convenient purpose. The downside of this approach, is we put all the application-specific configurations in the Wukong API side, and let the Wukong CLI to query the API whenever it needs the information to sucessfully execute the command. This makes the Wukong API a black box, and enabling a new application to use the Wukong CLI is impossible without adding a new configuration in the Wukong API.

When we started to redesign the Wukong CLI 2.x, we wanted to give the Wukong CLI the self-serve capability, so that newly projects can immedidately use the Wukong CLI to configure their application and use it without any middle-man. Given that the ultimate goal is to let developers manage their application from code-to-ship-to-operate, we made the decision to move the application config into their respective repository, so it can unlock these capabilities in future development of the Wukong project.

### How to use?

To use the CLI, navigate to your project's root directory (or any directory containing the `.wukong.toml` file) in your terminal. Then, you can run the Wukong CLI commands as usual.

For example:

```sh
cd /path/to/your/project
wukong command
```
