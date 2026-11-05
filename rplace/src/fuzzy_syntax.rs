use std::sync::Mutex;

use anyhow::{Context, Result, anyhow};
use serde_json::json;

use crate::{lexer::TokenResult, runtime_module::RuntimeModule};

pub struct FuzzySyntax {
    enabled: bool,
    module: Mutex<ModuleState>,
}

enum ModuleState {
    Uninitialized,
    Missing,
    Loaded(RuntimeModule),
}

impl FuzzySyntax {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            module: Mutex::new(ModuleState::Uninitialized),
        }
    }

    pub fn lex(&self, tokens: TokenResult) -> Result<TokenResult> {
        if !self.enabled {
            return Ok(tokens);
        }

        let mut state = self
            .module
            .lock()
            .map_err(|_| anyhow!("Fuzzy syntax module lock poisoned"))?;
        if matches!(*state, ModuleState::Uninitialized) {
            *state = match RuntimeModule::load_optional("rplace_fuzzy_sintax")? {
                Some(module) => ModuleState::Loaded(module),
                None => ModuleState::Missing,
            };
        }
        let ModuleState::Loaded(module) = &*state else {
            return Ok(tokens);
        };

        let response = module
            .call(&json!({
                "action": "fuzzy_lex",
                "tokens": tokens,
            }))
            .context("Fuzzy syntax module failed")?;
        serde_json::from_value(response).context("Fuzzy syntax module returned invalid tokens")
    }
}

#[cfg(test)]
mod tests {
    use super::FuzzySyntax;
    use crate::lexer::{Lexer, Token};

    #[test]
    fn disabled_fuzzy_syntax_returns_lexer_tokens_unchanged() {
        let tokens = Lexer::new("test", "def example:".to_string()).parse();
        let result = FuzzySyntax::new(false)
            .lex(tokens)
            .expect("Disabled fuzzy syntax should not load a plugin");

        assert!(matches!(result.tokens.first(), Some(Token::DEF)));
    }
}
