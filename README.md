# IES — CLI Rust para o SEI

Recriação limpa do IES em Rust, orientada pelo contrato público de comandos do Todos atual e pelas evidências de instalações reais.

**Estado:** fundação. O programa reconhece uma amostra da gramática, mas nenhuma operação acessa o SEI. Comandos ainda não implementados falham explicitamente.

Não há compatibilidade com o antigo agir, Action ou PageModel público. O código do antigo IES Python está preservado no cofre privado franklinbaldo/projects-backup.

## Desenvolvimento

- cargo test
- cargo fmt --all -- --check
- cargo clippy --all-targets -- -D warnings
- cargo run -- --help

Os contratos deverão ser completados e aprovados antes de implementar o transporte HTTP.
