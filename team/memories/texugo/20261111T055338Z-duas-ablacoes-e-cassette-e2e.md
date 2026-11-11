---
type: AgentMemory
title: "Duas ablações integradas sem revalidar memórias OKF"
description: "PRs #755 e #756 integradas após CI 7/7 e revisão; #750 exige conciliação funcional e #757 ainda antecede a remoção de Gherkin."
agent: texugo
timestamp: "2026-11-11T05:53:38Z"
tags: [antichurn, ablacao, integracao, ci, cassette-e2e, erros-tipados]
---

# Retomar

**Fatos:** li AGENTS.md, ficha, ciclo cassette-first, skill E2E, cinco memórias recentes e [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) (169 comentários nas duas páginas). A [#758](https://github.com/franklinbaldo/ies/pull/758) já havia removido runner Gherkin, feature, dependências Cucumber e 87 pacotes; confirmei ausência dos caminhos antigos na main. A decisão vigente é **E2E cassette-first**, sem novos cenários Gherkin nem descarte cego das regressões Rust.

Integrei [#755](https://github.com/franklinbaldo/ies/pull/755) em [f14e063](https://github.com/franklinbaldo/ies/commit/f14e0634146e8b5d5d6af4be1eb098dcac550051) e [#756](https://github.com/franklinbaldo/ies/pull/756) em [b049175](https://github.com/franklinbaldo/ies/commit/b04917559b88fcf52a30e96c6bf0edf8a3f41d5b). Nos heads exatos: CI rápido PASS e oficial 7/7 PASS ([#42967604380](https://github.com/franklinbaldo/ies/actions/runs/42967604380), [#42967971205](https://github.com/franklinbaldo/ies/actions/runs/42967971205)), revisão 🦅 favorável. O delta exclusivo da main após a última conciliação era memórias ou arquivos disjuntos; dispensei sincronização cerimonial, não gates. Atualizei [#720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6788255029) e [#719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6788267763), ainda abertas para outros sítios.

**Evidência:** regressões Rust técnicas PASS; não declarei E2E novo específico, confronto semântico bilateral de cassette ou live para as causas de erro (**NOT_RUN**). #755 inclui reparo da colisão de diretórios TLS sem retry; #756 liga causas existentes por `#[source]`, sem novas camadas ou subcódigos.

**Fila:** [#750](https://github.com/franklinbaldo/ies/pull/750) tem 7/7 no head, mas a main voltou a alterar `src/processo_prazo_cli.rs` depois da base, exigindo merge real e CI do conjunto; [#757](https://github.com/franklinbaldo/ies/pull/757) ainda está baseada no método anterior à #758, portanto não reaproveitar CI antigo na integração sem conciliação/novo head. [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft com migração antiga. Nenhum acesso ao SEI, credencial, subcódigo ou gate criado.

**Ideia do Cafezinho:** manter separadas falha da bancada, conteúdo de captura e hipótese sintética. Não transformar teste que atingiu uma fronteira diferente em prova de paridade.
