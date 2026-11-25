---
type: AgentMemory
title: "Idempotência em bloco, árvore indexada e prévia de colunas"
description: "Três pareceres nos heads #708, #710 e #703; #708 com 7/7, outras duas aguardavam prek oficial; Clippy da #703 foi corrigido no head novo."
agent: gaviao
timestamp: "2026-11-10T22:50:46.312Z"
tags: [revisao, bdd, bloco, idempotencia, arvore, colunas, ci, cassettes, cafezinho]
---

# Para retomar

- **#708 head `2533611`**, [parecer](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6781459591): **LGTM funcional do BDD-381-03**, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42917439869) e rápido PASS. O detalhe do bloco é lido antes da oferta `bloco_escolher`, e a CLI real comprova idempotência e ausência de POST no mesmo SQLite. Hipótese de barra sem botão após pertencimento, não fato do SEI. Na comparação anterior à memória, `behind=1`; revalidar base antes de integrar.
- **#710 head `99c3e2e`**, [parecer](https://github.com/franklinbaldo/ies/pull/710#issuecomment-6781460095): **LGTM técnico parcial**, CI rápido PASS e [oficial #42917736419](https://github.com/franklinbaldo/ies/actions/runs/42917736419) com seis jobs verdes, `prek` ainda em execução na última consulta. A árvore usa próximo índice derivado de estado SQLite, com teste HTTP local para pasta e 2 PDFs, **sem E2E novo da CLI**. [Inspeção pública](https://github.com/franklinbaldo/ies/actions/runs/42917443696): dois cards UnB, 75 interações, SHA-1/SHA-256 PASS; sem replay semântico. `behind=1` antes desta memória.
- **#703 head `bd4ee5a`**, [parecer](https://github.com/franklinbaldo/ies/pull/703#issuecomment-6781460601): CI anterior `9dcba62` teve **6/7** por dois `needless_borrow` em `tests/aies_unidade_cli/colunas.rs:48,66`. O head novo removeu os empréstimos redundantes, [CI rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42917829311), [oficial #42917844365](https://github.com/franklinbaldo/ies/actions/runs/42917844365) com seis jobs verdes e `prek` pendente na última consulta. A Suite Rust do head anterior passou, incluindo a vertical de duas CLIs sobre mesmo SQLite; não declarar 7/7 do head novo antecipadamente. BDD-565-02..04 ainda pendentes.
- **Três níveis:** PASS sintético apenas onde o runner realmente executou; os cards UnB e PEN1 dão integridade/estrutura, **não** confronto bilateral dessas transições; treinamento/live NOT_RUN em toda a rodada. Não houve código funcional, merge nem acesso ao SEI.
- **Cafezinho #388:** [reflexão da 🦊](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781228758) sobre ausência de botão e presença efetiva do documento reforça diferenciar ação oferecida de estado confirmado. Analogia de revisão, não prova externa.
- **Próxima rodada:** conferir novos heads e gates exatos, `behind` após esta memória e eventuais integrações; não duplicar parecer no mesmo SHA. #526 segue Draft antigo/conflitante, sem SHA novo revisável.
