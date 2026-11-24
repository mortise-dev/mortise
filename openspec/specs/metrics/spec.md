# metrics Specification

## Purpose
Defines how Mortise reads the metrics of a running app, what it keeps in memory, how it turns counters into rates, how it places a series in a module, and what the Metrics tab shows for each answer.

## Requirements

### Requirement: The Prometheus text format is read in the core
`mortise-core` SHALL read the Prometheus text format itself, with no new dependency: families of type counter, gauge and histogram, labels, and the escapes `\\`, `\"` and `\n` in a label value. It SHALL skip a family of any other type and a sample with no `# TYPE` line before it. A line it cannot read SHALL be skipped and SHALL NOT fail the read.

#### Scenario: The recorded busy answer
- **WHEN** `fixtures/actuator/prometheus-busy.txt` is read
- **THEN** the job label reads `Modules\Blog\Jobs\PublishPost` with single backslashes, a bucket with `le="+Inf"` has an infinite bound, and `_sum` and `_count` are told apart from buckets

#### Scenario: A family of another type
- **WHEN** an answer holds a family of type `summary` between two histograms
- **THEN** its lines are skipped and both histograms are read

#### Scenario: A bad line
- **WHEN** one line of an answer has no value, a label value with no closing quote, two labels with no comma between them, a timestamp that is not a whole number, or a word after the timestamp
- **THEN** that line is skipped and every other line is read

### Requirement: Metrics are kept in memory only
Mortise SHALL keep what it reads from `prometheus` in memory. It SHALL NOT write it to disk, to `mortise.json` or to `settings.json`.

#### Scenario: After a read
- **WHEN** the Metrics tab has shown the table for an environment
- **THEN** the config folder and the data folder of Mortise hold the same files with the same content as before

### Requirement: A read keeps running totals for 24 hours within a stated bound
Mortise SHALL keep, per route with its method, per job class and per `module` label, running totals of the request count, the count of 5xx answers, the request time, the bucket counts and the job count. It SHALL keep one read per 15 s of the last hour, the newest read always, and one read per 5 minutes of the 23 hours before. It SHALL drop a read older than 24 hours. It SHALL keep at most 520 reads, however often it reads, so memory stays flat once 24 hours are kept.

#### Scenario: 24 hours of reads
- **WHEN** 7,000 reads arrive 15 s apart for the same series
- **THEN** no more than 520 reads are kept, and the number does not grow after read 5,760

#### Scenario: A burst of reads
- **WHEN** "Try again" makes reads 1 s apart for two hours
- **THEN** no more than 520 reads are kept, and the rate of the last minute is unchanged

### Requirement: A span beyond the last hour is right for the time it covers
A span that reaches beyond the last hour SHALL start at a kept read at most 5 minutes inside it. Its rate SHALL divide by the time covered between the two reads it uses, so the rate is right for the time it covers. The totals and the covered time SHALL carry on across a gap, however long.

#### Scenario: A range that reaches into the thinned part
- **WHEN** figures are asked for a span of 24 hours of reads 15 s apart
- **THEN** they cover at least 23 hours 55 minutes and at most 24 hours, and equal the figures worked out from every read over that time, because a kept read holds totals and not differences

#### Scenario: A gap longer than an hour
- **WHEN** reads come at 285 s and 300 s, then at 4,000 s and 4,005 s, and figures are asked for the last 3,710 s
- **THEN** they cover the 5 s from 4,000 s to 4,005 s

### Requirement: Two reads of one run always give a rate
Mortise SHALL keep the first read of each run within the last hour, even when the next read comes within the same 15 s. Two reads of one run SHALL always give a rate.

#### Scenario: Two reads within one pace
- **WHEN** after a gap, "Try again" makes a second read 1 s after the first
- **THEN** the second read gives a rate over that second

### Requirement: A read keeps at most 32,768 numbers and 4 MiB of labels
A read SHALL keep at most 32,768 numbers: 3 and one per bucket bound, for each series of the kept reads. It SHALL keep at most 4 MiB (4,194,304 bytes) of labels and subject strings for those series. A series that no read of the last 24 hours holds SHALL NOT count. A read over either SHALL be refused as a whole and SHALL add nothing.

#### Scenario: An answer over the cap
- **WHEN** an answer holds 11,000 job series, 3 numbers each
- **THEN** nothing of it is kept, and the Metrics tab reads "The metrics of local have too many series" with "Try again"

#### Scenario: Long labels
- **WHEN** an answer holds one job whose `laravel_job_name` is 2 MiB long
- **THEN** nothing of it is kept, and the tab reads as for too many series

#### Scenario: Old series age out
- **WHEN** an answer was refused because the kept reads hold other series, and the reads that hold them are now older than 24 hours
- **THEN** the next answer that fits on its own is kept

### Requirement: A counter that goes down starts a new rate
Mortise SHALL treat a counter that is lower than at the read before as a reset. That read SHALL add nothing for the series and SHALL be the start of its next rate. Each status of a route SHALL be a series of its own: a reset of one status SHALL NOT hide the growth of another. A counter that a read does not give, or gives as a line it cannot read, SHALL add nothing and SHALL keep its value of the read before. Only counters that both reads give SHALL decide a reset. No rate SHALL be negative.

#### Scenario: The store of the app is cleared
- **WHEN** a route counts 100, then 3, then 5 requests in three reads 15 s apart
- **THEN** the second read adds 0, the third adds 2, and no figure is below zero

#### Scenario: A reset of one status
- **WHEN** a route counts 100 answers of 200 and 100 of 404, then 110 of 200 and 1 of 404
- **THEN** the second read adds 10 for the route

#### Scenario: A missing `_sum`
- **WHEN** a route counts 10 requests in 5 s, then 12 requests with no `_sum` line, then 14 requests in 7 s
- **THEN** the second read adds 2 requests and no time, and the third adds 2 requests and 2 s

### Requirement: Buckets match by their bound
A bucket SHALL add to the kept bucket of the same bound, in whatever order a read gives the bounds. A bucket that a read does not give SHALL be a counter that the read does not give. A bound that no read gave before SHALL widen what is kept and SHALL empty nothing. A read whose new bounds take what is kept over the cap SHALL be refused as for too many series. A bound that no read of the last 24 hours gave SHALL be dropped and SHALL no longer count towards the cap.

#### Scenario: Buckets in another order, or one missing
- **WHEN** a route counts 4 requests with buckets `le="0.1"` and `le="1"`, then 5 with the same buckets in the other order, then 6 with no `le="0.1"` line
- **THEN** each read adds 1 request and the buckets it gives, and nothing kept is emptied

#### Scenario: A new bound
- **WHEN** a read gives a bucket `le="0.5"` that no read gave before
- **THEN** every kept row gains that bucket at zero, and the request rate goes on

#### Scenario: The app changes its bounds
- **WHEN** a route is read every few minutes for a day, and its reads give `le="2"` in place of `le="0.1"`, `le="0.5"` and `le="1"`
- **THEN** once the last read that gave the old bounds is older than 24 hours, only `le="2"` is kept and the route keeps its rate

### Requirement: No rate is worked out across a gap or from one read
Mortise SHALL work out no rate from the first read. It SHALL work out no rate across a gap: two reads more than 45 s apart, such as while the tab was closed or the window hidden. The read after a gap SHALL start again as a first read, and SHALL give no rate while it is the newest read. A series that is missing from that read SHALL add nothing when it returns. A rate SHALL divide by the time between reads that Mortise did cover.

#### Scenario: The tab was closed for an hour
- **WHEN** the tab closes at 1,000 requests and opens an hour later at 61,000 requests
- **THEN** the 60,000 requests in between add to no figure, and the first rate comes with the second read after the tab opened

#### Scenario: A read after a gap shows no rate
- **WHEN** reads come at 0 s and 15 s, then at 61 s
- **THEN** the read at 61 s gives no rate, not the rate of 0 s to 15 s

#### Scenario: A series that returns after a gap
- **WHEN** a route counts 2 before a gap, is missing from the first read after it, and counts 50 in the read after that
- **THEN** its return adds nothing, and the read after adds what it grew by from 50

#### Scenario: The first read
- **WHEN** one read exists
- **THEN** the table lists the modules with no number, and says that a rate needs two reads

### Requirement: A series that appears counts from zero
A series that is in a read and was in no kept read before SHALL count from zero when the read before it is less than a gap away. A series that is missing from a read SHALL keep its totals and add nothing.

#### Scenario: The first 5xx of a route
- **WHEN** a route with 200 answers gets its first three 500 answers between two reads
- **THEN** the later read adds three requests and three errors for that route

### Requirement: A module label places a series first
Mortise SHALL place a series as decision 2 of the plan says. A label `module` wins over every other rule. The value `none` is "Not in a module". A value that equals a module name without regard to case is that module. Any other value is "Not in a module".

#### Scenario: The module label
- **WHEN** a series has `module="billing"` and an `http_route` of a Blog controller
- **THEN** it counts for Billing

#### Scenario: A label that names no module of the app
- **WHEN** a series has `module="none"`, or `module="Reports"` in an app with no module Reports
- **THEN** it counts as "Not in a module"

### Requirement: Without the label a request is placed by its route and a job by its class
Without a `module` label, Mortise SHALL place a request by its `http_route` and method through the last `mappings` answer, where `http_route` equals `/` followed by the `uri` without leading slashes. It SHALL place a job by the namespace of its `laravel_job_name`. A series that matches nothing, or routes of more than one module, SHALL count as "Not in a module".

#### Scenario: A route of a module
- **WHEN** a series has `http_route="/billing/invoices/{invoice}"` and method `GET`, and `mappings` gives `billing/invoices/{invoice}` to a controller of Billing
- **THEN** the series counts for Billing

#### Scenario: A job of a module
- **WHEN** a series has `laravel_job_name="Modules\\Billing\\Jobs\\ChargeInvoice"`
- **THEN** the series counts for Billing

#### Scenario: One path in two modules
- **WHEN** two routes of `mappings` have the uri `portal` and the method `GET`, on two domains, in two modules
- **THEN** the series `http_route="/portal"` counts as "Not in a module"

#### Scenario: A request that matched no route
- **WHEN** a series has no `http_route` label, as a 404 has
- **THEN** it counts as "Not in a module"

#### Scenario: mappings is not exposed
- **WHEN** the app does not expose `mappings`
- **THEN** requests without a `module` label count as "Not in a module", jobs are still placed, and the table says why

### Requirement: Switching the environment empties the reads
Mortise SHALL keep the reads of the chosen environment only. Choosing another environment, or changing its credentials, SHALL drop every read.

#### Scenario: From local to staging
- **WHEN** the user chooses staging after ten reads of local
- **THEN** the table of staging starts with no read

### Requirement: An answer is read off the interface thread within a stated budget
Mortise SHALL read the text of an answer on the worker thread, not on the thread that draws. An answer for 60 modules of 10 routes each (about 3.3 MB) SHALL be read into totals within 50 ms in a release build, and SHALL allocate less than 2 MB beside the text.

#### Scenario: A generated answer for 60 modules
- **WHEN** a generated answer with 600 routes and 60 jobs is read in a debug build
- **THEN** the read ends within 1 s and gives 660 series

### Requirement: The Metrics tab says what the environment answered
The Metrics tab SHALL NOT show the S7.4 block. It SHALL show S7.1 when no environment is connected, and S7.2 when the environment answers and does not list `prometheus`. When the environment gives no answer, refuses the credentials or answers with an unexpected status, it SHALL show the sentence the runtime box shows, with "Try again" and "Open settings". Else it SHALL show the table.

#### Scenario: Not connected
- **WHEN** shop-api has no chosen environment and the Metrics tab opens
- **THEN** the tab reads "Metrics are not connected" with the paragraph of S7.1, and "Connect an environment" opens Settings on the environments of shop-api

#### Scenario: Not exposed, then exposed
- **WHEN** local does not list `prometheus`, the tab reads "Metrics are not exposed on local", the app's team adds the endpoint, and the user presses "Try again"
- **THEN** Mortise asks for the base path again and the tab shows the table

#### Scenario: No answer
- **WHEN** nothing listens at the address of local
- **THEN** the tab shows the sentence of the runtime box, and "Open settings" opens Settings on that environment

### Requirement: An answer without metrics has its own sentence
For a 503 from `prometheus` the Metrics tab SHALL quote the first line of the body as the app wrote it. For an empty answer it SHALL say in one sentence that metrics are off in the app. For an answer over the body limit it SHALL name the limit. For an answer with more series than a read keeps it SHALL name the most numbers a read keeps. Each SHALL offer "Try again". These sentences are not in the spec. A failed read after good ones SHALL show its sentence, not an old table.

#### Scenario: A 503
- **WHEN** `prometheus` answers 503 with `The metrics store cannot be read.`
- **THEN** the tab quotes that line and offers "Try again"

#### Scenario: Metrics are off
- **WHEN** `prometheus` answers 200 with an empty body
- **THEN** the tab reads "Metrics are off on local" and names the setting that turns them on

#### Scenario: Too large
- **WHEN** `prometheus` answers with more than 16 MiB of text
- **THEN** the tab says the answer is larger than 16 MB and that Mortise does not read it

### Requirement: An answer that is not metrics is refused
Mortise SHALL refuse a 200 from `prometheus` whose text has no `# TYPE` line, and SHALL keep nothing of it. An answer with only families that Mortise does not read SHALL be kept. The Metrics tab SHALL say that the answer is not metrics, with "Try again" and "Open settings". This sentence is not in the spec.

#### Scenario: A sign-in page
- **WHEN** `prometheus` answers 200 with a sign-in page
- **THEN** nothing of it is kept, and the tab reads "The answer of local is not metrics" with "Try again" and "Open settings"

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
