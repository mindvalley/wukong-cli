complete -c wukong -n "__fish_use_subcommand" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_use_subcommand" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_use_subcommand" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_use_subcommand" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_use_subcommand" -f -a "tui" -d 'Start TUI session'
complete -c wukong -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from init" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from init" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from init" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from init" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "logs" -d 'Getting the logs of the applications from the Google Cloud Logging'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "instances" -d 'This command group contains the commands to interact with an application’s instances'
complete -c wukong -n "__fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l namespace -d '(optional) The namespace to deploy to' -r -f -a "{prod	'',staging	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l version -d '(optional) The version that the deployment will perform against' -r -f -a "{blue	'',green	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s s -l since -d 'Show logs lines newer from relative duration, e.g 5m, 1h, 1d. Also accept datetime in RFC 3339 format' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s u -l until -d 'Show logs lines older than relative duration, e.g 30m, 2h, 2d. Also accept datetime in RFC 3339 format' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l limit -d 'Limiting the number of log entries to return' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s i -l include -d '(allow multiple flags) Logs lines to include' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s e -l exclude -d '(allow multiple flags) Logs lines to exclude' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l errors -d 'Only print out logs line with severity >= ERROR'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l url-mode -d 'Generate the URL to view the logs in browser'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from logs" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "list" -d 'Listing the currently running Elixir instances, normally under a GKE Pod'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "connect" -d 'Start the interactive session to connect to the remote Elixir instance'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -l namespace -d '(optional) The namespace to list the running instances' -r -f -a "{prod	'',staging	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -l version -d '(optional) The version of the application to filter the returning running instances' -r -f -a "{blue	'',green	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -l namespace -d '(optional) The namespace to list the running instances' -r -f -a "{prod	'',staging	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -l version -d '(optional) The version of the application to filter the returning running instances' -r -f -a "{blue	'',green	''}"
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from connect" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "list" -d 'Listing the currently running Elixir instances, normally under a GKE Pod'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "connect" -d 'Start the interactive session to connect to the remote Elixir instance'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "logs" -d 'Getting the logs of the applications from the Google Cloud Logging'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "instances" -d 'This command group contains the commands to interact with an application’s instances'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect" -f -a "list" -d 'Listing the currently running Elixir instances, normally under a GKE Pod'
complete -c wukong -n "__fish_seen_subcommand_from application; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect" -f -a "connect" -d 'Start the interactive session to connect to the remote Elixir instance'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from describe" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from pipeline; and __fish_seen_subcommand_from ci-status" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l namespace -d 'The namespace to deploy to' -r -f -a "{prod	'',staging	''}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l version -d 'The version that the deployment will perform against' -r -f -a "{blue	'',green	''}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l artifact -d 'The build artifact that the deployment will use' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from execute" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -l namespace -d 'The namespace to deploy to' -r -f -a "{prod	'',staging	''}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -l version -d 'The version that the deployment will perform against' -r -f -a "{blue	'',green	''}"
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from rollback" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List the current available deployment pipelines of an application'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "execute" -d 'Start the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "rollback" -d 'Rollback the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from deployment; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "diff" -d 'Show changes between the local configuration and the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "pull" -d 'Pull the development config file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from push" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from diff" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from diff" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from diff" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from diff" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from diff" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from pull" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from pull" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from pull" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from pull" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from pull" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from lint" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from lint" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from lint" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from lint" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from lint" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "diff" -d 'Show changes between the local configuration and the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "pull" -d 'Pull the development config file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "diff" -d 'Show changes between the local configuration and the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "pull" -d 'Pull the development config file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from dev; and __fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get; and not __fish_seen_subcommand_from help" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get" -l canary -d 'Use the Canary channel API'
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
complete -c wukong -n "__fish_seen_subcommand_from login" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from login" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from completion" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from completion" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from tui" -s a -l application -d 'Override the application name that the CLI will perform the command against. If the flag is not used, then the CLI will use the default application name from the config' -r
complete -c wukong -n "__fish_seen_subcommand_from tui" -s v -l verbose -d 'Use verbos output. More output per occurrence.

By default, it\'ll only report errors.
`-v` show warnings
`-vv` show info
`-vvv` show debug
`-vvvv` show trace'
complete -c wukong -n "__fish_seen_subcommand_from tui" -s q -l quiet -d 'Do not print log message'
complete -c wukong -n "__fish_seen_subcommand_from tui" -l canary -d 'Use the Canary channel API'
complete -c wukong -n "__fish_seen_subcommand_from tui" -s h -l help -d 'Print help'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize Wukong\'s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "application" -d 'This command group contains the commands to interact with an application’s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "pipeline" -d 'This command group contains the commands to view & interact with an application’s pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "deployment" -d 'This command group contains the commands to view and interact with the Continuous Delivery pipeline of an application'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "dev" -d 'This command group contains the commands to interact with the local development environment'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "config" -d 'This command group contains the commands to view & interact with Wukong\'s configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "login" -d 'Login to start using wukong command'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate wukong cli completions for your shell to stdout'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "tui" -d 'Start TUI session'
complete -c wukong -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from login; and not __fish_seen_subcommand_from completion; and not __fish_seen_subcommand_from tui; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances" -f -a "info" -d 'Show the application’s relevant informations'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances" -f -a "logs" -d 'Getting the logs of the applications from the Google Cloud Logging'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and not __fish_seen_subcommand_from info; and not __fish_seen_subcommand_from logs; and not __fish_seen_subcommand_from instances" -f -a "instances" -d 'This command group contains the commands to interact with an application’s instances'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect" -f -a "list" -d 'Listing the currently running Elixir instances, normally under a GKE Pod'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from application; and __fish_seen_subcommand_from instances; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from connect" -f -a "connect" -d 'Start the interactive session to connect to the remote Elixir instance'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "list" -d 'List the current pipelines of the application'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "describe" -d 'Show the details of a pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from pipeline; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from describe; and not __fish_seen_subcommand_from ci-status" -f -a "ci-status" -d 'Show the build status and (possible) errors on branch ci pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "list" -d 'List the current available deployment pipelines of an application'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "execute" -d 'Start the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from deployment; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from execute; and not __fish_seen_subcommand_from rollback" -f -a "rollback" -d 'Rollback the deployment pipeline'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and not __fish_seen_subcommand_from config" -f -a "config" -d 'This command group contains the commands to interact with the config secrets with bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "push" -d 'Push the current configuration changes to the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "diff" -d 'Show changes between the local configuration and the Bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "pull" -d 'Pull the development config file from Bunker'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from dev; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from push; and not __fish_seen_subcommand_from diff; and not __fish_seen_subcommand_from pull; and not __fish_seen_subcommand_from lint" -f -a "lint" -d 'Linting the config and show possible warnings, as well as suggestion how to fix the config file'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "list" -d 'List the configurations'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "set" -d 'Set the value of a configuration'
complete -c wukong -n "__fish_seen_subcommand_from help; and __fish_seen_subcommand_from config; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from get" -f -a "get" -d 'Print the value of a configuration'
