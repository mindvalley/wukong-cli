complete -c wukong -n "__fish_use_subcommand" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_use_subcommand" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_use_subcommand" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c wukong -n "__fish_use_subcommand" -f -a "init" -d 'Initialize Wukong\'s configurations'
complete -c wukong -n "__fish_use_subcommand" -f -a "application" -d 'This command group contains the commands to interact with an application’s configurations'
complete -c wukong -n "__fish_use_subcommand" -f -a "pipeline" -d 'This command group contains the commands to view & interact with an application’s pipeline'
complete -c wukong -n "__fish_use_subcommand" -f -a "deployment" -d 'This command group contains the commands to view and interact with the Continuous Delivery pipeline of an application'
complete -c wukong -n "__fish_use_subcommand" -f -a "dev" -d 'This command group contains the commands to interact with the local development environment'
complete -c wukong -n "__fish_use_subcommand" -f -a "config" -d 'This command group contains the commands to view & interact with Wukong\'s configurations'
complete -c wukong -n "__fish_use_subcommand" -f -a "login" -d 'Login to start using wukong command'
complete -c wukong -n "__fish_use_subcommand" -f -a "completion" -d 'Generate wukong cli completions for your shell to stdout'
complete -c wukong -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from init" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from init" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from init" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the current pipelines of the application'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "describe" -d 'Show the details of a pipeline'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "ci-status" -d 'Show the build status and (possible) errors on branch ci pipeline'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -l repo-url -d 'Repository url' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -l branch -d 'Branch name' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the current pipelines of the application'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "describe" -d 'Show the details of a pipeline'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "ci-status" -d 'Show the build status and (possible) errors on branch ci pipeline'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the current available deployment pipelines of an application'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "execute" -d 'Start the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "rollback" -d 'Rollback the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l namespace -d 'The namespace to deploy to' -r -f -a "{prod	,staging	}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l version -d 'The version that the deployment will perform against' -r -f -a "{blue	,green	}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l artifact -d 'The build artifact that the deployment will use' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -l namespace -d 'The namespace to deploy to' -r -f -a "{prod	,staging	}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -l version -d 'The version that the deployment will perform against' -r -f -a "{blue	,green	}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the current available deployment pipelines of an application'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "execute" -d 'Start the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "rollback" -d 'Rollback the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config-lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config-synthesizer" -d 'Synthesize the development config with secrets file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-lint" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-lint" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-lint" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-lint" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-synthesizer" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-synthesizer" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-synthesizer" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config-synthesizer" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config-lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config-synthesizer" -d 'Synthesize the development config with secrets file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the configurations'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "set" -d 'Set the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "get" -d 'Print the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the configurations'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "set" -d 'Set the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "get" -d 'Print the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from login" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from login" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from login" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from login" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from completion" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize Wukong\'s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "application" -d 'This command group contains the commands to interact with an application’s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "pipeline" -d 'This command group contains the commands to view & interact with an application’s pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "deployment" -d 'This command group contains the commands to view and interact with the Continuous Delivery pipeline of an application'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "dev" -d 'This command group contains the commands to interact with the local development environment'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to view & interact with Wukong\'s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "login" -d 'Login to start using wukong command'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate wukong cli completions for your shell to stdout'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "list" -d 'List the current pipelines of the application'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "describe" -d 'Show the details of a pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "ci-status" -d 'Show the build status and (possible) errors on branch ci pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "list" -d 'List the current available deployment pipelines of an application'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "execute" -d 'Start the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "rollback" -d 'Rollback the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config" -f -a "config-lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config" -f -a "config-synthesizer" -d 'Synthesize the development config with secrets file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config-lint; and not __fish_seen_subcommand_from config-synthesizer; and not __fish_seen_subcommand_from config" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "list" -d 'List the configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "set" -d 'Set the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "get" -d 'Print the value of a configuration'
