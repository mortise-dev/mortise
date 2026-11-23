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
    /// `--pretend`: print the SQL and change nothing.
    pub pretend: bool,
    /// `--seed`: run the seeders after the migrations.
    pub seed: bool,
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

impl Action {
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

    /// The options the action takes.
    pub fn options(self) -> OptionSet {
        let (pretend, seed, database, class) = match self {
            Action::Migrate => (true, true, true, false),
            Action::RollBack => (true, false, false, false),
            Action::Seed => (false, false, true, true),
            Action::ResetMigrations => (true, false, true, false),
            Action::RefreshMigrations | Action::FreshMigrate => (false, true, true, false),
            Action::TurnOn
            | Action::TurnOff
            | Action::RunTests
            | Action::CheckTranslations
            | Action::DeleteModule => (false, false, false, false),
        };
        OptionSet {
            pretend,
            seed,
            database,
            class,
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
pub fn plan(action: Action, target: &Target, options: &Options, context: &Context) -> Plan {
    let Some(command) = action.artisan() else {
        return Plan::NoCommand;
    };
    let mut args = vec![command.to_string()];
    args.push(match target {
        Target::Module(name) => name.clone(),
        Target::All => "--all".to_string(),
    });
    let set = action.options();
    let pretend = set.pretend && options.pretend;
    if pretend {
        args.push("--pretend".to_string());
    }
    if set.seed && options.seed {
        args.push("--seed".to_string());
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
        Action::ResetMigrations
        | Action::RefreshMigrations
        | Action::FreshMigrate
        | Action::DeleteModule
            if production =>
        {
            return Plan::Refused { args };
        }
        Action::ResetMigrations | Action::RefreshMigrations => Some(Ask::NamingTables),
        Action::FreshMigrate => Some(Ask::TypeAppName),
        Action::DeleteModule => Some(Ask::TypeModuleName),
    };
    match ask {
        None => Plan::Run { args },
        Some(ask) => {
            if action.package_confirms(production) {
                args.push("--force".to_string());
            }
            Plan::Ask { ask, args }
        }
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

    /// The plan for Checkout with `pretend`, in production or not, with `users` on.
    fn plan_for(action: Action, production: bool, pretend: bool, users: &[String]) -> Plan {
        let options = Options {
            pretend,
            ..Options::default()
        };
        let context = Context {
            production,
            users_on: users,
        };
        plan(
            action,
            &Target::Module("Checkout".to_string()),
            &options,
            &context,
        )
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

        // Action, production, pretend, the modules on that use Checkout, the plan.
        let table = [
            (
                TurnOn,
                true,
                false,
                none,
                run(&["module:enable", "Checkout"]),
            ),
            (
                TurnOff,
                false,
                false,
                none,
                run(&["module:disable", "Checkout"]),
            ),
            (
                TurnOff,
                true,
                false,
                &shipping[..],
                ask(Ask::TurnOff, &["module:disable", "Checkout"]),
            ),
            (
                Migrate,
                false,
                false,
                none,
                run(&["module:migrate", "Checkout"]),
            ),
            (
                Migrate,
                true,
                true,
                none,
                ask(
                    Ask::Production,
                    &["module:migrate", "Checkout", "--pretend", "--force"],
                ),
            ),
            (Seed, false, false, none, run(&["module:seed", "Checkout"])),
            (
                Seed,
                true,
                false,
                none,
                ask(Ask::Production, &["module:seed", "Checkout", "--force"]),
            ),
            (
                RollBack,
                false,
                false,
                none,
                ask(Ask::RollBack, &["module:migrate-rollback", "Checkout"]),
            ),
            (
                RollBack,
                false,
                true,
                none,
                run(&["module:migrate-rollback", "Checkout", "--pretend"]),
            ),
            (
                RollBack,
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
            (RunTests, false, false, none, Plan::NoCommand),
            (
                CheckTranslations,
                true,
                false,
                none,
                run(&["module:lang", "Checkout"]),
            ),
            (
                ResetMigrations,
                false,
                false,
                none,
                ask(Ask::NamingTables, &["module:migrate-reset", "Checkout"]),
            ),
            (
                RefreshMigrations,
                false,
                false,
                none,
                ask(Ask::NamingTables, &["module:migrate-refresh", "Checkout"]),
            ),
            (
                RefreshMigrations,
                true,
                false,
                none,
                refused(&["module:migrate-refresh", "Checkout"]),
            ),
            (
                FreshMigrate,
                false,
                false,
                none,
                ask(Ask::TypeAppName, &["module:migrate-fresh", "Checkout"]),
            ),
            (
                FreshMigrate,
                true,
                false,
                none,
                refused(&["module:migrate-fresh", "Checkout"]),
            ),
            (
                DeleteModule,
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
                true,
                false,
                none,
                refused(&["module:delete", "Checkout"]),
            ),
        ];
        for (action, production, pretend, users, expected) in table {
            assert_eq!(
                plan_for(action, production, pretend, users),
                expected,
                "{action:?}, production {production}, pretend {pretend}"
            );
        }
        assert_eq!(Action::for_module(true)[0], TurnOff);
        assert_eq!(Action::for_module(false)[0], TurnOn);
    }

    #[test]
    fn force_comes_only_after_a_question() {
        use Action::*;
        let every = [
            TurnOn,
            TurnOff,
            Migrate,
            RollBack,
            Seed,
            RunTests,
            CheckTranslations,
            ResetMigrations,
            RefreshMigrations,
            FreshMigrate,
            DeleteModule,
        ];
        let users = words(&["Shipping"]);
        for action in every {
            for production in [false, true] {
                for pretend in [false, true] {
                    let plan = plan_for(action, production, pretend, &users);
                    let case = format!("{action:?}, production {production}, pretend {pretend}");
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

    #[test]
    fn words_keep_their_order() {
        let options = Options {
            pretend: true,
            seed: true,
            database: "  testing ".to_string(),
            class: "Modules\\Checkout\\Database\\Seeders\\CheckoutSeeder".to_string(),
        };
        let production = Context {
            production: true,
            users_on: &[],
        };
        // Migrate takes no class.
        assert_eq!(
            plan(
                Action::Migrate,
                &Target::Module("Checkout".to_string()),
                &options,
                &production
            ),
            Plan::Ask {
                ask: Ask::Production,
                args: words(&[
                    "module:migrate",
                    "Checkout",
                    "--pretend",
                    "--seed",
                    "--database=testing",
                    "--force",
                ]),
            }
        );

        // Seed takes neither pretend nor seed, and leaves out an empty database.
        let options = Options {
            database: "   ".to_string(),
            class: " CheckoutSeeder ".to_string(),
            ..options
        };
        let local = Context {
            production: false,
            users_on: &[],
        };
        assert_eq!(
            plan(Action::Seed, &Target::All, &options, &local),
            Plan::Run {
                args: words(&["module:seed", "--all", "--class=CheckoutSeeder"]),
            }
        );
    }
}
