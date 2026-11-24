## ADDED Requirements

### Requirement: The table shows five figures for each module over the chosen range
When connected, the Metrics tab SHALL show a table with the columns of S7.3: Module, Requests/min, 95% under, Errors, Jobs/min, Database time and Last hour. It SHALL have one row per module of the app, most requests first, then a row "Not in a module". Each figure SHALL be worked out over the chosen range. No figure SHALL have a colour or a limit.

#### Scenario: Two reads 15 s apart
- **WHEN** Billing counts 10 more requests and 2 more jobs in the second read than in the first
- **THEN** the Billing row reads 40 requests and 8 jobs per minute

### Requirement: Old figures are not shown
Figures SHALL be shown only while the newest read is at most 45 s old. Else the tab SHALL read "Reading metrics from {environment}" until a new read ends.

#### Scenario: The tab opens again
- **WHEN** the tab showed figures, closed for five minutes, and opens again
- **THEN** it reads "Reading metrics from local" and shows no old figure until the next read ends

### Requirement: The 95% figure is a bucket bound
"95% under" SHALL be the smallest finite bucket bound of the request histogram at or under which at least 95% of the requests of the row in the range fall. Mortise SHALL NOT guess a value between two bounds. When more than 5% of the requests are over the largest finite bound, the figure SHALL read "over" that bound. With no request in the range the figure SHALL be an en dash.

#### Scenario: The recorded busy answer
- **WHEN** `prometheus-quiet.txt` and then `prometheus-busy.txt` are read 15 s apart
- **THEN** the 95% figure of each module is the first bound whose bucket holds at least 95% of the module's requests

#### Scenario: Slow requests
- **WHEN** 10 requests of a module are all slower than 10 s, the largest finite bound
- **THEN** its 95% figure reads "over 10 s"

### Requirement: Errors are the share of requests with a 5xx status
"Errors" SHALL be the share of the requests of the row in the range that were answered with a status from 500 to 599. With no request in the range the figure SHALL be an en dash.

#### Scenario: Three 500 answers in 200 requests
- **WHEN** a module counts 197 answers of 200 and 3 of 500 in the range
- **THEN** its Errors figure reads "1.5%"

### Requirement: Database time is known only with the module label
"Database time" SHALL be the time of database operations with a `module` label of the row against the request time of the row. When no kept series has a `module` label, every row SHALL read "unknown" with the tooltip "Needs a newer nwidart/laravel-modules" (not in the spec). A route SHALL show an en dash for database time and for jobs.

#### Scenario: An app without the label
- **WHEN** the recorded busy answer is read twice
- **THEN** every Database time cell reads "unknown", and its tooltip reads "Needs a newer nwidart/laravel-modules"

#### Scenario: An app with the label
- **WHEN** Billing spends 2 s in database operations labelled `module="billing"` and 8 s in requests in the range
- **THEN** its Database time reads "25%"

### Requirement: Three ranges over the same reads
The Metrics tab SHALL offer the ranges 15 min, 1 hour and 24 hours, with 1 hour chosen first. Switching the range SHALL keep the reads and SHALL change every figure, the chart and "now". A range that reaches beyond the last hour SHALL use the thinned reads as the requirement "A span beyond the last hour is right for the time it covers" says.

#### Scenario: From 1 hour to 15 min and back
- **WHEN** the user chooses 15 min, then 1 hour, after twenty reads
- **THEN** no read is dropped, and the figures of 1 hour are the same as before

### Requirement: One module lists its routes
With a module selected in the index, the chart SHALL show that module and the table SHALL list a row per route of the module, by method and route template, with the same figures. A route that no kept read holds SHALL NOT be listed.

#### Scenario: Billing selected
- **WHEN** Billing is selected and its routes `/billing/invoices` and `/portal` were asked for
- **THEN** the chart is titled "Requests per minute, Billing" and the table lists both routes

### Requirement: The chart follows the selected row
The chart SHALL show the requests per minute between neighbouring kept reads over the chosen range, with straight segments in 1.5 px ink on a hairline grid, and no segment across a gap. It SHALL show the selected row, else everything in the scope. J, K, Down, Up, Home and End SHALL move the selected row while the table has focus. The chart SHALL have an accessible name and a text summary.

#### Scenario: J moves the chart
- **WHEN** the table has focus on Catalog and the user presses J
- **THEN** the next row is selected and the chart is titled with its name

#### Scenario: The newest point
- **WHEN** the newest read gave a rate of 842 requests per minute for the scope
- **THEN** the chart box reads "842 now"

### Requirement: The tab says how old the last read is
The Metrics tab SHALL read "read every 15 s, last read N s ago", where N counts up between reads by the clock of the reads.

#### Scenario: Between two reads
- **WHEN** the newest read is 5 s old by the clock of the reads
- **THEN** the tab reads "last read 5 s ago", and "last read 6 s ago" one second later with no new read

## REMOVED Requirements

### Requirement: The table shows requests and jobs per minute for each module
**Reason**: S7.3 is built in full. The table has seven columns and its figures are over the chosen range, not the last minute.
**Migration**: See "The table shows five figures for each module over the chosen range".
