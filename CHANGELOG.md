# Changelog

All notable changes to this project are listed here.
<!-- git-cliff: end of header -->
## [1.1.0] - 2026-11-26

### Features

- *(core)* Check the allowed use rule of mortise.json (#76)
- Make room for feedback (issue forms, Report a problem, request rule) (#78)
- Draw the allowed use rule in the window (milestone 30) (#79)

### Documentation

- Name laravel-modules 13.6.0 as the release with the module tag (#74)
- Harden phase 5 (tasks 33.2 to 33.4) (#80)
- Name larch 0.2.0 as the release that reads allowed-use (#83)

### Other

- Move the repository to the GitHub organisation `mortise-dev`. The links in the app, the docs and the release files point there (#81)
- Start the window on a Windows runner in CI and keep what it shows (#77)

## [1.0.1] - 2026-11-26

### Features

- *(app)* Follow the reduced motion setting on windows and linux (#68)
- *(core)* Place scheduled tasks by their module tag (#69)

### Bug fixes

- *(app)* Keep the environment switch and its focus while metrics are read (#67)
- *(app)* Give columns of figures way to the name in a narrow metrics table (#71)
- *(app)* Keep the palette from restarting the input method in each frame (#72)

### Build

- Move to Rust 1.100.0 and mago 1.61.0 (#66). Building Mortise or `cargo install mortise-cli` now needs Rust 1.100

### Documentation

- Mark phase 4 done in the plan (#62)
- Fix the milestone numbers of phase 4 in the phase 3 plan (#64)
- Add the phase 5 plan (#65)
- Record the linux sweep of the deviation rows (#70)

## [1.0.0] - 2026-11-26

### Features

- Add the command line rule check (milestone 24) (#51)
- *(cli)* Add the push document and mortise push (#53)
- Accept a public surface for a module (milestone 27) (#57)

### Documentation

- Add the documentation site (#48)
- Add start here and three how-to guides (#49)
- Add the phase 4 plan (#50)
- Archive the phase 4 command line check change (#52)
- Archive the phase 4 push change (#54)
- Archive the phase 4 distribution change (#56)
- Archive the phase 4 surface change (#58)
- Archive the phase 4 hardening change (#60)

## [0.2.0] - 2026-11-25

### Features

- *(core)* Place a scheduled task in the module that declares its command
- *(app)* Show the module of an artisan task on the summaries
- *(core)* Work out the runtime status of each module from health
- *(app)* Show the runtime mark of each module in the index
- *(app)* Show the runtime of a module in its summary
- *(core)* Read prometheus while the metrics tab is open
- *(core)* Read the prometheus text format
- *(core)* Keep metrics reads in memory with rates and resets
- *(core)* Place metrics in modules and work out rates per module
- *(app)* Show metrics per module on the metrics tab
- *(core)* Work out the figures of the metrics tab per scope and range
- *(app)* Draw the metrics chart and sparkline with the painter
- *(app)* Draw the metrics tab of s7.3 with ranges, routes and a selected row

### Bug fixes

- *(app)* Move interface text into strings files
- *(app)* Keep a long component detail beside its name in the runtime box
- *(app)* Keep one title across tabs and indent json in the runner
- *(app)* Bound json depth, clear the scroll bar and elide the header folder
- *(core)* Accept an artisan path and scan only the tasks of the module
- Keep a failed modules check and give each module indicator its own line
- *(core)* Take the modules component only in the shape of laravel-modules
- Keep every detail of a down module indicator and take an empty key
- *(core)* Find resets per series and bound what a read keeps
- *(core)* Skip a label line with no comma between two labels
- *(core)* Key a series by its labels and find a bound in constant time
- *(core)* Bound what the metrics store keeps and show no old rates
- Start no metrics read after the tab closes
- *(core)* Match buckets by bound and carry the totals over a long gap
- *(core)* Read the metrics body only for a 200 or a 503
- *(app)* Close the metrics tab from the state at the end of the frame
- *(core)* Skip a sample line with a bad timestamp or a word after it
- *(app)* Say that only requests without a module label are not placed
- *(core)* Refuse a metrics answer with no type line as not metrics
- *(core)* Drop a bucket bound once no kept read gave it
- *(app)* Give the chart a step of 1 for a subnormal highest point
- *(core)* Carry each bucket up to the bounds a series lacks before the 95% bound
- *(core)* Break the line between reads of different runs
- *(app)* Give routes the stamp column and the whole name of a cut row on hover
- *(app)* Wrap the range switch and the read line when the top of the metrics tab is too narrow
- *(spec-build)* Write inside the repository or stop
- *(app)* Draw one point per pixel column in a sparkline
- *(app)* Wrap the environment switch when the names do not fit
- *(spec-build)* Refuse symlinks and unchecked folders, use a unique temporary file
- *(app)* Scroll to the segment the arrow keys choose
- *(spec-build)* Refuse a link at the output folder, compare paths in one convention, encode file URLs
- *(spec-build)* Replace pages by rename, check the height, write paths with no line end
- *(app)* Scroll to a segment once it has focus in its new place
- *(app)* Keep one sparkline point per pixel column across a gap

### Performance

- *(core,app)* Build the range line for the total and the selected row only

### Refactoring

- *(app)* Move text two features show to the shared strings
- *(app)* Move the text left in views into strings.rs

### Documentation

- Propose the phase 3 groundwork change
- Add prometheus to the actuator rule
- Bring spec 2.5, 12.2 and S7.2 up to the released packages
- Describe how metrics are placed without a module label
- Add the worktree rule for agents
- Archive the phase 3 groundwork change
- Propose the phase 3 module tasks change
- Say which scheduled tasks s4.2 places before p10
- Archive the phase 3 module tasks change
- Propose the runtime per module change
- Record the deviations of the runtime per module
- Archive the phase 3 runtime per module change
- Propose the phase 3 reading metrics change
- Keep the bounds when a read has no request histogram
- Record how the metrics store takes its first bounds
- Record the measured fold of a 60 module answer
- Record how metrics are placed and what the metrics tab shows
- Record bases per series, stale bases, fine thinning and the cap of a read
- State the metrics caps, absent counters, run starts, closing the tab and old rates
- State matching buckets by bound, totals over long gaps, closing the tab, thinned spans and error bodies
- Archive the phase 3 reading metrics change
- Propose the phase 3 metrics tab
- Record the 95% bucket bound and what the metrics tab adds to s7.3
- Record the range line of the selected row and three declined review findings
- Archive the phase 3 metrics tab change
- Propose the phase 3 hardening change
- Record the image fixes and the keyboard gap of the waiting state
- Say how to run the real actuator check and record what serve passes
- Tick the real actuator job
- Describe phase 3 in the readme and record what the build did in the plan
- *(core)* Drop an intra-doc link to a private constant
- Mark the fixes that belong to no milestone in the phase 3 plan
- Record the linux look of phase 3 and the second title bar of hyprbars
- Archive the phase 3 hardening change

## [0.1.0] - 2026-11-23

### Features

- *(core)* Read an app folder into a snapshot
- *(app)* Show the workbench with the real module list
- *(core)* Keep the app list and read an app again when its files change
- *(app)* Add apps, keep them between runs and follow changes on disk
- *(core)* Read lines, kinds, tests, coverage and git changes
- *(app)* Show statistics on the Summary and filter the index
- *(core)* Find links between modules, circles and the rules of an app
- *(app)* Show the boundary of a module and of the app
- *(core)* Work out findings from rules and checks on files
- *(app)* Show findings in the index, on the rail and in a list
- *(core)* Add the palette matcher, the editor command and more settings
- *(app)* Add the command palette
- *(app)* Add Settings, the editor command and the window widths
- *(app)* Show focus on every control and honour reduced motion
- *(core)* Read runtime status through laravel-actuator
- *(app)* Show runtime status, routes and scheduled tasks
- *(core)* Scan a module and propose its public surface
- *(app)* Show the public surface of a module and scan it
- *(app)* Complete the keyboard model
- Read a folder off the interface thread and say how far it is
- *(app)* Set the index width, reorder the rail, show shortcuts in tooltips
- *(app)* Follow the reduce motion setting of macOS
- *(app)* Add the menu bar of macOS
- *(app)* Add the app icon for macOS, Windows and Linux
- *(app)* Name the module that counts a circle in the summary of the other members
- *(core)* Find herd, sail and docker from the app folder
- *(core)* Read APP_ENV from .env beside APP_URL
- *(core)* Build an artisan command line for each kind of runner
- *(core)* Keep the runner, working folder and time limit of each app
- *(core)* Test how php runs with artisan --version
- *(app)* Choose and test how php runs in settings
- *(app)* Choose how php runs when an app is added
- *(core)* Map sgr codes of command output to four styles
- *(core)* Keep the last 50 runs of each app
- *(core)* Run a command with streamed output, stop and a time limit
- *(app)* Show runs in the runner drawer
- *(core)* Plan the everyday actions and their confirmations
- *(core)* Read the migration lines of a run
- *(app)* Add the actions tab
- *(app)* List the actions of a module in the palette
- *(core)* Count the files that are not committed in a module
- *(core)* Plan the destructive actions and check the typed name
- *(core)* Read a prohibited command and count rollbacks
- *(app)* Offer the destructive actions in a closed group
- *(app)* Confirm destructive actions with a typed name
- *(core)* Describe the 42 generators of 13.3.0
- *(core)* Build the words of a generator and check names
- *(core)* Read the files a generator run wrote
- *(app)* Build the generate tab, its runs and its ways in
- *(app)* List the new module and generate keys in settings
- *(core)* Wait for the end of a run with a deadline
- *(core)* Map the runs of the history to recent actions and generators
- *(app)* List recent actions in the empty palette
- *(app)* List the runner keys in settings and copy the command with Ctrl Shift C in Actions
- *(app)* List the run again, history and copy output keys in settings
- *(core)* Add the package data groundwork of 13.4
- *(app)* Keep the package data of an app across its reads
- *(app)* Start and poll reads of package data beside runs
- *(app)* Read the migration state and count migrations in the actions
- *(app)* Run module:test and read its counts on a 13.4 app
- *(app)* Show translation warnings and work to do in findings
- Read the generators of a 13.4 app and keep them in a cache
- List the files of a dry run and read the answer of a generator run
- *(app)* Offer list files again beside a file list

### Bug fixes

- *(core)* Say an unknown host and an untrusted certificate in plain words
- *(app)* Show the name from Settings in the index header
- *(core)* Say the host was not found on Windows too
- *(app)* Write the home path with / on every platform
- *(app)* Ship the font licences in the installers
- *(core)* Do not follow redirects from the actuator
- *(core)* Read a file again when its test folder changes
- *(core)* Watch the folder of the statuses file the config names
- *(core)* See a commit that changes only the refs and the git log
- *(core)* Run git without a console window on windows
- *(core)* Save the rules through a temporary file with a unique name
- *(app)* Open a circle on a module its carrier uses
- *(app)* Show a one-way link in a circle as part of the circle
- *(app)* Add only the folder the path field shows
- *(app)* Close settings first on esc and keep only saved field values
- *(app)* Alert a problem of the app itself with its own severity
- *(app)* Key rail tiles by app folder so equal names do not clash
- *(app)* Give the index strip a scrollbar track and keep its focus ring inside
- *(app)* Wrap the findings states so they stay inside their stat cell
- *(core)* Name where a redirect of the actuator sends to
- *(core)* Watch a renamed statuses file and folders that appear later
- *(app)* Keep ctrl o and the rules dialog to the app it was asked for
- *(app)* Let esc leave a settings field before it closes settings
- *(app)* Close settings when a command or menu item goes somewhere
- *(app)* Open a link or a finding on the links of boundary
- *(app)* Keep the click that leaves an edited settings field
- *(app)* Wrap the dependency names so they stay clear of the counts
- *(app)* Show a circle in the files of a link only while its rule is on
- *(app)* Draw the neighbours that break a rule before the more node
- *(core)* Watch a folder wanted twice with its subfolders
- *(app)* Wrap a long test answer on its own lines
- *(app)* Leave settings cleanly from the menu bar and the add panel
- *(app)* Keep a long test answer to two lines and repaint after a menu choice
- *(core)* Leave test files out of links
- *(app)* Draw the code stamp in a graph node and the name only when it fits
- *(app)* Scroll the add panel when it does not fit
- *(core)* Count a rule finding only when its modules are on
- *(core)* Name no carrier of a circle through a module that is off
- *(app)* Show the settings name of an app on every screen
- *(app)* Scroll the add panel to the control that gets focus
- *(app)* Scroll the add panel only for a control inside it
- *(core)* Count a circle of modules that are on when an off module joins it
- *(app)* Draw the add panel's bar only once the files are counted
- *(core)* Name a missing working folder in the php test
- *(app)* Show the line that the php test runs
- *(core)* Take the last line of a name in .env
- *(core)* Read a docker line with an empty service as custom
- *(core)* Add each artisan flag only once
- *(core)* Quote a program with = in the posix line
- *(core)* Start a relative program with a separator in the folder
- *(core)* Refuse a batch file with % in its path on windows
- *(core)* Bound the output a php test keeps
- *(app)* Keep the status of the runner still when the run time changes
- *(core)* Kill the whole group when a run ends and bound the line queue
- *(core)* Lock the history while a run is kept and prune only its own logs
- *(app)* Keep the room of the status and make mortise the default binary
- *(app)* Run again only a line that is on screen
- *(core)* Read the lines a reader still sends after a run ends
- *(app)* Give the fake artisan of the app its own name
- *(core)* Count rolled back migrations and drop the database of migrate all
- *(core)* Keep a dry run dry and say which commands run again asks for
- *(core)* Drop an unused binding in plan
- *(app)* Settle the review of the actions tab
- *(app)* Check the typed name again when the button is pressed
- *(app)* Settle the review of the destructive actions
- *(app)* Check refusals from the app's files when a destructive dialog confirms
- *(core)* Keep --factory when the model file is replaced
- *(app)* Settle the review of the generate tab
- *(app)* Say the dry run runs none of the sql
- *(app)* Draw the harpoon of laravel output
- *(app)* Stop every run when mortise closes
- *(app)* Keep graph nodes, add buttons and box heads inside their room
- *(app)* Wrap the labels of the rules inside the rules box
- *(core)* Leave out recent actions the actions tab does not offer
- *(app)* Keep the hint of the current palette row
- *(app)* Say open on a current palette row with no hint
- *(app)* Scroll a focused graph node into view
- *(app)* Grow each box by its wrapped head
- *(core)* Honour a stop while the readers drain
- *(app)* Keep a palette row's detail clear of its hint
- *(core)* Start a stop that is pending when the readers drain
- *(app)* Scroll the page to a focused graph node too
- *(core)* Start a pending stop even when the pipes closed
- *(app)* Pass the gate to the options of a migration panel
- *(core)* Replace only the model file when the model passes --force on
- *(app)* Start a read only from a line that was whole on screen
- *(app)* Drop a generator read when the package version changed while it ran
- *(core)* Mark a cut line of standard output and never read it as a document
- Count an option only before -- when words get --json or --pretend
- *(app)* Turn off run again in the drawer while a read runs
- *(app)* Say why a run gave no answer when it read no file
- *(core)* Read created tables through use aliases and the named table argument
- *(app)* List files and add --json only for generators the package ships
- *(core)* Offer no replace for a generator the package does not ship
- *(core)* Offer no replace of a 13.4 model whose run made its factory
- *(app)* Keep the factory in the command when the dry run finds it
- *(core)* Place a container path of a generator under the app folder
- *(app)* List the files s9.3 took out in the result with their replace
- *(core)* Count only the schema facade when a migration imports another schema
- *(core)* Check each name of an array argument of a generator
- *(app)* Keep why a failed autoload result was read from the output
- *(app)* Offer replace in the own-file sentence only when the form offers it
- *(core)* Keep a module named like a flag in the words that replace a model
- Read the runner's own options in the guards of replace
- *(core)* Place a container path by a module of the app, else by the last modules folder
- *(app)* Mark the factory box exists when the dry run keeps the factory
- *(app)* Give each file a run left its own replace in its row
- *(app)* Open a generator without a module argument under all modules
- *(app)* Drop the file list when the runner or the working folder changes
- *(core)* Offer no model replace when the runner sets a model option
- *(app)* Drop a stale dry run before it edits the form and restore its flags
- *(app)* Keep the other rows of a result when one left file is replaced
- *(app)* Let only the generator's own file stop create
- *(core)* Place the paths of a new module by the names module:make asks for
- *(app)* Keep the path of a result row clear of its state and replace
- *(core)* Place a container path by the first part of the run's own module
- *(app)* Update a replaced row only by its own path and keep the reason of a failed replace
- *(app)* Turn off the replace buttons of a result while the app is busy
- *(app)* Give back the flags of a dropped file list when the words change
- *(app)* Give back the flags a run took out of the form when it ends
- *(app)* Clear the reason of a failed replace when a later replace answers
- *(app)* Read the migration state only from a line whole on screen
- *(app)* Show and read the migration state of the panel's connection
- *(app)* Read the state after a run only when the panel showed that read
- *(app)* Forget the migration state of a connection after a fresh
- *(app)* Take in the test and translation documents of every run
- *(app)* Count no tests when a module that ran has no counts
- *(app)* List at most six migrations or tables in a confirmation
- *(app)* Forget the migration state a migration command changed instead of reading it again
- *(app)* Show the counts of all modules only when every module that is on was read
- *(app)* Give no count toast to a failed test run that counts no failed test
- *(app)* Say when the translation report of a run cannot be read
- *(core)* Cover only the modules that were on at a read of every module
- *(core)* Add no translation check when no module was checked
- *(app)* Count the tests of every module in the run tests row on 13.4
- *(app)* Show a failed state read only for its target and connection
- *(app)* Forget the migration state when the runner or working folder changes
- *(core)* Stop doc comments linking to private items

### Refactoring

- *(core)* Add the run module tree and the run feature
- *(core)* Start git and the editor through one command setup
- *(core)* Name the reading of the two lines of .env
- *(core)* Pass the time limit to a run and share the output limit
- *(app)* Move the alert box and the empty state to draw.rs
- *(app)* Move selection to index.rs
- *(app)* Rename the boundary action to work action and move it to workspace.rs
- *(app)* Move the theme and the components to ui/
- *(app)* Move each screen to its feature folder
- *(app)* Move the window, its chrome and the workspace to app/
- *(app)* Move the keys to app/keys.rs
- *(app)* Move the generate wiring to app/generate.rs
- *(app)* Move the actions wiring to app/actions.rs
- *(app)* Move the runs wiring to app/runs.rs
- *(app)* Move the apps wiring to app/apps.rs
- *(app)* Move the settings wiring to app/settings.rs
- *(app)* Move the palette wiring to app/palette.rs
- *(app)* Move the runtime wiring to app/runtime.rs
- *(core)* Move starting a program to program.rs
- *(core)* Move the reading of modules to modules/
- *(core)* Move target to modules/target.rs
- *(core)* Move the app read and its watch to app/
- *(core)* Move links and rules to boundary/
- *(core)* Move the runtime to runtime/
- *(core)* Move the actions plan to actions/
- *(core)* Move the generators catalog to generate/
- *(core)* Split the output readers into actions and generate
- *(core)* Move what the package ships to the generate catalog
- *(core)* Move the palette's matcher and recent runs to palette/
- *(core)* Import command in the runner only with the run feature
- *(app)* Move the text of actions to actions/strings.rs
- *(app)* Move the text of app to app/strings.rs
- *(app)* Move the text of apps to apps/strings.rs
- *(app)* Move the text of boundary to boundary/strings.rs
- *(app)* Move the text of findings to findings/strings.rs
- *(app)* Move the text of generate to generate/strings.rs
- *(app)* Move the text of index to index/strings.rs
- *(app)* Move the text of palette to palette/strings.rs
- *(app)* Move the text of runner to runner/strings.rs
- *(app)* Move the text of runtime to runtime/strings.rs
- *(app)* Move the text of settings to settings/strings.rs
- *(app)* Move the text of summary to summary/strings.rs

### Documentation

- Add the phase 0 design specification
- Add the phase 1 plan and the list of deviations
- Record the owner's decisions and what milestone 1 found
- Record what milestone 6 moved to milestone 9
- Describe phase 1, list third-party licences, keep differing images in CI
- Set up openspec and add instructions for coding agents
- Name the actuator base path among the requests
- Add orchestration instructions to agents.md
- Add the approved phase 2 plan
- Propose the phase 2 groundwork change
- Let mortise run package commands the user starts
- Bring spec section 12.2 up to the packages
- Name the prohibit flags of the destructive commands
- Mark the phase 2 groundwork tasks done
- Keep the 13.3 generator fallback at 42 commands
- Say that a module of unknown state counts as on
- Archive the phase 2 groundwork change
- Propose the phase 2 how php runs change
- Tick the core groundwork tasks of how php runs
- Add the missing working folder to the how php runs design
- Tick the settings and palette tasks of how php runs
- Say how mortise runs php in the readme
- Tick the add panel, readme and image tasks of how php runs
- Tick the integration tasks of how php runs
- *(core)* Say that a batch file still gets the legacy arguments
- Record the bounds of a php test in the design
- Archive the phase 2 how php runs change
- Propose the phase 2 runner drawer change
- Mark the core tasks of the runner drawer done
- Record the header of the runner drawer as a deviation
- Mark the runner drawer change built
- Record the review fixes of the process model in the design
- Archive the phase 2 runner drawer change
- Propose the phase 2 everyday actions change
- Count rolled back migrations and the options of all modules in the design
- Record the deviations of the actions
- Mark the everyday actions change built
- Record the review decisions of the actions
- Archive the phase 2 everyday actions change
- Propose the phase 2 destructive actions change
- Tick the git count task
- Tick the core tasks of the destructive actions
- Record the deviations of the destructive actions
- Archive the phase 2 destructive actions change
- Propose the phase 2 generate change
- Tick the core tasks of the generate change
- Record the deviations of the generate screens
- Tick the interface tasks of the generate change
- Record what replace module does and the review of the generate tab
- Archive the phase 2 generate change
- Propose the phase 2 hardening change
- Record that the owner accepts deviation S40
- Mark the small fixes of the hardening change done
- Mark the fixes of the hardening change done
- Say how to run the real php tests
- Mark the real php task of the hardening change done
- Mark the accessibility task of the hardening change done
- Describe phase 2 in the readme
- Check deviations s32 to s50 against the build
- Name the phase 2 plan and the real php tests in the readme
- Record the linux look of the hardening change
- Mark the images task of the hardening change done
- Mark the integration of the hardening change done
- Say run none of it in every drawn dry run of the spec
- Archive the phase 2 hardening change
- Propose the phase 2 package data change
- Note the app plumbing of reads in the design
- Record the deviations of the actions with package data and describe 13.4 in the readme
- Record the deviations of the generate package data
- Note what changed after the review of the generate package data
- Record the third review round in s55 and the design notes
- Record the fourth review round in s55, s57 and the design notes
- Say what changed after the review of the migration state and the documents
- Say in the proposal that a migration command forgets the state
- Archive the phase 2 package data change
- Split the json answer out of the result requirement of module generators
- Propose the phase 2 package by feature change
- Name the cycle of catalog and run in the proposal
- Add readme banner and social preview
- Name the feature folders and the strings rule in agents.md
- Update the moved paths in the deviations
- Add milestone 17 to the phase 2 plan
- Record what the package by feature build did and tick its tasks
- Use the measured sizes in the milestone 17 plan
- Archive the phase 2 package by feature change

