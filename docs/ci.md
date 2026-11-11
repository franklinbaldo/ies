---
type: Policy
title: Política de integração contínua do IES
description: Gates reproduzíveis com prek, GitHub Actions, OKF e limite de tamanho
---

# CI orientado por prek

O arquivo [prek.toml](../prek.toml) é a única fonte de verdade dos gates.

## Comandos

- Pré-commit: `uvx --from prek==0.5.7 prek run --hook-stage pre-commit --all-files`.
- Pré-push e CI completo: `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`.
- CI rápido: `uvx --from prek==0.5.7 prek run --hook-stage pre-commit --all-files`.
- Instalação: `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.

O Clippy ativa `clippy::pedantic` e trata avisos como erros. Não há exceções globais; a validação literal de extensões de evidências possui exceções locais documentadas e as flags independentes do Clap conservam uma exceção localizada de `struct_excessive_bools`.

O pré-commit executa tamanho, rustfmt e OKF. O pré-push acrescenta Clippy,
testes, integridade de evidências, autotestes Python, auditoria OSV e
decisões e cobertura documental de subcódigos. O `cargo test --workspace
--all-targets` compila os targets; não há gate `cargo build` redundante.

## Testes pendentes de fatias WIP

O hook `wip-ignores` (`scripts/wip_ignores.py`, só biblioteca padrão)
deixa fatias incompletas entrarem na `main` sem esconder regressão.
Proíbe `#[ignore]` sem motivo; exige `wip #<issue>: <o que falta>` em
motivo que começa com `wip`; e recusa teste que existia sem `#[ignore]`
na base e passou a ser ignorado. A base é o primeiro pai do commit de
merge (por isso o checkout do gate usa `fetch-depth: 2`) ou o
`merge-base` com `origin/main`. Sem base, falha no CI e fica `NOT_RUN`
local. O relatório lista os pendentes por issue.

## Binário para os agentes

O job `binario` do `ci.yml` não é gate. Ele compila `ies` em release no
`ubuntu-22.04` (glibc 2.35 e libssl3, para rodar em sandboxes mais
antigos que o runner do gate) e publica o artifact
`ies-linux-x86_64-<sha>` (o head da PR, ou o commit no push), com o binário e o `ies.sha256`, por 7 dias.
Serve aos agentes sem Rust no sandbox: eles observam o RED no log do
gate e testam esse binário no sandbox ou num workflow temporário.

## Auditoria de dependências (cargo-deny)

O hook `auditoria-dependencias` roda `cargo deny check advisories`
(`cargo-deny` 0.20.2, instalado no CI com `cargo install --locked`). Ele
confere o `Cargo.lock` contra a base RustSec e falha listando o aviso.

Avisos aceitos ficam em `deny.toml`, `[advisories].ignore`, cada um com
`id` e `reason` (por que não se aplica ao IES e quando revisar).

Sem `cargo-deny` instalado, o hook falha no CI (`CI` definido) e imprime
`NOT_RUN` localmente, para o pré-push não depender da ferramenta; o CI é
a barreira que não pode ser pulada.

## GitHub Actions: rápido por commit, completo antes do merge

- `.github/workflows/ci-rapido.yml`: em cada PR e push na `main`;
  só os hooks canônicos de **pre-commit** (tamanho, Rustfmt, OKF).
  **Verde nesse workflow não autoriza merge.**
- `.github/workflows/ci.yml`: suíte completa com **7 jobs** (prek
  pre-push, cinco replays offline e binário Linux). Dispara ao passar
  a PR de **Draft para Ready for review**, ou por `workflow_dispatch`
  para investigação. Não roda a cada commit ou push na `main`.
- `.github/workflows/wheel-uvx.yml`: builds das três wheels somente
  em `workflow_dispatch` ou chamada do workflow de release; não é
  gate de PR.
- `build-audit.yml` e `perf-audit.yml`: auditorias exploratórias sob
  demanda; deixam de rodar em cada alteração relevante da PR.
- O workflow de release continua rodando os gates canônicos na tag.

**Pré-merge obrigatório para 🐘/🦡:**

1. Sincronize a PR com `main` por merge (nunca rebase/force-push);
   verifique `behind=0`.
2. Terminada a implementação, marque **Ready for review**. Se já
   estiver Ready, converta para Draft e marque Ready de novo. Essa
   transição dispara o CI completo na PR.
3. Exija **success explícito, nunca skipped**, em todos os **7 jobs**
   do último CI completo do evento `pull_request`, no `head_sha`
   atual, além do sucesso do CI rápido. Verifique escopo e
   `--confirmar` fechado. A execução manual é útil para diagnosticar,
   mas não substitui a rodada do evento `pull_request`.
4. Havendo novo commit ou mudança da `main` que exija sincronização,
   repita a transição e todos os checks. Integre por merge commit de
   dois pais somente depois de todos verdes.

Um job pulado pode aparecer como aprovado no GitHub: para integração,
`skipped`, `cancelled` e resultados ausentes **não** contam como
gates verdes. A `main` estava sem proteção de branch no momento desta
alteração; o protocolo deve ser cumprido pelo integrador até que uma
regra de proteção seja habilitada.

Os workflows usam `contents: read`, salvo permissões específicas
de publicação. Sem acesso ao SEI.

## Links locais

O hook `links-locais` roda o `okf-parser` (o mesmo do hook `okf`) e falha
se algum link Markdown local não resolver (OKF101), fora de `.agents/`,
que guarda skills de terceiros. Frontmatter e tipos OKF ficam com o hook
`okf` (`--require-spec`). Os antigos `check_design` e `check_evidence`
saíram (#679).

## Ferramentas de verificação

`check_size` pertence ao membro Rust `xtask`, sem dependências. O check de tamanho não deve compilar `ies`
(`openssl`, `zip` ou `clap`) apenas para verificar os arquivos. Todos
os caminhos OKF e de contratos são resolvidos na raiz do workspace.

Para medir gargalos sem especular, execute `cargo build --timings` num
executor adequado; não mantenha o HTML de timings versionado.

A auditoria de build sob demanda (`build-audit.yml`, com inputs
`base` e `head`) usa `scripts/build_audit.py` para compilar base e proposta no mesmo runner,
em cópias descartáveis e com diretórios `target` isolados: build limpo,
sem mudança e com comentário sintético em `processo_obter/fluxo.rs`.
O relatório lista dependências internas do workspace e diferenças de tempo.
A ordem base→proposta e a variabilidade do cache tornam os números
exploratórios, não uma prova de aceleração. Nunca faz requisições ao SEI.

Primeira fronteira: `ies-core` contém a identidade de instalação e a versão
declarada (apenas `std`); o antigo `src/instalacao.rs` é compatibilidade de
caminhos internos. Próximas extrações exigem avaliar o grafo e a invalidação
incremental antes de acrescentar novos crates.

## Compatibilidade da saída JSON

Os goldens `tests/fixtures/caixa-v2.json` (resumida) e
`tests/fixtures/caixa-detalhada-v2.json` (detalhada) são comparados byte a
byte. Alterações intencionais de saída atualizam o golden e o contrato OKF
na mesma PR.
O domínio tipado pode evoluir sem gerar chaves JSON acidentalmente.

## OKF

Todo Markdown autoral exige frontmatter com `type` e Spec correspondente
em `.okf/specs/{slug}.md`; o gate exige `--normative-spec`.
`Cassette`, `OperationEvidence` e `SEIObservation` são categorias de
**documentos**, não classes, serviços ou camadas do runtime.
A versão do parser é fixada em `0.48.2`, ferramenta externa de CI;
o executável IES não depende de Python.

## Arquivos

Máximo de 20.000 bytes UTF-8 por arquivo texto. Arquivos binários são
ignorados. A única isenção são os arquivos `.lock` (como `Cargo.lock`):
são gerados pelo gerenciador de pacotes e não podem ser divididos
(decisão do proprietário, 03/11/2026). Nada de outras exceções
automáticas para gerados.
Arquivos que precisem superar o limite exigem reconsideração explícita da
política — nunca truncamento ou minificação para contornar o check.


## E2E cassette-first sem runner Gherkin (11/11/2026)

O teste Cucumber/Gherkin e o arquivo `.feature` foram excluídos, assim
como suas dependências de desenvolvimento. A suíte Rust existente continua
executada por `cargo test --locked --workspace --all-targets`; E2Es de
integração devem exercitar o binário real e usar cassettes originais, com
confronto separado do anti-IES com a fonte. Testes unitários não são
excluídos automaticamente. Não criar um gate especial de BDD nem
aceitar somente resultados sintéticos como fidelidade às cassettes.

Consultar [skill](../.agents/skills/cassette-e2e/SKILL.md) e
[ciclo](anti-ies/ciclo-cassettes.md).
