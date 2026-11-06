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

## Auditoria de dependências (OSV)

O hook `auditoria-osv` roda `scripts/osv_audit.py` (só biblioteca padrão).
Ele lê o `Cargo.lock`, pega todo pacote cuja `source` é o registro
crates.io e faz **uma** consulta em lote a `https://api.osv.dev/v1/querybatch`
(páginas extras só para pacote com mais de 1.000 avisos). Falha listando
pacote, versão e ids dos avisos (RUSTSEC e GHSA, como o OSV os relata).

Avisos aceitos ficam em `scripts/osv_allowlist.json`: cada entrada exige
`id`, `package` e `justification` escrita (por que não se aplica ao IES e
quando revisar); entrada sem justificativa derruba o hook. A lista começa
vazia. Entrada que o OSV deixou de relatar gera aviso, não falha.

Sem rede (ou resposta irreconhecível), o comportamento depende de `CI`:
com a variável definida e não vazia (o GitHub Actions define `CI=true`),
o hook **falha**; sem ela, imprime `NOT_RUN` e passa. O trade-off é
deliberado: o pré-push local não fica refém da rede, ao custo de um push
offline não ser auditado; o CI remoto, que roda o mesmo comando, é a
barreira que não pode ser pulada em silêncio. O resultado depende da base
do OSV no momento: um aviso novo pode derrubar um commit que ontem passava.

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

A auditoria de build em PRs que alteram a arquitetura (`build-audit.yml`)
usa `scripts/build_audit.py` para compilar base e proposta no mesmo runner,
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

## Cobertura documental dos subcódigos

O hook `subcodigos` executa `python3 -I scripts/subcodigos.py`, só stdlib,
sem rede, sobre os arquivos atuais de `src` e `crates`. Não mantém lista
fixa de emissores. Tokeniza comentários simples/aninhados, strings normais
e raw, escapes, caracteres, lifetimes e delimitadores equilibrados;
lê literais dentro de funções `subcodigo` e `subcodigo_*`, além de valores
literais de chaves JSON `subcodigo`. Comentários, funções de mensagem,
arquivos de testes/acervo e itens inline `#[cfg(test)]` são excluídos.
Funções delegadas contribuem por suas próprias definições, sem exigir
repetição do literal no chamador. Declarações de trait sem corpo não
consomem a função seguinte. Fonte malformada ou literal não classificável
em função de subcódigo faz o hook falhar para revisão.

A documentação vem da coluna `Subcódigo`/`Subcódigos` das tabelas de
`docs/erros-tipados.md` e `contracts/cli/*.md`, inclusive os complementos
“Falhas tipadas”. Curingas, prosa, campos de apresentação e exemplos em
blocos de código não dão cobertura. Um literal emitido sem tabela falha;
um documentado sem literal detectado apenas avisa. Aviso não demonstra
aposentadoria: há chamadas que passam literais por parâmetros de helpers.

`scripts/subcodigos_permitidos.json` é uma lista inicialmente vazia.
Cada exceção exige `subcodigo` e `justificativa` não vazia, sem duplicatas;
a justificativa deve explicar por que dispensar documentação e quando
revisar. As exceções são mostradas no diagnóstico. Não usar para mascarar
um contrato que falta escrever.

É análise léxica conservadora, não AST nem prova de fluxo: não expande
macros, não resolve tipos/delegações, chamadas indiretas ou strings
construídas; não interpreta `cfg` arbitrário nem comprova alcance pela
CLI. Os testes sintéticos entram no hook `scripts-python` já existente.
Não há workflow ou conjunto alternativo de checks.
