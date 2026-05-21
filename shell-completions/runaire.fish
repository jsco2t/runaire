# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_runaire_global_optspecs
	string join \n format= registry= h/help V/version
end

function __fish_runaire_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_runaire_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_runaire_using_subcommand
	set -l cmd (__fish_runaire_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c runaire -n "__fish_runaire_needs_command" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_needs_command" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_needs_command" -s V -l version -d 'Print version'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "vault" -d 'Vault lifecycle and registration commands'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "entry" -d 'Secret-entry CRUD, search, and TOTP commands'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "gen" -d 'Password and passphrase generation'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "sync" -d 'Synchronise a vault with its configured sync transport'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "ssh" -d 'SSH key entry management'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c runaire -n "__fish_runaire_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "create" -d 'Create a new vault and register it'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "open" -d 'Probe vault unlock with the given master password'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "list" -d 'List registered vaults'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "set-sync" -d 'Configure the sync transport for a vault'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "set-lock" -d 'Configure the per-vault idle-lock timeout'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and not __fish_seen_subcommand_from create open list set-sync set-lock help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l id -d 'Registry name for the new vault (unique)' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l path -d 'Absolute or relative path where the `.kdbx` file will be created' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l keyfile -d 'Optional keyfile required to unlock this vault' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -l no-recovery-warning -d 'Acknowledge the no-recovery warning. Required — there is no master-password recovery in Rùnaire'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from open" -l id -d 'Registry name of the vault to probe' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from open" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from open" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from open" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from list" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from list" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -l id -d 'Registry name of the vault to configure' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -l remote -d 'Remote URL (e.g. `git@github.com:user/vault.git`)' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -l branch -d 'Remote branch name. Defaults to `main` when the implementation lands' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-sync" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -l id -d 'Registry name of the vault to configure' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -l timeout -d 'Idle-timeout in seconds before the vault auto-locks. Must be at least 1' -r
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -l clear -d 'Remove the per-vault override and fall back to the default'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from set-lock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new vault and register it'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "open" -d 'Probe vault unlock with the given master password'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "list" -d 'List registered vaults'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "set-sync" -d 'Configure the sync transport for a vault'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "set-lock" -d 'Configure the per-vault idle-lock timeout'
complete -c runaire -n "__fish_runaire_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "add" -d 'Add a new entry to a vault'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "get" -d 'Get an entry by UUID or title'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "edit" -d 'Edit an existing entry'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "rm" -d 'Remove an entry (move to Recycle Bin by default)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "list" -d 'List entries in a vault'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "search" -d 'Search entries'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and not __fish_seen_subcommand_from add get edit rm list search help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l vault -d 'Registry name of the vault to add into' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l title -d 'Title of the new entry (required)' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l username -d 'Optional username' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l url -d 'Optional URL' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l notes -d 'Optional notes' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l length -d 'Generated-password length (only consulted with `--generate`). Default: 20' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l tag -d 'Tag to attach to the new entry. Repeat for multiple tags' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l password-stdin -d 'Read the entry\'s password from stdin (no-echo when stdin is a TTY). Mutually exclusive with `--generate`'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l generate -d 'Generate a fresh password via `runaire-genpw`. Mutually exclusive with `--password-stdin`. Honours the `PasswordClassFlags` + `--length` flags'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l show-password -d 'Include the generated/captured password in the output view (JSON + human). Default: omit'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l no-lowercase -d 'Disable lowercase letters in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l no-uppercase -d 'Disable uppercase letters in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l no-digits -d 'Disable digits in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l no-symbols -d 'Disable symbols in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -l exclude-ambiguous -d 'Exclude visually ambiguous characters (`0/O/o/1/l/I/|/backtick`)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l vault -d 'Registry name of the vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l uuid -d 'UUID of the entry. Mutually exclusive with `--title`' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l title -d 'Title of the entry; case-insensitive exact match. If multiple entries share the title the command exits 1 listing the candidate UUIDs' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l show-password -d 'Include the password value in the output. Mutually exclusive with `--copy` (copying redundantly with showing is suspicious)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l show-totp -d 'Compute and include the current TOTP code (HMAC-SHA1; RFC 6238)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -l copy -d 'Copy the password to the system clipboard with auto-clear. Mutually exclusive with `--show-password`'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l vault -d 'Registry name of the vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l uuid -d 'UUID of the entry to edit (required)' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l title -d 'New title' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l username -d 'New username' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l url -d 'New URL' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l notes -d 'New notes' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l add-tag -d 'Tag to add. Repeat for multiple' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l rm-tag -d 'Tag to remove (silent no-op if not present). Repeat for multiple' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -l password-stdin -d 'Read a new password from stdin (no-echo when stdin is a TTY)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -l vault -d 'Registry name of the vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -l uuid -d 'UUID of the entry to remove (required)' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -l permanent -d 'Permanently delete the entry, bypassing the recycle bin'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from rm" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l vault -d 'Registry name of the vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l tag -d 'Filter to entries carrying every supplied tag (intersect). Repeat the flag for multiple tags' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l limit -d 'Optional pagination — max rows to emit' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l offset -d 'Optional pagination — rows to skip from the start' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -l include-expired -d 'Include expired entries. Default: omit'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -l vault -d 'Registry name of the vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -l limit -d 'Optional cap on returned matches' -r
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -l include-recycled -d 'Include entries in the recycle bin. Default: exclude'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add a new entry to a vault'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get an entry by UUID or title'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit an existing entry'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "rm" -d 'Remove an entry (move to Recycle Bin by default)'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "list" -d 'List entries in a vault'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search entries'
complete -c runaire -n "__fish_runaire_using_subcommand entry; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -f -a "password" -d 'Generate a random password with selectable character classes'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -f -a "passphrase" -d 'Generate an EFF-large-wordlist diceware passphrase'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and not __fish_seen_subcommand_from password passphrase help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l length -d 'Password length (characters). Default: 20' -r
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l no-lowercase -d 'Disable lowercase letters in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l no-uppercase -d 'Disable uppercase letters in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l no-digits -d 'Disable digits in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l no-symbols -d 'Disable symbols in the generated password'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l exclude-ambiguous -d 'Exclude visually ambiguous characters (`0/O/o/1/l/I/|/backtick`)'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l copy -d 'Copy the value to the clipboard with auto-clear. Mutually exclusive with `--show` (copy implies do-not-print)'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -l show -d 'In JSON mode, include the generated value in the output. Default JSON output omits the value; human-mode output always prints the value to stdout regardless of this flag (mirror `pbcopy` style)'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from password" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l word-count -d 'Number of words. Default: 6' -r
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l separator -d 'Separator inserted between words. Default: `-`' -r
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l copy -d 'Copy the value to the clipboard with auto-clear. Mutually exclusive with `--show`'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -l show -d 'In JSON mode, include the generated value in the output'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from passphrase" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from help" -f -a "password" -d 'Generate a random password with selectable character classes'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from help" -f -a "passphrase" -d 'Generate an EFF-large-wordlist diceware passphrase'
complete -c runaire -n "__fish_runaire_using_subcommand gen; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l vault -d 'Vault registry name to sync' -r
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l branch -d 'Override the configured sync branch' -r
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l remote -d 'Override the configured sync remote URL' -r
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand sync" -l dry-run -d 'Show what would be pushed/pulled without writing anything'
complete -c runaire -n "__fish_runaire_using_subcommand sync" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -f -a "add" -d 'Add an SSH-key entry to a vault. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -f -a "load" -d 'Load an SSH key into ssh-agent with TTL. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -f -a "generate" -d 'Generate a new SSH keypair and store the private key. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and not __fish_seen_subcommand_from add load generate help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -l vault -d 'Registry name of the destination vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -l key-path -d 'Path to the existing private key to import' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -l comment -d 'Optional comment to attach to the entry' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -l vault -d 'Registry name of the source vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -l uuid -d 'UUID of the SSH-key entry to load' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -l ttl -d 'TTL in seconds before ssh-agent expires the key' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from load" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -l vault -d 'Registry name of the destination vault' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -l algorithm -d 'Algorithm (`ed25519` or `rsa`). Defaults to `ed25519` when implemented' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -l comment -d 'Optional comment to attach to the public key' -r
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from generate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add an SSH-key entry to a vault. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from help" -f -a "load" -d 'Load an SSH key into ssh-agent with TTL. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from help" -f -a "generate" -d 'Generate a new SSH keypair and store the private key. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand ssh; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand completions" -l format -d 'Output format. `human` (default) is line-oriented for terminals; `json` is the stable machine-readable schema for scripts' -r -f -a "human\t'Line-oriented human-readable output (default)'
json\t'Stable JSON schema — see `views/` for per-subcommand shapes'"
complete -c runaire -n "__fish_runaire_using_subcommand completions" -l registry -d 'Path to the vault registry file. Defaults to `$HOME/.local/state/runaire/vaults.toml` (per `runaire_core::RunairePaths`)' -r -F
complete -c runaire -n "__fish_runaire_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "vault" -d 'Vault lifecycle and registration commands'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "entry" -d 'Secret-entry CRUD, search, and TOTP commands'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "gen" -d 'Password and passphrase generation'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "sync" -d 'Synchronise a vault with its configured sync transport'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "ssh" -d 'SSH key entry management'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c runaire -n "__fish_runaire_using_subcommand help; and not __fish_seen_subcommand_from vault entry gen sync ssh completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "create" -d 'Create a new vault and register it'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "open" -d 'Probe vault unlock with the given master password'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "list" -d 'List registered vaults'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "set-sync" -d 'Configure the sync transport for a vault'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "set-lock" -d 'Configure the per-vault idle-lock timeout'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "add" -d 'Add a new entry to a vault'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "get" -d 'Get an entry by UUID or title'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "edit" -d 'Edit an existing entry'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "rm" -d 'Remove an entry (move to Recycle Bin by default)'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "list" -d 'List entries in a vault'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from entry" -f -a "search" -d 'Search entries'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from gen" -f -a "password" -d 'Generate a random password with selectable character classes'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from gen" -f -a "passphrase" -d 'Generate an EFF-large-wordlist diceware passphrase'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from ssh" -f -a "add" -d 'Add an SSH-key entry to a vault. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from ssh" -f -a "load" -d 'Load an SSH key into ssh-agent with TTL. (Slot — see `features/ssh-keys/`.)'
complete -c runaire -n "__fish_runaire_using_subcommand help; and __fish_seen_subcommand_from ssh" -f -a "generate" -d 'Generate a new SSH keypair and store the private key. (Slot — see `features/ssh-keys/`.)'
