//! The place of a series in a module, as decision 2 of the plan says.
//!
//! Placing runs each time figures are worked out, on the files and the `mappings` of now. So what is
//! kept is never placed by old routes, and the worker needs no snapshot.

use std::collections::HashMap;

use crate::app::AppSnapshot;
use crate::metrics::sample::Subject;
use crate::runtime::actuator::Route;
use crate::runtime::live::module_of;

/// The methods of one route and its module.
type Placed<'a> = (&'a [String], Option<&'a str>);

/// Places subjects in the modules of one app.
pub(crate) struct Placing<'a> {
    app: &'a AppSnapshot,
    /// `/` + uri → the routes with that path.
    routes: HashMap<String, Vec<Placed<'a>>>,
}

impl<'a> Placing<'a> {
    /// `routes` is `None` when `mappings` is not exposed. Then no request is placed by its route.
    pub(crate) fn new(app: &'a AppSnapshot, routes: Option<&'a [Route]>) -> Self {
        let mut by_path: HashMap<String, Vec<_>> = HashMap::new();
        for route in routes.unwrap_or_default() {
            let module = module_of(app, &route.action).map(|module| module.name.as_str());
            by_path
                .entry(format!("/{}", route.uri.trim_start_matches('/')))
                .or_default()
                .push((route.methods.as_slice(), module));
        }
        Self {
            app,
            routes: by_path,
        }
    }

    /// The name of the module, or `None` for "Not in a module". The first rule that applies wins.
    pub(crate) fn place(&self, subject: &Subject) -> Option<&'a str> {
        // `Reading::of` already turned an empty label into no label.
        let label = match subject {
            Subject::Request { module, .. } | Subject::Job { module, .. } => module.as_deref(),
            Subject::Database { module } => Some(module.as_str()),
        };
        if let Some(label) = label {
            return self.named(label);
        }
        match subject {
            Subject::Request {
                route: Some(route),
                method,
                ..
            } => self.by_route(route, method),
            Subject::Job { class, .. } => {
                module_of(self.app, class).map(|module| module.name.as_str())
            }
            Subject::Request { route: None, .. } | Subject::Database { .. } => None,
        }
    }

    /// A `module` label: a module name in any case, else "Not in a module". `none` is never a module.
    fn named(&self, label: &str) -> Option<&'a str> {
        if label.eq_ignore_ascii_case("none") {
            return None;
        }
        self.app
            .modules
            .iter()
            .find(|module| module.name.eq_ignore_ascii_case(label))
            .map(|module| module.name.as_str())
    }

    /// The module of the routes with this path and method, or with this path when none has the
    /// method. `None` when they are in more than one module, or no route has the path.
    fn by_route(&self, route: &str, method: &str) -> Option<&'a str> {
        let routes = self.routes.get(route)?;
        let holds =
            |methods: &[String]| methods.iter().any(|held| held.eq_ignore_ascii_case(method));
        let any_method = !routes.iter().any(|(methods, _)| holds(methods));
        let mut places = routes
            .iter()
            .filter(|(methods, _)| any_method || holds(methods))
            .map(|(_, module)| *module);
        let first = places.next()?;
        places.all(|place| place == first).then_some(first)?
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::*;
    use crate::metrics::figures::{LAST_MINUTE, Rates};
    use crate::metrics::sample::{LiveMetrics, MetricsState};
    use crate::runtime::actuator::NoMetrics;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");

    fn recorded(name: &str) -> String {
        std::fs::read_to_string(Path::new(FIXTURES).join("actuator").join(name)).unwrap()
    }

    fn request(module: Option<&str>, route: &str, method: &str) -> Subject {
        Subject::Request {
            module: module.map(str::to_owned),
            route: Some(route.to_owned()),
            method: method.to_owned(),
        }
    }

    #[test]
    fn the_recorded_answers_are_placed_in_their_modules() {
        let mut app = crate::read_app(&Path::new(FIXTURES).join("shop-api")).unwrap();
        let mut blog = app.modules[0].clone();
        blog.name = "Blog".to_owned();
        blog.namespace = r"Modules\Blog".to_owned();
        app.modules.push(blog);
        let mappings: serde_json::Value = serde_json::from_str(&recorded("mappings.json")).unwrap();
        let routes: Vec<Route> = serde_json::from_value(mappings["routes"].clone()).unwrap();
        let reads = Arc::new(AtomicU32::new(0));
        let metrics = LiveMetrics::with_clock(Arc::new(move || {
            Duration::from_secs(15) * reads.fetch_add(1, Ordering::Relaxed)
        }));
        let mut sink = metrics.sink();
        sink(Ok(recorded("prometheus-quiet.txt")));
        sink(Ok(recorded("prometheus-busy.txt")));

        let figures = metrics.figures(&app, Some(&routes), LAST_MINUTE);
        assert_eq!(figures.covered, Some(Duration::from_secs(15)));
        // What 15 s counted, per minute.
        let rates = |requests: f64, jobs: f64| Rates {
            requests_per_minute: requests * 4.0,
            jobs_per_minute: jobs * 4.0,
        };
        let rows: Vec<(&str, Rates)> = figures
            .modules
            .iter()
            .map(|row| (row.module.as_str(), row.rates.clone()))
            .collect();
        let zero = Rates::default();
        assert_eq!(
            rows,
            [
                ("Blog", rates(57.0, 4.0)),
                // `/portal` is a route of Billing on its own domain.
                ("Billing", rates(26.0, 4.0)),
                ("Catalog", zero.clone()),
                ("Checkout", zero.clone()),
                ("Inventory", zero.clone()),
                ("Notifications", zero.clone()),
                ("Shipping", zero.clone()),
                ("Users", zero.clone()),
            ]
        );
        // `/`, `/hello/{name}`, `/status`, `/up`, the 404 and the 405.
        assert_eq!(figures.outside, rates(20.0, 0.0));
        let counted: f64 = recorded("prometheus-busy.txt")
            .lines()
            .filter(|line| line.starts_with("http_server_request_duration_seconds_count"))
            .filter_map(|line| line.rsplit(' ').next()?.parse::<f64>().ok())
            .sum();
        let placed: f64 = rows
            .iter()
            .map(|(_, rates)| rates.requests_per_minute)
            .sum();
        assert_eq!(placed + figures.outside.requests_per_minute, counted * 4.0);

        // A read with no metrics ends a read and adds nothing.
        sink(Err(NoMetrics::Off));
        assert_eq!(
            metrics.state(),
            MetricsState {
                last: Some(Err(NoMetrics::Off)),
                serial: 3,
            }
        );
        assert_eq!(metrics.figures(&app, Some(&routes), LAST_MINUTE), figures);

        // Written input.
        let placing = Placing::new(&app, Some(&routes));
        assert_eq!(
            placing.place(&request(Some("billing"), "/blog", "GET")),
            Some("Billing")
        );
        assert_eq!(placing.place(&request(Some("NONE"), "/blog", "GET")), None);
        assert_eq!(
            placing.place(&request(Some("Reports"), "/blog", "GET")),
            None
        );
        // No route of `/blog` takes POST, so every route of the path counts.
        assert_eq!(placing.place(&request(None, "/blog", "POST")), Some("Blog"));
        let mut two_portals = routes.clone();
        two_portals.push(Route {
            methods: vec!["GET".to_owned(), "HEAD".to_owned()],
            uri: "portal".to_owned(),
            name: None,
            action: r"Modules\Blog\Http\Controllers\PostController@portal".to_owned(),
            middleware: Vec::new(),
        });
        let placing = Placing::new(&app, Some(&two_portals));
        assert_eq!(placing.place(&request(None, "/portal", "GET")), None);
        let without_routes = Placing::new(&app, None);
        assert_eq!(without_routes.place(&request(None, "/blog", "GET")), None);
        let job = Subject::Job {
            module: None,
            class: r"Modules\Blog\Jobs\PublishPost".to_owned(),
        };
        assert_eq!(without_routes.place(&job), Some("Blog"));
    }
}
