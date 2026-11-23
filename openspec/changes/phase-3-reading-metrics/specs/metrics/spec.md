# Spec Delta

## ADDED Requirements

### Requirement: The Prometheus text format is read in the core
`mortise-core` SHALL read the Prometheus text format itself, with no new dependency: families of type counter, gauge and histogram, labels, and the escapes `\\`, `\"` and `\n` in a label value. It SHALL skip a family of any other type and a sample with no `# TYPE` line before it. A line it cannot read SHALL be skipped and SHALL NOT fail the read.

#### Scenario: The recorded busy answer
- **WHEN** `fixtures/actuator/prometheus-busy.txt` is read
- **THEN** the job label reads `Modules\Blog\Jobs\PublishPost` with single backslashes, a bucket with `le="+Inf"` has an infinite bound, and `_sum` and `_count` are told apart from buckets

#### Scenario: A family of another type
- **WHEN** an answer holds a family of type `summary` between two histograms
- **THEN** its lines are skipped and both histograms are read

#### Scenario: A bad line
- **WHEN** one line of an answer has no value, a label value with no closing quote, or two labels with no comma between them
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

#### Scenario: A range that reaches into the thinned part
- **WHEN** figures are asked for a span of 24 hours
- **THEN** they equal the figures worked out from every read, because a kept read holds totals and not differences

### Requirement: A read keeps at most 32,768 numbers
A read SHALL keep at most 32,768 numbers: 3 and one per bucket bound, for each series of the kept reads. A read over that SHALL be refused as a whole and SHALL add nothing.

#### Scenario: An answer over the cap
- **WHEN** an answer holds 11,000 job series, 3 numbers each
- **THEN** nothing of it is kept, and the Metrics tab reads "The metrics of local have too many series" with "Try again"

### Requirement: A counter that goes down starts a new rate
Mortise SHALL treat a counter that is lower than at the read before as a reset. That read SHALL add nothing for the series and SHALL be the start of its next rate. Each status of a route SHALL be a series of its own: a reset of one status SHALL NOT hide the growth of another. No rate SHALL be negative.

#### Scenario: The store of the app is cleared
- **WHEN** a route counts 100, then 3, then 5 requests in three reads 15 s apart
- **THEN** the second read adds 0, the third adds 2, and no figure is below zero

#### Scenario: A reset of one status
- **WHEN** a route counts 100 answers of 200 and 100 of 404, then 110 of 200 and 1 of 404
- **THEN** the second read adds 10 for the route

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

### Requirement: The table shows requests and jobs per minute for each module
When connected, the Metrics tab SHALL show a table with the columns Module, Requests/min and Jobs/min: one row per module of the app, most requests first, then a row "Not in a module". A figure SHALL be the rate over the reads of the last minute. No figure SHALL have a colour or a limit.

#### Scenario: Two reads 15 s apart
- **WHEN** Billing counts 10 more requests and 2 more jobs in the second read than in the first
- **THEN** the Billing row reads 40 requests and 8 jobs per minute
