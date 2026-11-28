## ADDED Requirements

### Requirement: The window asks for a newer version at most once a day
With the setting "Updates" at "At start", the window SHALL ask at start for the update manifest of the newest release, unless the last check ended less than 24 hours ago. The request SHALL carry no path, no name and no token. With the setting at "Off" the window SHALL make no request by itself. A check at start that fails SHALL show nothing and SHALL count as the check of that day.

#### Scenario: The first start of a day
- **WHEN** the window starts, the setting is "At start" and no check ended in the last 24 hours
- **THEN** it asks for the manifest once and keeps the time and the newer version, if any, in `update.json`

#### Scenario: A second start within the day
- **WHEN** the window starts again within 24 hours of a check that found a newer version
- **THEN** it makes no request and offers that version again

#### Scenario: The setting is off
- **WHEN** the window starts and the setting is "Off"
- **THEN** it makes no request, and "Check now" in General still asks

### Requirement: A newer version is offered and nothing is downloaded before the user asks
When a check finds a newer version, the window SHALL show a toast that stays until it is closed, with the version and the button "Install and restart", and the same button in General. The window SHALL download nothing before the button is pressed. A check the user started with "Check now" SHALL show its answer in General, also when it is "This is the newest version." or a failure.

#### Scenario: A newer version at start
- **WHEN** the check at start finds 9.0.0
- **THEN** the toast says "Mortise 9.0.0 is available." with "Install and restart", and nothing was downloaded

### Requirement: Only a download with the signature of the update key is installed
"Install and restart" SHALL download the newer version, verify its signature with the public key built into the app, replace the app and start it again. A download whose signature does not verify, or whose signed file name is not that of the offered version and this platform, SHALL be discarded, and the window SHALL say that it was not installed.

#### Scenario: The right signature
- **WHEN** the download carries the signature of the key
- **THEN** the app is replaced and started again, and the window closes

#### Scenario: Another content under the same signature
- **WHEN** the server gives a file that the signature of the manifest does not fit
- **THEN** nothing is installed and the window says the download does not carry the signature of Mortise

#### Scenario: An older signed file under a newer version
- **WHEN** the manifest offers 9.5.0 with the file and the signature of 9.0.0
- **THEN** nothing is installed

### Requirement: An install and a command never run together
The install SHALL wait while a command runs in an app. While the app is replaced, no command SHALL start, and closing the window SHALL wait until the replacing has ended.

#### Scenario: A command runs
- **WHEN** the download ends while a command runs in an app
- **THEN** General says the update installs when the command ends, and it installs then

#### Scenario: A command is asked for during the install
- **WHEN** the user starts a command while the app is replaced
- **THEN** the command does not start and a toast says to run it after the restart

### Requirement: A copy that cannot replace itself gets a link
A `.deb` install, a build from source, a macOS app that runs from a translocated place, and an app in a folder the user cannot write without an administrator SHALL get the message with the button "Open the download page" in place of "Install and restart". The button SHALL open the page of the newest release in the browser.

#### Scenario: The .deb package
- **WHEN** a check finds a newer version in a window installed from the `.deb` package
- **THEN** the toast and General offer "Open the download page" and no install

### Requirement: A release publishes signed updates and their manifest
A release SHALL sign the packed macOS app, the Windows setup and the AppImage with the update key, verify each signature with the public key of the source, and publish `latest.json` with the version and, per platform, the address, the signature and the format. A release SHALL fail before it publishes when the key is missing or a signature does not verify. The update key SHALL reach one job only, and that job SHALL build nothing of the app.

#### Scenario: The key is missing
- **WHEN** a release runs and the secret of the update key is empty
- **THEN** `tools/sign-updates.sh` fails and no release is published

#### Scenario: The key is not the one of the source
- **WHEN** the files are signed with a key whose public half is not `PUBLIC_KEY`
- **THEN** `tools/sign-updates.sh` fails and no release is published
