## ADDED Requirements

### Requirement: Paths are written the same on every platform
Every path in the output of `mortise check`, in each format, SHALL be relative to the app folder with `/` between its parts on Linux, macOS and Windows. A path outside the app folder SHALL be written whole, with `/` between its parts and a Windows prefix such as `D:` kept, and SHALL NOT start with two separators.

#### Scenario: A file inside the app on Windows
- **WHEN** `mortise check fixtures/shop-api` runs on Windows
- **THEN** the circle is placed on `Modules/Checkout/app/Listeners/RequestShipment.php:6`, and no line holds a backslash in its path

#### Scenario: A modules folder outside the app
- **WHEN** the modules folder is `D:\shared\Modules` and the app is in `C:\ci\app`
- **THEN** the folder is written `D:/shared/Modules`, and on Unix `/shared/Modules` is written `/shared/Modules`

### Requirement: CI runs the built tool on three platforms
A CI job SHALL build `mortise` alone on Linux, macOS and Windows and run it on the fixture shop-api: `check` in the text, github and json formats, `check` on a folder that is not an app, and `push --dry-run` without `MORTISE_TOKEN`. It SHALL fail when an exit code or the key output differs.

#### Scenario: The fixture shop-api
- **WHEN** the job runs on any of the three platforms
- **THEN** each format exits 1 and names the circle and the three reads of Billing internals, the folder that is not an app exits 3, and the dry run exits 0 with the accepted surface of Users and only relative paths with `/`
