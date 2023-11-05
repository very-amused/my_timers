# my_timers
### My ***T***hreaded, ***I***nformation-***M***ultiplexing ***E***vent ***R***unner for ***S***QL

my_timers is a multithreaded, async MariaDB/MySQL event runner, made out of frustration
with the verbose syntax and poor logging capabilities of the included `event_scheduler`.

[![AUR version](https://img.shields.io/aur/version/my_timers?label=AUR&logo=archlinux&style=flat-square)](https://aur.archlinux.org/packages/my_timers)

## Installation
- `# make install` - Install to /usr/local/bin (default)
- `# PREFIX=/usr make install` - Install to /usr/bin

## Configuration
There are two main files used to configure my_timers, both of them have a configurable location via environment variables:
- `$MY_TIMERS_CONFIG` (default: `./config.json`): Database connection and logging options.
- `$MY_TIMERS_EVENTS` (default: `./events.conf`): Event definitions.

### config.json
There are two top level keys in `config.json`: `db` and `log`. `db` configures how my_timers connects to a MariaDB/MySQL database,
while `log` configures how my_timers records event runs via logs/traces.

```jsonc
{
  "db": {
    // Name of the database user my_timers will try to connect as
    "user": "admin",

    // Password for the database user
    // (optional)
    "password": "$0mething_v3ry--53cur3",

    // Connection protocol used to access the database
    // (optional, values: "SOCKET"|"TCP", default: "SOCKET")
    "protocol": "TCP",

    // Address of the database server.
    // Must be an IP address or domain name if `protocol` is "TCP",
    // or the path of a Unix socket if `protocol` is "SOCKET"
    // (optional, default: "/var/run/mysqld/mysqld.sock")
    "address": "db1.myapp.net",

    // Name of the database to connect under
    "database": "MyDatabase",

    // Whether to use TLS when connecting (optional, default: false)
    "tls": true
  },

  "log": {
		// Global log format, overrides per-destination format options.
		// (optional, values: "normal", "pretty", "compact", "json", default: None)
		"format": "pretty",

    // Output to logfile(s), with optional periodic rotation
    "file": {
      // All log destinations can be enabled/disabled with the `enabled` option.
      // If a destination config section is present with no `enabled` option specified,
      // the destination will be enabled
      // (optional, default: true)
      "enabled": true,

			// Log output format
			// (optional, values: "default", "pretty", "compact", "json", default: "default")
			"format": "default",

			// Colorized logging output using ANSI escape codes.
			// Use `less -R` or a `less -R` analog to display log files with color
			// (optional, default: false)
			"color": false,

      // The absolute path of the file logs will be written to.
      // If `rotation` is "never", this is the path of a single file that all logs will be written to.
      // Otherwise, this path serves as a prefix for the creation of rotated log files over time
      // i.e /var/log/myapp/my_timers.log.YYY-MM-DD-HH.
      // Due to this behavior, it is highly recommended that specified log directories are used
      // to avoid flooding system directories like /var/log
      "path": "/var/log/myapp/my_timers.log",

      // The frequency at which output is moved to a newly created logfile
      // (optional, values: "never"|"daily"|"hourly"|"minutely", default: "daily")
      "rotation": "never"
    },

    // Output to stdout/stderr
    "stdio": {
      // (optional, default: true)
      "enabled": true,

			// Log output format
			// (optional, values: "default", "pretty", "compact", "json", default: "pretty")
			"format": "default",

			// Colorized logging output using ANSI escape codes.
			// (optional, default: false)
			"color": false,

      // The output stream logs will be written to
      // (optional, values: "stdout"|"stderr", default: "stdout")
      "stream": "stderr"
    }
  }
}
```

### events.conf
Event configuration syntax is [sxhkd](https://github.com/baskerville/sxhkd)-like.
Comments begin with `#`, any text on the line after this character is ignored.

Events are composed of a *name*, an *interval*, and a *body*:
```
# Comments can be placed above events or in the event bodies.
# Comments above intervals are the recommended placement due to clarity.
name:
interval
  some sql statement;
  another sql statement;

# Same-line interval definitions are permitted but not recommended due to clarity.
name: interval
  statement;
```

#### Name
A clear, concise description of the event's function/purpose.

#### Interval
Cron syntax specifying when the event will run. The non-standard, optional `@startup` suffix
can be used to cause an event to run when my_timers starts, in addition to its cron interval.

#### Body
An event's body is composed of SQL statement(s) to be executed when the event runs. Each line in an event's body must be indented with a minimum of 1 tab or 2 spaces,
unindented lines will be interpreted as the beginning of new events. SQL statements are semicolon-terminated and may span multiple lines (as long as each line is indented).

**NOTE:** Events are run on single MariaDB/MySQL transactions; no changes will be committed unless
*all* statements in the event execute successfully. Therefore, it is safe to write statements that depend on each other.

#### Examples:

```
# Clear auth sessions older than 2 weeks (and not currently in use,
# decided by whether the token has been active within the past hour)
Expire inactive sessions:
0 * * * *
  DELETE FROM Sessions
    WHERE UNIX_TIMESTAMP() - Created_Timestamp >= (14 * 24 * 60 * 60)
    AND UNIX_TIMESTAMP() - LastUsed_Timestamp >= (60 * 60);

# Clear unverified TOTP entries after 1 minute
Clear unverified TOTP:
* * * * * @startup
  DELETE FROM TOTP WHERE Verified = 0 AND UNIX_TIMESTAMP() - Created_Timestamp > 60;

# Reset users who have changed their email but not verified the new email within 1 week to their old email
Reset unverified email changes:
0 * * * * @startup
	UPDATE Users, EmailVerifyTokens SET Users.Email = Users.OldEmail, Users.Verified = 1
		WHERE Users.ID = EmailVerifyTokens.UserID
			AND UNIX_TIMESTAMP() - EmailVerifyTokens.Created_Timestamp >= (7 * 24 * 60 * 60)
			AND Users.OldEmail IS NOT NULL;
	DELETE FROM EmailVerifyTokens WHERE UNIX_TIMESTAMP() - Created_Timestamp >= (7 * 24 * 60 * 60);
	UPDATE Users SET OldEmail = NULL WHERE Email = OldEmail AND Verified = 1;
```
