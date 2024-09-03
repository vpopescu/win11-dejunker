Extensible CLI to dejunk Windows 11.
There are no warranties, either expressed or implied. Use at your own risk.


Supports changing settings from command line, or from file.  Settings are defined in db.yaml

## Command line

```bash
Usage: dejunker.exe [OPTIONS]

Options:
  -i, --input-file <input rules file>
          Settings file to be applied
  -o, --output-file <output rules file>
          Write settings to this file
  -s, --database-file <rules database>
          Database file (definitions of known settings) [default: db.yaml]
      --win-tailored-experience-with-diagnostic-data=<on|off>
          Tailored experiences based on diagnostic data [possible values: on, off]
      --win-start-menu-show-ads=<on|off>
          Ads (recommendations) in start menu [possible values: on, off]
      --win-tips-and-suggestions=<on|off>
          Tips and suggestions when using windows [possible values: on, off]
      --win-get-more-from-windows-suggestion=<on|off>
          The 'Get even more out of windows' suggestion' [possible values: on, off]
      --edge-shopping-assistant=<on|off>
          Microsoft edge shopping assistant [possible values: on, off]
      --win-tips-on-lock-screen=<on|off>
          Get fun facts, tips, tricks, and more on your lock screen [possible values: on, off]
      --win-windows-web-search=<on|off>
          Web search as part of windows search [possible values: on, off]
      --win-notifications-suggestions=<on|off>
          Suggestions about disabling some notifications [possible values: on, off]
      --win-suggested-content-in-settings=<on|off>
          Show suggested content in settings app [possible values: on, off]
      --win-web-widget-allowed=<on|off>
          Tips and suggestions when using windows [possible values: on, off]
      --win-sync-provider-notifications=<on|off>
          Notifications about getting a better experience [possible values: on, off]
  -h, --help
          Print help
  -V, --version
          Print version
```

### File mode examples (local file or url)


1. Create an input file:

dejunker -o file.yaml

2. Apply an input file:

dejunker -i file.yaml

3. Use alternate rules db:
Generally it is assumed that the rules file is in current directory, named db.yaml.

dejunker -s /elsewhere/rules.yaml -o file.yaml


