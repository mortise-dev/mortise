---
title: mortise.json
description: The schema of mortise.json, the file in the app folder that holds the boundary rules, the CI switch and the coverage settings, and how Mortise writes it.
sidebar:
  order: 12
---

`mortise.json` holds the boundary rules of an app, whether a broken rule fails CI, and the coverage
settings the team shares. It sits in the app folder, so a team shares it through git. The window and
the command line tool `mortise check` read the same file. It is the only file Mortise writes in an app
without a command.

## Schema

```json
{
    "rules": {
        "no-circular-use": true,
        "public-surface-only": {
            "enabled": true,
            "folders": ["Contracts", "Events"]
        },
        "allowed-use": {
            "enabled": true,
            "modules": {
                "Audit": [],
                "History": ["Audit", "Billing"]
            }
        }
    },
    "ci": {
        "fail": true
    },
    "coverage": {
        "limit": 70,
        "report": "build/coverage/clover.xml"
    }
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `rules` | object | All rules at their defaults | The boundary rules |
| `rules.no-circular-use` | boolean | `true` | The rule **No circular use**: two modules must not use each other, directly or through others |
| `rules.public-surface-only` | object | `{ "enabled": true, "folders": ["Contracts", "Events"] }` | The rule **Other modules use only a module's public surface** |
| `rules.public-surface-only.enabled` | boolean | `true` | The rule is on |
| `rules.public-surface-only.folders` | array of strings | `["Contracts", "Events"]` | For a module without an accepted public surface, the folders that count as public. A class in one of them may be used by other modules |
| `rules.allowed-use` | object | None: the rule checks nothing | The rule **Allowed use**: the modules each module may use. [nwidart/larch](https://github.com/nWidart/larch) reads the same key |
| `rules.allowed-use.enabled` | boolean | `true` | The rule is on |
| `rules.allowed-use.modules` | object of arrays of strings | `{}` | For each module with a list, the modules it may use. An empty list means no module. A module without a list is not checked |
| `ci` | object | `{ "fail": true }` | What `mortise check` does in CI |
| `ci.fail` | boolean | `true` | **Fail CI when a rule breaks**. On, `mortise check` exits 1 when it finds a problem. Off, it prints the same report and exits 0 |
| `coverage` | object | None | The coverage settings of the app. Each key that is set wins over the setting of the window |
| `coverage.limit` | number, 0 to 100 | None: the **Coverage limit** of Settings, 50% until it is changed | A module covered less than this per cent is a warning |
| `coverage.report` | string | None: the **Coverage report** of Settings, else `phpunit.xml` and the usual places | The clover report, relative to the app folder or absolute |

A folder name matches the first part of the class name after the namespace of the module, ignoring
case. `Contracts\PaymentGateway` is in the folder `Contracts`. A class directly in the namespace of
the module is in no folder.

### The rule allowed use

- A module with a list may use the modules of its list only. Each other module it uses is a problem,
  counted on the module that uses, with each file that does.
- A module that is off breaks no rule, and no link to it does.
- A name in `allowed-use` that is no module of the app, as a module with a list or in a list, is a
  warning. So a typing mistake does not pass in silence.
- The rule says which module. **Public surface only** still says which classes of it.
- Mortise checks the imports it reads without PHP. To check links that only the booted app shows,
  such as facades and container keys, use Larch's `toOnlyDependOn()->via(Edge::all())` in Pest.

The Boundary tab has the switch of this rule and says how many modules have a list. The switch
writes `enabled` and keeps the lists. Edit the lists in the file. Its findings appear in the Boundary
tab, the Findings tab and `mortise check`.

### Where the coverage settings come from

For the coverage limit and for the coverage report, the first of these that is set applies:

1. A flag of `mortise check`: `--coverage-limit` or `--coverage`.
2. The `coverage` object of `mortise.json`.
3. The setting of the window, in **Settings** > **Apps**. `mortise check` has none.
4. The default: 50%, and the report found in `phpunit.xml` or the usual places.

## Reading

| The file | The rules |
|---|---|
| Does not exist | The defaults |
| Has no `rules` key | The defaults |
| Leaves out a key | The default of that key |
| Is not a JSON object, or its `rules` or `ci` cannot be read | The defaults, for the rules, `ci` and `coverage`. The Boundary tab says "mortise.json is not valid: `<message>`. The default rules apply until it is corrected." `mortise check` says the same on standard error |
| Has a `coverage` that cannot be read, or a limit over 100 | The rules and `ci` as read. `coverage` is ignored: the window's settings apply. The Boundary tab says "The coverage block of mortise.json is ignored: `<message>`. Coverage comes from Settings until it is corrected." `mortise check` says so on standard error |

`rules`, `rules.public-surface-only`, `rules.allowed-use`, `rules.allowed-use.modules`, `ci` and
`coverage` must each be a JSON object. A list in their place makes the file invalid. So does a list of
`rules.allowed-use.modules` that is not a list of strings. Larch stops on such a file, where Mortise
uses the defaults.

Mortise ignores the other keys of the file.

## Writing

Mortise writes `mortise.json` only when you change a rule, or **Fail CI when a rule breaks**, in the
Boundary tab. `mortise check` never writes it.

- When the file does not exist, Mortise asks first. The dialog "Create mortise.json in `<app>`?"
  says: "Mortise keeps the boundary rules of `<app>` in mortise.json in the app folder, so a team
  shares them. It is the only file Mortise writes into the app without a command." Nothing is
  written until **Create mortise.json**.
- When the file exists, the change is written at once.
- Mortise replaces the `rules` key and sets `ci.fail`. It keeps `allowed-use` and its lists, with
  the modules by name. It adds no `allowed-use` key to a file that has none. It keeps every other key of the file, such
  as `coverage` and the other keys of `ci`.
- When the file holds something that is not a JSON object, Mortise does not replace it. The save fails.
- The file is indented with four spaces and ends with a line end, like the other JSON files of a Laravel app.
- No secret is ever written to it. Credentials go to the credential store of the operating system.

After a save, Mortise reads the app again. A save that fails shows the toast
"The rules were not saved: `<error>`."

### The save through a temporary file

Mortise never writes `mortise.json` in place.

1. It writes the whole file to a temporary file beside it, in the app folder. The name of the
   temporary file starts with `.mortise.json.`, then random letters, and ends with `.tmp`.
2. It moves the temporary file over `mortise.json` in one step. This replaces the file on Windows too.

So the file is never half written, and no other file of the app is touched. Only a crash during the
save can leave the temporary file behind. This is deviation S28 in `docs/deviations.md`.

On macOS and Linux the temporary file gets the permissions a plain write gives, not those of a
private temporary file.

## Related pages

- [Boundary tab](/mortise/reference/boundary/)
- [How to add boundary rules](/mortise/how-to/add-boundary-rules/)
- [How to check the rules in CI](/mortise/how-to/check-rules-in-ci/)
- [Command line tool](/mortise/reference/command-line/)
- [About boundaries](/mortise/explanation/boundaries/)
