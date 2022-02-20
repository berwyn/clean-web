# CleanWeb

Twitter keeps appending bullshit tracking params to URLs, so I wrote a quick little daemon to fix it.

This is an absolute kludge, please don't use this as an example of Windows programming. Use it at your own peril.

## Configuration
Configuration is kept in `%APPDATA%\CleanWeb\config\config.csv`. The application expects a CSV file in the format of
`host_regex,param_regex`.

### Defaults
| host regex    | parameter regex |
| :------------ | :-------------- |
| `.*`          | `utm_.*`        |
| `twitter.com` | `s\|t`          |
