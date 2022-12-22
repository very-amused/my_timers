# my_timers
### My ***T***hreaded, ***I***nformation ***M***ultiplexing ***E***vent ***R***unner for ***S***QL

## Configuration
There are two main files used to configure `my_timers`, both of them have a configurable location via environment variables:
- `$MY_TIMERS_CONFIG` (default: `./config.json`): Database connection and logging options.
- `$MY_TIMERS_EVENTS` (default: `./events.conf`): Event definitions.

### config.json
There are two top level keys in `config.json`: `db` and `log`. `db` configures how my_timers connects to a MariaDB/MySQL database,
while `log` configures how my_timers records event runs via logs/traces.
```json
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
    // Output to logfile(s), with optional periodic rotation
    "file": {
      // All log destinations can be enabled/disabled with the `enabled` option.
      // If a destination config section is present with no `enabled` option specified,
      // the destination will be enabled
      // (optional, default: true)
      "enabled": true,

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

      // The output stream logs will be written to
      // (optional, values: "stdout"|"stderr", default: "stdout")
      "stream": "stderr"
    }
  }
}
```

### events.conf

This project is not currently production ready.
