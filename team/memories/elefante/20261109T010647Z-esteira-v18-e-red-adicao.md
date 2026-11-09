---
type: AgentMemory
title: "Criação v17 integrada; atribuição precisa v18; RED de adição confirmado"
description: "Merges #518/#520/#522 reconciliados; #521 colide em v17, #523 requer merge da main, #524 obteve RED real; critérios #245 e #273 atualizados."
agent: elefante
timestamp: "2026-11-09T01:06:47Z"
tags: [integracao, anti-ies, sqlite, bdd, ci, antichurn]
---

# Retomar

**Fatos:** a `main` observada em `03c9c35` inclui [#518](https://github.com/franklinbaldo/ies/pull/518) (prévia de criação, SQLite v17), [#520](https://github.com/franklinbaldo/ies/pull/520) (invariantes tipadas de remoção) e [#522](https://github.com/franklinbaldo/ies/pull/522) (recusa de documento sem `src`). Marquei [#245](https://github.com/franklinbaldo/ies/issues/245) BDD-245-01 como PASS sintético, preservando 02/03 pendentes. Em [#273](https://github.com/franklinbaldo/ies/issues/273) corrigi critério legado: `--motivo` ausente é erro, texto fornecido é opaco sem `trim`, conforme contrato.

**PRs ativas:** [#521](https://github.com/franklinbaldo/ies/pull/521) tem CI 7/7 no head antigo, mas `behind=51` e `0017_processo_atribuicao.sql` colide com a v17 da #518; orientar merge verdadeiro e **v18**. [#523](https://github.com/franklinbaldo/ies/pull/523) tem 7/7 no head `4eb6ebc`, mas `behind=13`; novo CI necessário após sync. [#524](https://github.com/franklinbaldo/ies/pull/524) teve **RED real** no [CI #42543301059](https://github.com/franklinbaldo/ies/actions/runs/42543301059), Rust 2125 PASS/1 FAIL, perda indevida de vínculo na releitura; aguardar GREEN. Publiquei orientações na #521, #523 e [fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6729802175). Não houve merge nesta rodada.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388); a observação da 🦊 sobre sequências não comutativas que se tornam indistintas sob projeção reforça, como metáfora, a necessidade de releitura independente no BDD-258, sem constituir evidência do SEI.

**Próxima rodada:** consultar SHAs/CI atuais antes de integrar; corrigir colisão de migração, aproveitar GREEN do #524 e especificações BDD já existentes. Prova sintética ≠ confronto semântico com cassette ≠ live. Sem produção, credenciais ou subcódigos novos.
