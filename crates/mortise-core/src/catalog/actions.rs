//! The actions of a module and when Mortise asks before it runs one.
//!
//! [`plan`] turns an action, its target and its options into the words of an
//! artisan command. It also says whether Mortise asks first, as the table of S10.3 says.

/// An action that Mortise can run for a module or for all modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    TurnOn,
    TurnOff,
    Migrate,
    RollBack,
    Seed,
    /// The package has no command for the tests of one module yet.
    RunTests,
    CheckTranslations,
    ResetMigrations,
    RefreshMigrations,
    FreshMigrate,
    DeleteModule,
}

/// What an action runs on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// One module, by its name.
    Module(String),
    /// Every module, through `--all`.
    All,
}

/// The options the user set for an action.
/// An option that the action does not take is ignored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// `--pretend`: print the SQL and run none of it.
    pub pretend: bool,
    /// `--seed`: run the seeders after the migrations.
    pub seed: bool,
    /// `--drop-views`: a fresh also drops the views.
    pub drop_views: bool,
    /// `--drop-types`: a fresh also drops the types, on Postgres.
    pub drop_types: bool,
    /// `--database=<value>`: the connection. Empty leaves it out.
    pub database: String,
    /// `--class=<value>`: the root seeder. Empty leaves it out.
    pub class: String,
}

/// The options an action takes, as S10.2 draws them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OptionSet {
    pub pretend: bool,
    pub seed: bool,
    pub drop_views: bool,
    pub drop_types: bool,
    pub database: bool,
    pub class: bool,
}

/// What [`plan`] needs to know about the app.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// The app reports the environment `production`, exactly.
    pub production: bool,
    /// The modules that are on and use the target module.
    pub users_on: &'a [String],
    /// Mortise could not read where the app keeps its modules and assumes the
    /// package default. The package may then delete another folder than the one
    /// on screen.
    pub modules_folder_assumed: bool,
}

/// The question Mortise asks before it runs a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Modules that are on use the module.
    TurnOff,
    /// A rollback changes the database.
    RollBack,
    /// The app reports production.
    Production,
    /// The dialog names the tables that change.
    NamingTables,
    /// The user types the name of the app.
    TypeAppName,
    /// The user types the name of the module.
    TypeModuleName,
}

/// What happens when the user runs an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// The command runs at once.
    Run { args: Vec<String> },
    /// Mortise asks first. The words already hold `--force` when the package confirms.
    Ask { ask: Ask, args: Vec<String> },
    /// The command does not run in production. It can only be copied.
    Refused { args: Vec<String> },
    /// The action has no command.
    NoCommand,
}

impl Ask {
    /// The name the user types to confirm, if this question asks for one:
    /// the app for [`Ask::TypeAppName`], the module for [`Ask::TypeModuleName`].
    pub fn name_to_type<'a>(self, app: &'a str, target: &'a Target) -> Option<&'a str> {
        match (self, target) {
            (Ask::TypeAppName, _) => Some(app),
            (Ask::TypeModuleName, Target::Module(module)) => Some(module),
            _ => None,
        }
    }
}

/// True when `typed` is `name` exactly: same case, no space trimmed.
/// An empty text never matches.
pub fn typed_name_matches(typed: &str, name: &str) -> bool {
    !typed.is_empty() && typed == name
}

impl Action {
    /// Every action, for the reverse of [`Action::artisan`].
    const EVERY: [Action; 11] = [
        Action::TurnOn,
        Action::TurnOff,
        Action::Migrate,
        Action::RollBack,
        Action::Seed,
        Action::RunTests,
        Action::CheckTranslations,
        Action::ResetMigrations,
        Action::RefreshMigrations,
        Action::FreshMigrate,
        Action::DeleteModule,
    ];

    /// The actions of one module, in the order of S10.1. The first is
    /// [`Action::TurnOff`] when the module is on, else [`Action::TurnOn`].
    pub fn for_module(on: bool) -> [Action; 6] {
        let turn = if on { Action::TurnOff } else { Action::TurnOn };
        [
            turn,
            Action::Migrate,
            Action::RollBack,
            Action::Seed,
            Action::RunTests,
            Action::CheckTranslations,
        ]
    }

    /// The actions of "All modules".
    pub fn for_all() -> [Action; 4] {
        [
            Action::Migrate,
            Action::Seed,
            Action::RunTests,
            Action::CheckTranslations,
        ]
    }

    /// The destructive actions of `target`, in the order of S10.4. A fresh drops
    /// every table, so it belongs to all modules only. Reset, refresh and delete
    /// belong to one module: `module:delete --all` would delete every module.
    pub fn destructive(target: &Target) -> &'static [Action] {
        match target {
            Target::Module(_) => &[
                Action::ResetMigrations,
                Action::RefreshMigrations,
                Action::DeleteModule,
            ],
            Target::All => &[Action::FreshMigrate],
        }
    }

    /// True for reset, refresh, fresh and delete: the actions that lose data.
    pub fn is_destructive(self) -> bool {
        matches!(
            self,
            Action::ResetMigrations
                | Action::RefreshMigrations
                | Action::FreshMigrate
                | Action::DeleteModule
        )
    }

    /// The action whose artisan command is `word`, such as `module:delete`.
    pub fn from_artisan(word: &str) -> Option<Action> {
        Action::EVERY
            .into_iter()
            .find(|action| action.artisan() == Some(word))
    }

    /// The action runs on `target`. A destructive action runs only on the
    /// targets of [`Action::destructive`].
    fn takes(self, target: &Target) -> bool {
        !self.is_destructive() || Action::destructive(target).contains(&self)
    }

    /// The artisan command of the action. [`Action::RunTests`] has none.
    pub fn artisan(self) -> Option<&'static str> {
        Some(match self {
            Action::TurnOn => "module:enable",
            Action::TurnOff => "module:disable",
            Action::Migrate => "module:migrate",
            Action::RollBack => "module:migrate-rollback",
            Action::Seed => "module:seed",
            Action::RunTests => return None,
            Action::CheckTranslations => "module:lang",
            Action::ResetMigrations => "module:migrate-reset",
            Action::RefreshMigrations => "module:migrate-refresh",
            Action::FreshMigrate => "module:migrate-fresh",
            Action::DeleteModule => "module:delete",
        })
    }

    /// The options the action takes on `target`. Migrate of all modules takes
    /// no database, as S10.2 f draws it.
    pub fn options(self, target: &Target) -> OptionSet {
        let all = *target == Target::All;
        let none = OptionSet::default();
        match self {
            Action::Migrate => OptionSet {
                pretend: true,
                seed: true,
                database: !all,
                ..none
            },
            Action::RollBack => OptionSet {
                pretend: true,
                ..none
            },
            Action::Seed => OptionSet {
                database: true,
                class: true,
                ..none
            },
            Action::ResetMigrations => OptionSet {
                pretend: true,
                database: true,
                ..none
            },
            Action::RefreshMigrations => OptionSet {
                seed: true,
                database: true,
                ..none
            },
            Action::FreshMigrate => OptionSet {
                seed: true,
                drop_views: true,
                drop_types: true,
                database: true,
                ..none
            },
            Action::TurnOn
            | Action::TurnOff
            | Action::RunTests
            | Action::CheckTranslations
            | Action::DeleteModule => none,
        }
    }

    /// The package asks before the command, and fails under `--no-interaction`
    /// unless `--force` is given.
    fn package_confirms(self, production: bool) -> bool {
        match self {
            Action::DeleteModule => true,
            Action::Migrate
            | Action::Seed
            | Action::RollBack
            | Action::ResetMigrations
            | Action::RefreshMigrations
            | Action::FreshMigrate => production,
            Action::TurnOn | Action::TurnOff | Action::RunTests | Action::CheckTranslations => {
                false
            }
        }
    }
}

/// The plan for an action: its words, and whether Mortise asks first.
///
/// The words are the command, the module name or `--all`, the options in the
/// order of [`OptionSet`], then `--force`. A text option is trimmed and is one
/// word, such as `--database=testing`. An empty one is left out.
/// `--force` is added only when the package confirms and Mortise asks.
///
/// A destructive action on a target it does not take has no command, also in
/// production: see [`Action::destructive`].
pub fn plan(action: Action, target: &Target, options: &Options, context: &Context) -> Plan {
    if !action.takes(target) {
        return Plan::NoCommand;
    }
    let Some(command) = action.artisan() else {
        return Plan::NoCommand;
    };
    let mut args = vec![command.to_string()];
    // A name that starts with `-` would read as an option: it goes last, after `--`.
    let mut last = None;
    match target {
        Target::Module(name) if name.starts_with('-') => last = Some(name.clone()),
        Target::Module(name) => args.push(name.clone()),
        Target::All => args.push("--all".to_string()),
    }
    let set = action.options(target);
    let pretend = set.pretend && options.pretend;
    if pretend {
        args.push("--pretend".to_string());
    }
    // The package seeds after a pretend migrate, so a dry run leaves out `--seed`.
    if set.seed && options.seed && !pretend {
        args.push("--seed".to_string());
    }
    if set.drop_views && options.drop_views {
        args.push("--drop-views".to_string());
    }
    if set.drop_types && options.drop_types {
        args.push("--drop-types".to_string());
    }
    if set.database {
        args.extend(text_option("database", &options.database));
    }
    if set.class {
        args.extend(text_option("class", &options.class));
    }

    let production = context.production;
    let ask = match action {
        Action::TurnOn | Action::RunTests | Action::CheckTranslations => None,
        Action::TurnOff => (!context.users_on.is_empty()).then_some(Ask::TurnOff),
        Action::Migrate | Action::Seed => production.then_some(Ask::Production),
        // A dry run changes nothing, so it asks only where the package asks.
        Action::RollBack => (production || !pretend).then_some(Ask::RollBack),
        _ if refuses(action, context) => {
            if let Some(name) = last {
                args.push("--".to_string());
                args.push(name);
            }
            return Plan::Refused { args };
        }
        // A dry run of reset changes nothing, as for roll back.
        Action::ResetMigrations if pretend => None,
        Action::ResetMigrations | Action::RefreshMigrations => Some(Ask::NamingTables),
        Action::FreshMigrate => Some(Ask::TypeAppName),
        Action::DeleteModule => Some(Ask::TypeModuleName),
    };
    if ask.is_some() && action.package_confirms(production) {
        args.push("--force".to_string());
    }
    if let Some(name) = last {
        args.push("--".to_string());
        args.push(name);
    }
    match ask {
        None => Plan::Run { args },
        Some(ask) => Plan::Ask { ask, args },
    }
}

/// True when Mortise asks before `args` run: a command with `--force`, a roll back
/// or a reset that is not a dry run, and refresh, fresh and delete. Run again asks
/// first for these too, because the question was about this very line.
pub fn asks_again(args: &[String]) -> bool {
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .take_while(|word| *word != "--")
        .collect();
    match words.first() {
        Some(&"module:migrate-rollback" | &"module:migrate-reset")
            if !words.contains(&"--pretend") =>
        {
            true
        }
        Some(&"module:migrate-refresh" | &"module:migrate-fresh" | &"module:delete") => true,
        _ => words.contains(&"--force"),
    }
}

/// True when Mortise does not run `action`, and its line can only be copied:
/// reset, refresh, fresh and delete in production, and delete when the modules
/// folder is assumed.
pub fn refuses(action: Action, context: &Context) -> bool {
    action.is_destructive()
        && (context.production
            || (action == Action::DeleteModule && context.modules_folder_assumed))
}

impl Options {
    /// The options that artisan words hold: the flags and the text options that
    /// [`plan`] writes, before a `--`. Run again opens a panel with them.
    pub fn from_args(args: &[String]) -> Options {
        let mut options = Options::default();
        for word in args.iter().take_while(|word| *word != "--") {
            match word.as_str() {
                "--pretend" => options.pretend = true,
                "--seed" => options.seed = true,
                "--drop-views" => options.drop_views = true,
                "--drop-types" => options.drop_types = true,
                word => {
                    if let Some(value) = word.strip_prefix("--database=") {
                        options.database = value.to_string();
                    } else if let Some(value) = word.strip_prefix("--class=") {
                        options.class = value.to_string();
                    }
                }
            }
        }
        options
    }
}

/// `--<name>=<value>` with the value trimmed, or nothing when it is empty.
fn text_option(name: &str, value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| format!("--{name}={value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    fn checkout() -> Target {
        Target::Module("Checkout".to_string())
    }

    /// The plan for `target` with `pretend`, in production or not, with `users` on.
    fn plan_for(
        action: Action,
        target: &Target,
        production: bool,
        pretend: bool,
        users: &[String],
    ) -> Plan {
        let options = Options {
            pretend,
            ..Options::default()
        };
        let context = Context {
            production,
            users_on: users,
            modules_folder_assumed: false,
        };
        plan(action, target, &options, &context)
    }

    #[test]
    fn plans_follow_the_confirmation_table() {
        use Action::*;
        let run = |args: &[&str]| Plan::Run { args: words(args) };
        let ask = |ask, args: &[&str]| Plan::Ask {
            ask,
            args: words(args),
        };
        let refused = |args: &[&str]| Plan::Refused { args: words(args) };
        let shipping = words(&["Shipping"]);
        let none: &[String] = &[];
        let module = checkout();
        let all = Target::All;

        // Action, target, production, pretend, the modules on that use Checkout, the plan.
        let table = [
            (
                TurnOn,
                &module,
                true,
                false,
                none,
                run(&["module:enable", "Checkout"]),
            ),
            (
                TurnOff,
                &module,
                false,
                false,
                none,
                run(&["module:disable", "Checkout"]),
            ),
            (
                TurnOff,
                &module,
                true,
                false,
                &shipping[..],
                ask(Ask::TurnOff, &["module:disable", "Checkout"]),
            ),
            (
                Migrate,
                &module,
                false,
                false,
                none,
                run(&["module:migrate", "Checkout"]),
            ),
            (
                Migrate,
                &module,
                true,
                true,
                none,
                ask(
                    Ask::Production,
                    &["module:migrate", "Checkout", "--pretend", "--force"],
                ),
            ),
            (
                Seed,
                &module,
                false,
                false,
                none,
                run(&["module:seed", "Checkout"]),
            ),
            (
                Seed,
                &module,
                true,
                false,
                none,
                ask(Ask::Production, &["module:seed", "Checkout", "--force"]),
            ),
            (
                RollBack,
                &module,
                false,
                false,
                none,
                ask(Ask::RollBack, &["module:migrate-rollback", "Checkout"]),
            ),
            (
                RollBack,
                &module,
                false,
                true,
                none,
                run(&["module:migrate-rollback", "Checkout", "--pretend"]),
            ),
            (
                RollBack,
                &module,
                true,
                true,
                none,
                ask(
                    Ask::RollBack,
                    &[
                        "module:migrate-rollback",
                        "Checkout",
                        "--pretend",
                        "--force",
                    ],
                ),
            ),
            (RunTests, &module, false, false, none, Plan::NoCommand),
            (
                CheckTranslations,
                &module,
                true,
                false,
                none,
                run(&["module:lang", "Checkout"]),
            ),
            (
                ResetMigrations,
                &module,
                false,
                false,
                none,
                ask(Ask::NamingTables, &["module:migrate-reset", "Checkout"]),
            ),
            // A dry run of reset changes nothing, so it runs at once, except in production.
            (
                ResetMigrations,
                &module,
                false,
                true,
                none,
                run(&["module:migrate-reset", "Checkout", "--pretend"]),
            ),
            (
                ResetMigrations,
                &module,
                true,
                true,
                none,
                refused(&["module:migrate-reset", "Checkout", "--pretend"]),
            ),
            (
                RefreshMigrations,
                &module,
                false,
                false,
                none,
                ask(Ask::NamingTables, &["module:migrate-refresh", "Checkout"]),
            ),
            (
                RefreshMigrations,
                &module,
                true,
                false,
                none,
                refused(&["module:migrate-refresh", "Checkout"]),
            ),
            (
                FreshMigrate,
                &all,
                false,
                false,
                none,
                ask(Ask::TypeAppName, &["module:migrate-fresh", "--all"]),
            ),
            (
                FreshMigrate,
                &all,
                true,
                false,
                none,
                refused(&["module:migrate-fresh", "--all"]),
            ),
            (
                DeleteModule,
                &module,
                false,
                false,
                none,
                ask(
                    Ask::TypeModuleName,
                    &["module:delete", "Checkout", "--force"],
                ),
            ),
            (
                DeleteModule,
                &module,
                true,
                false,
                none,
                refused(&["module:delete", "Checkout"]),
            ),
            // A destructive action on a target it does not take has no command,
            // before production is looked at.
            (ResetMigrations, &all, true, false, none, Plan::NoCommand),
            (RefreshMigrations, &all, false, false, none, Plan::NoCommand),
            (DeleteModule, &all, true, false, none, Plan::NoCommand),
            (FreshMigrate, &module, true, false, none, Plan::NoCommand),
        ];
        for (action, target, production, pretend, users, expected) in table {
            assert_eq!(
                plan_for(action, target, production, pretend, users),
                expected,
                "{action:?} on {target:?}, production {production}, pretend {pretend}"
            );
        }
        assert_eq!(Action::for_module(true)[0], TurnOff);
        assert_eq!(Action::for_module(false)[0], TurnOn);
        assert_eq!(
            Action::destructive(&module),
            [ResetMigrations, RefreshMigrations, DeleteModule]
        );
        assert_eq!(Action::destructive(&all), [FreshMigrate]);
        for action in Action::EVERY {
            let listed = Action::destructive(&module).contains(&action)
                || Action::destructive(&all).contains(&action);
            assert_eq!(action.is_destructive(), listed, "{action:?}");
            // Run again reads the action back from the first word.
            let back = action.artisan().and_then(Action::from_artisan);
            assert_eq!(back, action.artisan().map(|_| action), "{action:?}");
        }
        assert_eq!(Action::from_artisan("module:make"), None);

        // Where the modules folder is assumed, delete can only be copied: the
        // package could delete another folder. Reset is not affected.
        let assumed = Context {
            production: false,
            users_on: none,
            modules_folder_assumed: true,
        };
        let options = Options::default();
        assert_eq!(
            plan(DeleteModule, &module, &options, &assumed),
            refused(&["module:delete", "Checkout"])
        );
        assert!(matches!(
            plan(ResetMigrations, &module, &options, &assumed),
            Plan::Ask { .. }
        ));
    }

    #[test]
    fn force_comes_only_after_a_question() {
        use Action::*;
        let users = words(&["Shipping"]);
        for action in Action::EVERY {
            for target in [checkout(), Target::All] {
                for production in [false, true] {
                    for pretend in [false, true] {
                        let plan = plan_for(action, &target, production, pretend, &users);
                        let case = format!(
                            "{action:?} on {target:?}, production {production}, pretend {pretend}"
                        );
                        let confirms = match action {
                            DeleteModule => true,
                            Migrate | Seed | RollBack | ResetMigrations | RefreshMigrations
                            | FreshMigrate => production,
                            _ => false,
                        };
                        let (asks, args) = match &plan {
                            Plan::Ask { args, .. } => (true, args.clone()),
                            Plan::Run { args } | Plan::Refused { args } => (false, args.clone()),
                            Plan::NoCommand => (false, Vec::new()),
                        };
                        // Where the package confirms, Mortise asks or refuses.
                        if confirms {
                            assert!(!matches!(plan, Plan::Run { .. }), "{case}");
                        }
                        assert_eq!(
                            args.contains(&"--force".to_string()),
                            confirms && asks,
                            "{case}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn run_again_asks_where_mortise_asked() {
        let table: [(&[&str], bool); 10] = [
            (&["module:migrate", "Checkout", "--force"], true),
            (&["module:migrate", "Checkout"], false),
            (&["module:migrate-rollback", "Checkout"], true),
            (&["module:migrate-rollback", "Checkout", "--pretend"], false),
            (&["module:migrate-reset", "Checkout"], true),
            (&["module:migrate-reset", "Checkout", "--pretend"], false),
            (&["module:migrate-refresh", "Checkout"], true),
            (&["module:delete", "Checkout", "--force"], true),
            (&["module:disable", "Checkout"], false),
            // A module named like an option is not one.
            (&["module:migrate", "--", "--force"], false),
        ];
        for (args, asks) in table {
            assert_eq!(asks_again(&words(args)), asks, "{args:?}");
        }
    }

    #[test]
    fn words_keep_their_order() {
        let options = Options {
            pretend: true,
            seed: true,
            drop_views: true,
            drop_types: true,
            database: "  testing ".to_string(),
            class: "Modules\\Checkout\\Database\\Seeders\\CheckoutSeeder".to_string(),
        };
        let production = Context {
            production: true,
            users_on: &[],
            modules_folder_assumed: false,
        };
        // Migrate takes no class and no drop.
        assert_eq!(
            plan(Action::Migrate, &checkout(), &options, &production),
            Plan::Ask {
                ask: Ask::Production,
                args: words(&[
                    "module:migrate",
                    "Checkout",
                    "--pretend",
                    "--database=testing",
                    "--force",
                ]),
            }
        );

        // Migrate of all modules takes no database.
        assert_eq!(
            plan(Action::Migrate, &Target::All, &options, &production),
            Plan::Ask {
                ask: Ask::Production,
                args: words(&["module:migrate", "--all", "--pretend", "--force"]),
            }
        );

        // Without the dry run, Migrate seeds afterwards. A name that starts with `-`
        // goes last, after `--`, so that artisan reads it as the module.
        let seeding = Options {
            pretend: false,
            ..options.clone()
        };
        assert_eq!(
            plan(
                Action::Migrate,
                &Target::Module("-Odd".to_string()),
                &seeding,
                &production
            ),
            Plan::Ask {
                ask: Ask::Production,
                args: words(&[
                    "module:migrate",
                    "--seed",
                    "--database=testing",
                    "--force",
                    "--",
                    "-Odd",
                ]),
            }
        );

        let local = Context {
            production: false,
            users_on: &[],
            modules_folder_assumed: false,
        };
        // Fresh takes no pretend, so it seeds, and drops views and types.
        let fresh = words(&[
            "module:migrate-fresh",
            "--all",
            "--seed",
            "--drop-views",
            "--drop-types",
            "--database=testing",
        ]);
        assert_eq!(
            plan(Action::FreshMigrate, &Target::All, &options, &local),
            Plan::Ask {
                ask: Ask::TypeAppName,
                args: fresh.clone(),
            }
        );
        // Run again reads the options back from the words.
        assert_eq!(
            Options::from_args(&fresh),
            Options {
                pretend: false,
                class: String::new(),
                database: "testing".to_string(),
                ..options.clone()
            }
        );

        // Seed takes neither pretend nor seed, and leaves out an empty database.
        let options = Options {
            database: "   ".to_string(),
            class: " CheckoutSeeder ".to_string(),
            ..options
        };
        assert_eq!(
            plan(Action::Seed, &Target::All, &options, &local),
            Plan::Run {
                args: words(&["module:seed", "--all", "--class=CheckoutSeeder"]),
            }
        );
    }

    #[test]
    fn the_typed_name_must_match_exactly() {
        let module = checkout();
        assert_eq!(
            Ask::TypeAppName.name_to_type("shop-api", &Target::All),
            Some("shop-api")
        );
        assert_eq!(
            Ask::TypeModuleName.name_to_type("shop-api", &module),
            Some("Checkout")
        );
        assert_eq!(Ask::NamingTables.name_to_type("shop-api", &module), None);

        assert!(typed_name_matches("Checkout", "Checkout"));
        assert!(!typed_name_matches("checkout", "Checkout"));
        assert!(!typed_name_matches("Checkout ", "Checkout"));
        assert!(!typed_name_matches("", ""));
    }
}
