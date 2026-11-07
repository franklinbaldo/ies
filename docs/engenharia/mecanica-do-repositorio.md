---
type: Policy
title: Mecânica de desenvolvimento e CI do IES
description: Regras normativas de tamanho, fatias WIP, pre-push, CI e artefatos
---

# Mecânica de repositório — fonte normativa

Regras movidas integralmente do AGENTS.md. **Toda rotina implementadora**
deve lê-las junto ao AGENTS.md antes de modificar, testar ou publicar uma PR;
a extração não flexibiliza nem substitui os gates existentes.

## Mecânica do repositório

- Nenhum arquivo texto passa de 20.000 bytes (exceto `.lock`). Quando
  uma mudança deixar um arquivo acima de ~16.000 bytes, **divida-o na
  mesma PR**, por responsabilidade: o original fica com o núcleo e uma
  seção de links para as partes, e cada parte vira um arquivo próprio
  com frontmatter OKF (`contracts/cli.md` → `contracts/cli/*.md`,
  `docs/erros-tipados.md` → `docs/erros-tipados/*.md`). Não encolha
  texto, tabela ou código para caber; não espere o gate falhar.
- Ao trabalhar em paralelo com outros agentes, edite só os arquivos do
  seu prefixo; os dos outros você lê. Se uma peça compartilhada precisar
  mudar, faça a mudança mínima e explique na PR.
- `xtask` só tem verificadores de infraestrutura, sem dependências.
- **Sem Rust no sandbox, o CI é o compilador.** Faça push no branch da
  PR e leia o resultado do [CI](../../.github/workflows/ci.yml):
  - **RED:** faça push dos testes novos antes do código. O hook "Suíte
    Rust" falha, e o log mostra quantos testes falharam. Esse é o RED
    observado; registre o run e a contagem.
  - **Binário:** o job "binário ies" publica o artifact
    `ies-linux-x86_64-<sha>` (`ies` em release e `ies.sha256`, 7 dias),
    compilado no ubuntu-22.04 (glibc ≥ 2.35, libssl3). Baixe-o pelo
    GitHub (API de artifacts do run ou `gh run download`), confira o
    SHA-256 e rode no sandbox: comandos offline, replay e ensaio ao vivo
    em treinamento.
  - **Sandbox sem rede para o GitHub ou para o SEI:** rode o ensaio num
    workflow temporário (`workflow_dispatch` ou `push` em `codex/**`) que
    use esse binário, e remova o workflow antes de a PR sair de DRAFT.
- **Fatia WIP entra na `main`.** Ao fim da sessão, a fatia com gates
  verdes é integrada e a PR fecha; a issue segue aberta, e a próxima
  sessão abre PR nova a partir da `main`. Testes **novos** da fatia que
  ainda não passam levam `#[ignore = "wip #<issue>: <o que falta>"]`; a
  sessão seguinte começa tirando os da sua issue. O hook `wip-ignores`
  recusa `#[ignore]` sem motivo, `wip` sem issue e teste que já passava
  na base marcado como pendente. Rustfmt, Clippy, OKF e os demais gates
  não têm pendência: passam sempre. Escrita incompleta fica fechada
  (`--confirmar` recusa antes da rede).
- Antes de qualquer PR:
  `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
  (o [CI](../../.github/workflows/ci.yml) roda os mesmos gates; não crie outro
  conjunto de checks).
