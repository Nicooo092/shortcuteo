mod clipboard;
mod control;
mod navigation;
mod scripting;
mod text;
mod variables;
mod web;
mod windows;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::engine::{ParamKind, Registry};
    use crate::storage::workflow::{self, Step};

    // these guard the contract the front relies on, so a new action cannot break the editor
    #[test]
    fn metadata_is_consistent() {
        let metas = Registry::builtin().metas();
        assert!(metas.len() >= 20, "only {} actions registered", metas.len());
        for m in &metas {
            let keys: HashSet<_> = m.params.iter().map(|p| p.key.as_str()).collect();
            assert_eq!(keys.len(), m.params.len(), "{}: duplicate param key", m.id);
            assert!(m.id.contains('.'), "{}: id should be category.name", m.id);
            assert!(
                !m.name.en.is_empty() && !m.name.fr.is_empty(),
                "{}: missing name",
                m.id
            );
            for sentence in [&m.sentence.en, &m.sentence.fr] {
                for placeholder in placeholders(sentence) {
                    assert!(
                        keys.contains(placeholder),
                        "{}: unknown {{{placeholder}}}",
                        m.id
                    );
                }
            }
            for p in &m.params {
                if let ParamKind::Choice { options } = &p.kind {
                    let default = p.default.as_ref().and_then(|d| d.as_str());
                    assert!(
                        default.is_none_or(|d| options.iter().any(|o| o.value == d)),
                        "{}: default of '{}' is not a choice",
                        m.id,
                        p.key
                    );
                }
            }
        }
    }

    #[test]
    fn examples_only_use_known_actions() {
        let registry = Registry::builtin();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            let wf = workflow::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            walk(&wf.actions, &mut |step| {
                assert!(
                    registry.get(&step.action).is_some(),
                    "{}: {}",
                    path.display(),
                    step.action
                );
            });
            checked += 1;
        }
        assert!(checked > 0);
    }

    fn walk(steps: &[Step], f: &mut impl FnMut(&Step)) {
        for s in steps {
            f(s);
            walk(&s.body, f);
            walk(&s.otherwise, f);
            for c in &s.cases {
                walk(&c.body, f);
            }
        }
    }

    fn placeholders(s: &str) -> impl Iterator<Item = &str> {
        s.split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}').map(|(key, _)| key))
    }
}
