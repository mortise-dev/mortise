## MODIFIED Requirements

### Requirement: A read keeps running totals for 24 hours within a stated bound
Mortise SHALL keep, per route with its method, per job class and per `module` label, running totals of the request count, the count of 5xx answers, the request time, the bucket counts and the job count. It SHALL keep one read per 15 s of the last hour, the newest read always, and one read per 5 minutes of the 23 hours before. It SHALL drop a read older than 24 hours. It SHALL keep at most 520 reads, however often it reads, so memory stays flat once 24 hours are kept. The totals SHALL take at most 520 × subjects × (3 + bucket bounds) × 8 bytes.

#### Scenario: 24 hours of reads
- **WHEN** 7,000 reads arrive 15 s apart for the same series
- **THEN** no more than 520 reads are kept, and the number does not grow after read 5,760

#### Scenario: A burst of reads
- **WHEN** "Try again" makes reads 1 s apart for two hours
- **THEN** no more than 520 reads are kept, and the rate of the last minute is unchanged

#### Scenario: The memory of a full window
- **WHEN** 26 hours of reads of an app with many routes and jobs arrive 15 s apart by a fake clock
- **THEN** the memory the store holds stays under the bound the test states from the number of subjects and bounds, and does not grow after read 5,760

## ADDED Requirements

### Requirement: The switches of the tab fit its width
The environment switch and the range switch of the Metrics tab SHALL stay inside the width of the tab, whatever the number and the length of the environment names. Every environment SHALL stay reachable by pointer and by keyboard, and SHALL keep its full name as its accessible name.

#### Scenario: Many long environment names
- **WHEN** an app has eight environments with names of 30 characters and the tab is 900 px wide
- **THEN** no segment of the switches is drawn past the right edge of the tab, and each environment can be chosen
