# retro

Retro game catalog management.

## Development

### Logging

Logs can be emitted at the following levels: `error`, `warn`, `info`, and
`debug`. When each level is sent to stdout depends on the log level specified
when invoking `retro`.

`error` logs are sent to stdout by default. These should be used anytime the
command takes an action (e.g., a symlink is created). This includes dry runs of
commands. They should also be used to report actual errors.

`warn` logs should be used when an action is skipped (e.g., a symlink already
exists).

`info` logs should be used to indicate that no action was attempted (e.g., a
system is not configured).

`debug` logs should be used sparingly as there may be dependencies that use this
level of logging.
