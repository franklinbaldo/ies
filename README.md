---
type: Project
title: IES
description: CLI Rust para operar o SEI por contratos de dominio e evidencias reais
---

# IES — CLI Rust para o SEI

Recriação limpa do IES em Rust, orientada pelo contrato público de comandos
do Todos atual e pelas evidências de instalações reais.

**Estado:** fundação. A CLI reconhece uma amostra da gramática, mas nenhuma
operação acessa o SEI. Comandos ainda não implementados falham explicitamente.

Não há compatibilidade com o antigo `agir`, `Action` ou `PageModel` público.
O IES Python histórico está preservado no cofre privado
`franklinbaldo/projects-backup`.

O CI público em [GitHub Actions](.github/workflows/ci.yml) executa os mesmos
gates do `prek` usados localmente.

## Experimentos reais do SEI

O [catálogo OKF de cassettes](docs/cassettes.md) registra 478 revisões
distintas observadas em sete instalações de treinamento/homologação.
Os ZIPs continuam exclusivamente no Internet Archive. Consulte também
[como o SEI funciona](docs/como-o-sei-funciona/README.md), documentação
empírica independente da arquitetura Python anterior.

## Desenvolvimento

- `cargo test --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
- `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`

O [contrato de CLI](contracts/cli.md) deve ser completado e aprovado antes
de implementar transporte e operações do SEI. Consulte a [política de CI](docs/ci.md).
