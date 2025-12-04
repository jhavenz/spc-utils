pub fn run_examples() {
    println!(
        r#"Usage Examples:

  Get the latest version:
    spc-utils latest
    spc-utils latest -C common -V 8.4

  Check for updates:
    spc-utils check-update -V 8.4.10

  Download a binary:
    spc-utils download -o php
    spc-utils download -C bulk -V 8.4 -o ./php-bin

  Manage cache:
    spc-utils cache list
    spc-utils cache clear

  Skip cache on any command:
    spc-utils latest --no-cache"#
    );
}
