---
type: Policy
title: Mecânica do repositório — anti-IES, CI sob demanda e fatias WIP
description: Parte extraída do AGENTS.md com as regras de dupla implementação IES ↔ anti-IES, do CI como compilador sob demanda e da integração de fatias WIP
---

# Mecânica do repositório

Parte normativa do [AGENTS.md](../AGENTS.md), extraída sem alteração de
texto quando o arquivo atingiu o limite de 20.000 bytes. Vale com a mesma
força das regras que ficaram lá.

## Dupla implementação IES ↔ anti-IES

- **Dupla implementação IES ↔ anti-IES:** a **mesma PR** pode alterar
  `src/**`, `crates/iesei/**`, `crates/anti-ies/**` e seus testes,
  inclusive criar tipos e evoluir o esquema SQLite, **sem exigir
  cassette prévia: implementar e testar a capacidade já é progresso**.
  É exceção à divisão rígida por prefixos, não licença para editar
  branch ou arquivos com dono ativo sem coordenação. Fluxo, shape
  versionado, SQLite efêmero e critérios: [ciclo anti-IES](anti-ies/ciclo-cassettes.md).
  BDD AIES da CLI já registrado na #355 pode ser mantido pelo 🦡
  (autorização do proprietário); nos demais comandos, vale o 🐘.
  **Anti-IES é trabalho de todos os oito agentes**, sem nona rotina.
  **Cassette-first, não cassette-gated:** BDD → cards/ZIPs existentes →
  SQLite → IES ↔ anti-IES ↔ CLI → confronto com a cassette. Sem rede
  local, use o [workflow](../.github/workflows/inspecionar-cassettes.yml)
  via Actions/arquivo temporário na PR ([guia](cassettes.md)).
  Sem cassette aplicável, implemente hipótese/fixture mínima; não espere
  REA/live. PASS sintético ≠ paridade cassette ≠ live. 🦅 não veta
  por ausência; 🦡 evita churn; 🐘 coordena. Respeite BDD e ownership.
  **REA / emulação seletiva:** efeitos no SQLite; HTML, URLs,
  formulários e tokens são projeções sintéticas. Dispensar mecanismos
  internos não observáveis; separar fatos, hipóteses e desconhecidos.
  REA em HAR/artefato/runtime autorizado; não substitui Rust nem
  cria gate ([guia](anti-ies/rea-engenharia-reversa.md),
  #495–#497).

## CI sob demanda

- **Sem Rust no sandbox, o CI completo é o compilador sob demanda.**
  O [CI rápido](../.github/workflows/ci-rapido.yml) roda apenas
  pre-commit em atualizações de PR; o [CI completo](../.github/workflows/ci.yml)
  roda ao passar a PR de Draft para Ready for review (ou manualmente
  para diagnóstico). Não executar 7 jobs e 3 wheels a cada commit;
  consulte a [política de CI](ci.md).
  - **RED:** faça push dos testes novos antes do código. Acione a
    suíte completa por transição Draft → Ready (volte a Draft se
    necessário); o hook "Suíte Rust" mostra os testes falhos.
    Registre o run e a contagem. Não declare RED sem executá-lo.
  - **Binário:** o job "binário ies" do CI completo publica o artifact
    `ies-linux-x86_64-<sha>` (`ies` em release e `ies.sha256`,
    7 dias), compilado no ubuntu-22.04 (glibc ≥ 2.35, libssl3).
    Baixe-o pelo GitHub (API de artifacts do run ou
    `gh run download`), confira SHA-256 e rode no sandbox:
    comandos offline, replay e ensaio ao vivo em treinamento.
  - **Sandbox sem rede:** cassettes pelo workflow reutilizável
    [inspecionar](../.github/workflows/inspecionar-cassettes.yml);
    live em treinamento por workflow temporário, removido antes do Ready.

## Fatia WIP

- **Fatia WIP entra na `main`.** Ao fim da sessão, a fatia com gates
  verdes é integrada e a PR fecha; a issue segue aberta, e a próxima
  sessão abre PR nova a partir da `main`. Testes **novos** da fatia que
  ainda não passam levam `#[ignore = "wip #<issue>: <o que falta>"]`; a
  sessão seguinte começa tirando os da sua issue. O hook `wip-ignores`
  recusa `#[ignore]` sem motivo, `wip` sem issue e teste que já passava
  na base marcado como pendente. Rustfmt, Clippy, OKF e os demais gates
  não têm pendência: passam sempre. Escrita incompleta fica fechada
  (`--confirmar` recusa antes da rede).
