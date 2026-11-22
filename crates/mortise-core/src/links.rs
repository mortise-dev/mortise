//! Which module uses which: links between modules, worked out from the classes
//! their PHP files name, and the circles among them.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::php::ClassRef;
use crate::rules::Rules;

/// One file of a module that names one class of another module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    /// The file, as the user writes it: `Modules/Checkout/app/Services/InvoiceSync.php`.
    pub file: String,
    /// The first line of the file that names the class.
    pub line: u32,
    /// The class that is named: `Modules\Billing\Models\Invoice`.
    pub class: String,
    /// True when the class is not part of the public surface of its module.
    pub internal: bool,
}

/// One module uses another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The module that uses.
    pub from: String,
    /// The module that is used.
    pub to: String,
    /// What makes the link. References to internals first, then by file and line.
    pub refs: Vec<Reference>,
    /// True when the two modules are in one circle.
    pub circular: bool,
}

impl Link {
    /// How many files of `from` make the link.
    pub fn files(&self) -> usize {
        self.refs
            .iter()
            .map(|reference| reference.file.as_str())
            .collect::<BTreeSet<_>>()
            .len()
    }

    /// How many files of `from` read internals of `to`.
    pub fn internal_files(&self) -> usize {
        self.refs
            .iter()
            .filter(|reference| reference.internal)
            .map(|reference| reference.file.as_str())
            .collect::<BTreeSet<_>>()
            .len()
    }

    /// True when the link breaks a rule.
    pub fn is_problem(&self, rules: &Rules) -> bool {
        (rules.no_circular_use && self.circular)
            || (rules.public_surface_only && self.internal_files() > 0)
    }
}

/// Modules that use each other, directly or through others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cycle {
    /// The modules of the circle, by name.
    pub modules: Vec<String>,
    /// The module the finding is counted on: the one with the fewest files in the
    /// circle, because that is the cheaper side to cut.
    pub carrier: String,
    /// For each module of the circle, how many of its files use another module of it.
    pub files: Vec<(String, usize)>,
}

/// The PHP files of one module, as the links need them.
pub(crate) struct ModuleSource<'a> {
    pub(crate) name: &'a str,
    /// The namespace all classes of the module share: `Modules\Billing`.
    pub(crate) namespace: &'a str,
    /// Each file as the user writes it, with the classes it names.
    pub(crate) files: Vec<(String, &'a [ClassRef])>,
    /// The classes, interfaces, traits and enums the module declares.
    pub(crate) declared: BTreeSet<&'a str>,
}

/// Finds the links between modules and the circles among them.
pub(crate) fn build(modules: &[ModuleSource<'_>], rules: &Rules) -> (Vec<Link>, Vec<Cycle>) {
    // Longest namespace first, so that `Modules\Billing\Tax` wins over `Modules\Billing`.
    let mut owners: Vec<&ModuleSource<'_>> = modules
        .iter()
        .filter(|module| !module.namespace.is_empty())
        .collect();
    owners.sort_by_key(|module| std::cmp::Reverse(module.namespace.len()));
    let owner_of = |class: &str| -> Option<(&ModuleSource<'_>, String)> {
        owners.iter().find_map(|module| {
            let inside = class.strip_prefix(module.namespace)?.strip_prefix('\\')?;
            Some((*module, inside.to_owned()))
        })
    };

    let mut found: BTreeMap<(String, String), Vec<Reference>> = BTreeMap::new();
    for module in modules {
        for (file, refs) in &module.files {
            for named in *refs {
                let Some((owner, inside)) = owner_of(&named.class) else {
                    continue;
                };
                // A module using itself is not a link.
                if owner.name == module.name || inside.is_empty() {
                    continue;
                }
                // `use Modules\Billing\Contracts;` names a namespace, not a class:
                // nothing is declared under that name, but classes are declared inside it.
                let as_namespace = format!("{}\\", named.class);
                if !owner.declared.contains(named.class.as_str())
                    && owner
                        .declared
                        .iter()
                        .any(|declared| declared.starts_with(&as_namespace))
                {
                    continue;
                }
                found
                    .entry((module.name.to_owned(), owner.name.to_owned()))
                    .or_default()
                    .push(Reference {
                        file: file.clone(),
                        line: named.line,
                        class: named.class.clone(),
                        internal: rules.public_surface_only && !rules.is_public(&inside),
                    });
            }
        }
    }

    let edges: BTreeSet<(&str, &str)> = found
        .keys()
        .map(|(from, to)| (from.as_str(), to.as_str()))
        .collect();
    let names: Vec<&str> = modules.iter().map(|module| module.name).collect();
    // Owned, so that the links can be taken out of `found` below.
    let circles: Vec<Vec<String>> = circles(&names, &edges)
        .into_iter()
        .map(|circle| circle.into_iter().map(str::to_owned).collect())
        .collect();
    let circle_of = |name: &str| {
        circles
            .iter()
            .position(|circle| circle.iter().any(|member| member == name))
    };

    let cycles = circles
        .iter()
        .map(|circle| {
            let mut files: Vec<(String, usize)> = circle
                .iter()
                .map(|member| {
                    let own: BTreeSet<&str> = found
                        .iter()
                        .filter(|((from, to), _)| from == member && circle.contains(to))
                        .flat_map(|(_, refs)| refs.iter().map(|reference| reference.file.as_str()))
                        .collect();
                    (member.clone(), own.len())
                })
                .collect();
            files.sort();
            let carrier = files
                .iter()
                .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
                .map(|(name, _)| name.clone())
                .unwrap_or_default();
            Cycle {
                modules: files.iter().map(|(name, _)| name.clone()).collect(),
                carrier,
                files,
            }
        })
        .collect();

    let links = found
        .into_iter()
        .map(|((from, to), mut refs)| {
            refs.sort_by(|a, b| {
                b.internal
                    .cmp(&a.internal)
                    .then_with(|| a.file.cmp(&b.file))
                    .then_with(|| a.line.cmp(&b.line))
            });
            let circular = circle_of(&from).is_some() && circle_of(&from) == circle_of(&to);
            Link {
                from,
                to,
                refs,
                circular,
            }
        })
        .collect();
    (links, cycles)
}

/// The groups of modules that can reach each other along the links, with two or
/// more members each. Tarjan's algorithm, without recursion.
fn circles<'a>(names: &[&'a str], edges: &BTreeSet<(&'a str, &'a str)>) -> Vec<Vec<&'a str>> {
    let index_of: BTreeMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, i))
        .collect();
    let mut next: Vec<Vec<usize>> = vec![Vec::new(); names.len()];
    for (from, to) in edges {
        if let (Some(&from), Some(&to)) = (index_of.get(from), index_of.get(to)) {
            next[from].push(to);
        }
    }

    const UNSEEN: usize = usize::MAX;
    let mut order = vec![UNSEEN; names.len()];
    let mut low = vec![0; names.len()];
    let mut on_stack = vec![false; names.len()];
    let mut stack: Vec<usize> = Vec::new();
    let mut counter = 0;
    let mut groups: Vec<Vec<&str>> = Vec::new();

    for start in 0..names.len() {
        if order[start] != UNSEEN {
            continue;
        }
        // Each entry: a module and how many of its links were followed.
        let mut work = vec![(start, 0)];
        while let Some(&mut (node, ref mut followed)) = work.last_mut() {
            if *followed == 0 {
                order[node] = counter;
                low[node] = counter;
                counter += 1;
                stack.push(node);
                on_stack[node] = true;
            }
            if let Some(&target) = next[node].get(*followed) {
                *followed += 1;
                if order[target] == UNSEEN {
                    work.push((target, 0));
                } else if on_stack[target] {
                    low[node] = low[node].min(order[target]);
                }
                continue;
            }
            work.pop();
            if let Some(&(parent, _)) = work.last() {
                low[parent] = low[parent].min(low[node]);
            }
            if low[node] == order[node] {
                let mut group = Vec::new();
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    group.push(names[member]);
                    if member == node {
                        break;
                    }
                }
                if group.len() > 1 {
                    group.sort_unstable();
                    groups.push(group);
                }
            }
        }
    }
    groups.sort();
    groups
}

/// Places modules in columns from left to right: a module sits to the left of what
/// it uses, and the modules of a circle share a column.
pub fn columns(names: &[&str], links: &[Link], cycles: &[Cycle]) -> Vec<Vec<String>> {
    // A circle counts as one node.
    let group = |name: &str| -> String {
        cycles
            .iter()
            .find(|cycle| cycle.modules.iter().any(|member| member == name))
            .map_or_else(|| name.to_owned(), |cycle| cycle.modules[0].clone())
    };
    let mut users: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for link in links {
        let (from, to) = (group(&link.from), group(&link.to));
        if from != to {
            users.entry(to).or_default().insert(from);
        }
    }

    // The column of a node: one more than the furthest right of the nodes that use it.
    fn column(
        node: &str,
        users: &BTreeMap<String, BTreeSet<String>>,
        known: &mut BTreeMap<String, usize>,
    ) -> usize {
        if let Some(&column) = known.get(node) {
            return column;
        }
        // Circles are one node, so this cannot loop; the placeholder guards against bad input.
        known.insert(node.to_owned(), 0);
        let column = users.get(node).map_or(0, |users_of| {
            users_of
                .iter()
                .map(|user| column(user, users, known) + 1)
                .max()
                .unwrap_or(0)
        });
        known.insert(node.to_owned(), column);
        column
    }

    let mut known = BTreeMap::new();
    let mut columns: Vec<Vec<String>> = Vec::new();
    for name in names {
        let at = column(&group(name), &users, &mut known);
        if columns.len() <= at {
            columns.resize(at + 1, Vec::new());
        }
        columns[at].push((*name).to_owned());
    }
    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(class: &str, line: u32) -> ClassRef {
        ClassRef {
            class: class.to_owned(),
            line,
        }
    }

    fn source<'a>(
        name: &'a str,
        namespace: &'a str,
        files: &'a [(&'a str, Vec<ClassRef>)],
    ) -> ModuleSource<'a> {
        ModuleSource {
            name,
            namespace,
            files: files
                .iter()
                .map(|(file, refs)| ((*file).to_owned(), refs.as_slice()))
                .collect(),
            declared: BTreeSet::new(),
        }
    }

    #[test]
    fn a_link_is_made_of_files_that_name_classes_of_another_module() {
        let checkout = [
            (
                "Modules/Checkout/app/A.php",
                vec![
                    named(r"Modules\Billing\Models\Invoice", 5),
                    named(r"Modules\Checkout\Models\Order", 6),
                ],
            ),
            (
                "Modules/Checkout/app/B.php",
                vec![
                    named(r"Modules\Billing\Contracts\Gateway", 7),
                    named(r"Illuminate\Support\Str", 3),
                ],
            ),
            (
                "Modules/Checkout/app/C.php",
                vec![
                    named(r"Modules\Billing\Contracts", 4),
                    named(r"Modules\BillingExtras\Thing", 4),
                ],
            ),
        ];
        let mut billing = source("Billing", r"Modules\Billing", &[]);
        billing.declared = BTreeSet::from([
            r"Modules\Billing\Contracts\Gateway",
            r"Modules\Billing\Models\Invoice",
        ]);
        let modules = [source("Checkout", r"Modules\Checkout", &checkout), billing];
        let (links, cycles) = build(&modules, &Rules::default());

        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(
            (link.from.as_str(), link.to.as_str()),
            ("Checkout", "Billing")
        );
        assert!(!link.circular);
        // The internal one first.
        let refs: Vec<_> = link
            .refs
            .iter()
            .map(|r| (r.file.as_str(), r.line, r.class.as_str(), r.internal))
            .collect();
        assert_eq!(
            refs,
            [
                (
                    "Modules/Checkout/app/A.php",
                    5,
                    r"Modules\Billing\Models\Invoice",
                    true
                ),
                (
                    "Modules/Checkout/app/B.php",
                    7,
                    r"Modules\Billing\Contracts\Gateway",
                    false
                ),
            ]
        );
        assert_eq!((link.files(), link.internal_files()), (2, 1));
        assert!(link.is_problem(&Rules::default()));
        assert!(cycles.is_empty());
    }

    #[test]
    fn without_the_rule_nothing_is_internal() {
        let checkout = [("a.php", vec![named(r"Modules\Billing\Models\Invoice", 5)])];
        let modules = [
            source("Checkout", r"Modules\Checkout", &checkout),
            source("Billing", r"Modules\Billing", &[]),
        ];
        let rules = Rules {
            public_surface_only: false,
            ..Rules::default()
        };
        let (links, _) = build(&modules, &rules);
        assert!(!links[0].refs[0].internal);
        assert!(!links[0].is_problem(&rules));
    }

    #[test]
    fn two_modules_that_use_each_other_are_a_circle_counted_on_the_smaller_side() {
        let checkout = [
            (
                "c1.php",
                vec![named(r"Modules\Shipping\Contracts\Rates", 7)],
            ),
            (
                "c2.php",
                vec![named(r"Modules\Shipping\Contracts\Requester", 6)],
            ),
        ];
        let shipping = [
            (
                "s1.php",
                vec![named(r"Modules\Checkout\Events\OrderPaid", 8)],
            ),
            (
                "s2.php",
                vec![named(r"Modules\Checkout\Contracts\Lookup", 10)],
            ),
            (
                "s3.php",
                vec![named(r"Modules\Checkout\Contracts\Lookup", 9)],
            ),
            (
                "s4.php",
                vec![named(r"Modules\Checkout\Events\OrderShipped", 11)],
            ),
        ];
        let modules = [
            source("Checkout", r"Modules\Checkout", &checkout),
            source("Shipping", r"Modules\Shipping", &shipping),
            source("Users", r"Modules\Users", &[]),
        ];
        let (links, cycles) = build(&modules, &Rules::default());
        assert!(
            links
                .iter()
                .all(|link| link.circular && link.is_problem(&Rules::default()))
        );
        assert_eq!(
            cycles,
            [Cycle {
                modules: vec!["Checkout".into(), "Shipping".into()],
                carrier: "Checkout".into(),
                files: vec![("Checkout".into(), 2), ("Shipping".into(), 4)],
            }]
        );
        // With the rule off, a circle is not a problem, but it is still known.
        let rules = Rules {
            no_circular_use: false,
            ..Rules::default()
        };
        assert!(
            links
                .iter()
                .all(|link| link.circular && !link.is_problem(&rules))
        );
    }

    #[test]
    fn a_circle_through_three_modules_is_found() {
        let a = [("a.php", vec![named(r"M\B\Contracts\X", 1)])];
        let b = [("b.php", vec![named(r"M\C\Contracts\X", 1)])];
        let c = [
            ("c.php", vec![named(r"M\A\Contracts\X", 1)]),
            ("c2.php", vec![named(r"M\D\Contracts\X", 1)]),
        ];
        let modules = [
            source("A", r"M\A", &a),
            source("B", r"M\B", &b),
            source("C", r"M\C", &c),
            source("D", r"M\D", &[]),
        ];
        let (links, cycles) = build(&modules, &Rules::default());
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].modules, ["A", "B", "C"]);
        assert_eq!(cycles[0].carrier, "A");
        let to_d = links.iter().find(|link| link.to == "D").unwrap();
        assert!(!to_d.circular);
    }

    #[test]
    fn a_module_sits_left_of_what_it_uses_and_a_circle_shares_a_column() {
        let link = |from: &str, to: &str, circular: bool| Link {
            from: from.into(),
            to: to.into(),
            refs: Vec::new(),
            circular,
        };
        let links = [
            link("Notifications", "Checkout", false),
            link("Checkout", "Shipping", true),
            link("Shipping", "Checkout", true),
            link("Checkout", "Billing", false),
            link("Billing", "Users", false),
            link("Inventory", "Catalog", false),
            link("Checkout", "Catalog", false),
        ];
        let cycles = [Cycle {
            modules: vec!["Checkout".into(), "Shipping".into()],
            carrier: "Checkout".into(),
            files: Vec::new(),
        }];
        let names = [
            "Billing",
            "Catalog",
            "Checkout",
            "Inventory",
            "Notifications",
            "Shipping",
            "Users",
            "Alone",
        ];
        assert_eq!(
            columns(&names, &links, &cycles),
            [
                vec!["Inventory", "Notifications", "Alone"],
                vec!["Checkout", "Shipping"],
                vec!["Billing", "Catalog"],
                vec!["Users"],
            ]
        );
    }
}
