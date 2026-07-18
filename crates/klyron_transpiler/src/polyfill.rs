use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Polyfill {
    pub name: &'static str,
    pub code: &'static str,
    pub targets: Vec<&'static str>,
}

pub struct PolyfillRegistry {
    polyfills: HashMap<&'static str, Polyfill>,
}

impl PolyfillRegistry {
    pub fn new() -> Self {
        let mut polyfills = HashMap::new();
        polyfills.insert(
            "Array.prototype.includes",
            Polyfill {
                name: "Array.prototype.includes",
                code: r#"if (!Array.prototype.includes) {
  Object.defineProperty(Array.prototype, 'includes', {
    value: function(searchElement, fromIndex) {
      if (this == null) throw new TypeError('"this" is null or not defined');
      var o = Object(this);
      var len = o.length >>> 0;
      if (len === 0) return false;
      var k = fromIndex | 0;
      k = Math.max(k >= 0 ? k : len + k, 0);
      while (k < len) {
        if (o[k] === searchElement) return true;
        k++;
      }
      return false;
    }
  });
}"#,
                targets: vec!["es5"],
            },
        );
        polyfills.insert(
            "Object.entries",
            Polyfill {
                name: "Object.entries",
                code: r#"if (!Object.entries) {
  Object.entries = function(obj) {
    var ownProps = Object.keys(obj);
    var i = ownProps.length;
    var res = new Array(i);
    while (i--) res[i] = [ownProps[i], obj[ownProps[i]]];
    return res;
  };
}"#,
                targets: vec!["es5"],
            },
        );
        polyfills.insert(
            "String.prototype.startsWith",
            Polyfill {
                name: "String.prototype.startsWith",
                code: r#"if (!String.prototype.startsWith) {
  String.prototype.startsWith = function(search, pos) {
    return this.substr(!pos || pos < 0 ? 0 : +pos, search.length) === search;
  };
}"#,
                targets: vec!["es5"],
            },
        );
        polyfills.insert(
            "Promise",
            Polyfill {
                name: "Promise",
                code: r#"if (typeof Promise === 'undefined') {
  window.Promise = function(executor) {
    // minimal polyfill
  };
}"#,
                targets: vec!["es5"],
            },
        );
        Self { polyfills }
    }

    pub fn get(&self, name: &str) -> Option<&Polyfill> {
        self.polyfills.get(name)
    }

    pub fn all(&self) -> impl Iterator<Item = &Polyfill> {
        self.polyfills.values()
    }

    pub fn for_target(&self, target: &str) -> Vec<&Polyfill> {
        self.polyfills
            .values()
            .filter(|p| p.targets.iter().any(|t| *t == target))
            .collect()
    }

    pub fn inject_polyfills(source: &str, target: &str) -> String {
        let registry = Self::new();
        let needed = registry.for_target(target);
        if needed.is_empty() {
            return source.to_string();
        }
        let mut result = String::new();
        for p in &needed {
            result.push_str("// polyfill: ");
            result.push_str(p.name);
            result.push('\n');
            result.push_str(p.code);
            result.push('\n');
        }
        result.push_str(source);
        result
    }
}

impl Default for PolyfillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyfill_registry_new() {
        let reg = PolyfillRegistry::new();
        assert!(reg.get("Promise").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn test_polyfill_for_target() {
        let reg = PolyfillRegistry::new();
        let es5 = reg.for_target("es5");
        assert!(!es5.is_empty());
    }

    #[test]
    fn test_inject_polyfills() {
        let source = "const x = [1,2,3].includes(1);";
        let result = PolyfillRegistry::inject_polyfills(source, "es5");
        assert!(result.contains("polyfill:"));
        assert!(result.contains("Array.prototype.includes"));
        assert!(result.contains(source));
    }

    #[test]
    fn test_inject_no_target() {
        let source = "const x = 1;";
        let result = PolyfillRegistry::inject_polyfills(source, "esnext");
        assert_eq!(result, source);
    }
}
