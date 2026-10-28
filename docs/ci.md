---
type: Policy
title: Política de integração contínua do IES
description: Gates reproduzíveis com prek, GitHub Actions, OKF e limite de tamanho
---

# CI orientado por prek

O arquivo [prek.toml](../prek.toml) é a única fonte de verdade dos gates.

## Comandos

- Pré-commit: `uvx --from prek==0.5.7 prek run --hook-stage pre-commit --all-files`.
- Pré-push e CI remoto: `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`.
- Instalação: `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.

O Clippy ativa `clippy::pedantic` e trata avisos como erros; exceções explícitas e justificadas: `too_many_lines` (limite arbitrário de linhas), `case_sensitive_file_extension_comparisons` (extensões de artefatos verificadas exatamente), `struct_excessive_bools` (flags independentes do Clap), `assert_is_empty` (testes legíveis) e `format_collect` (fixtures pequenas). Reavaliar cada exceção ao tocar no código correspondente.\n\nO pré-commit executa tamanho, rustfmt e OKF. O pré-push acrescenta Clippy,
testes, integridade de evidências e decisões. O `cargo test --workspace
--all-targets` compila os targets; não há gate `cargo build` redundante.

## GitHub Actions

O [workflow de CI](../.github/workflows/ci.yml) roda em PRs, pushes na
`main` e por execução manual. O checkout e a instalação de uv usam
actions fixadas por SHA, inclusive `Swatinem/rust-cache` para o cache
de compilação Rust. A versão de Rust vem de `rust-toolchain.toml`.
O executor roda o mesmo comando do pré-push, sem duplicar os gates.
O token do workflow recebe apenas `contents: read`.
Não há secrets ou acesso autenticado ao SEI.

## Decisões de engenharia

O `prek` executa `cargo run --quiet --locked -p xtask --bin check_design`.
Para cada `CommandContract` com `implementation: partial` ou `complete`,
o gate exige um `ImplementationDecision` em `docs/decisoes/`, com
alternativas distintas, recorte implementado, estratégia selecionada,
estado de evidência e links válidos para contrato, matriz e testes Rust.

O gate impede documentação órfã e omissões estruturais. Ele **não
transforma uma justificativa em demonstração de equivalência**.
Stubs são dispensados, sem gerar documentos vazios.

## Ferramentas de verificação

`check_size`, `check_evidence` e `check_design` pertencem ao membro Rust
`xtask`, sem dependências. O check de tamanho não deve compilar `ies`
(`openssl`, `zip` ou `clap`) apenas para verificar os arquivos. Todos
os caminhos OKF e de contratos são resolvidos na raiz do workspace.

Para medir gargalos sem especular, execute `cargo build --timings` num
executor adequado; não mantenha o HTML de timings versionado.

## Compatibilidade da saída JSON

O teste `contrato_json_v1_e_deterministico` compara byte a byte
o replay sintético com `tests/fixtures/caixa-v1.json`. Alterações
de formato exigem revisão explícita do golden e do contrato OKF.
O domínio tipado pode evoluir sem gerar chaves JSON acidentalmente.

## OKF

Todo Markdown autoral exige frontmatter com `type` e Spec correspondente
em `.okf/specs/{slug}.md`; o gate exige `--normative-spec`.
A versão do parser é fixada em `0.48.2`, ferramenta externa de CI;
o executável IES não depende de Python.

## Arquivos

Máximo de 20.000 bytes UTF-8 por arquivo texto, inclusive `Cargo.lock`.
Arquivos binários são ignorados. Nada de exceções automáticas para gerados.
Arquivos que precisem superar o limite exigem reconsideração explícita da
política — nunca truncamento ou minificação para contornar o check.
