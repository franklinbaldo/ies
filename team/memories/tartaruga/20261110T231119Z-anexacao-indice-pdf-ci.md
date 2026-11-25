---
type: AgentMemory
title: "BDD-60-05: anexação desloca PDF na árvore sem mudar identidade"
description: "PR #712 acrescenta teste HTTP de árvore com pasta, PDF e anexação no mesmo SQLite; inspeção UnB 75 HTTP com hashes PASS, gates do head 6e42a225 ainda em andamento."
agent: tartaruga
timestamp: "2026-11-10T23:11:19Z"
tags: [anti-ies, bdd60, sqlite, anexacao, navegacao, cassette-first, ci]
---

# Para retomar

**Fatos:** [PR #710](https://github.com/franklinbaldo/ies/pull/710) foi integrada antes da rodada. Na [PR #712](https://github.com/franklinbaldo/ies/pull/712), aberta e Ready no head `6e42a225`, acrescentei apenas um teste técnico em `crates/anti-ies/tests/arvore_indice_sqlite.rs`: a árvore inicia com PASTA/AGUARDE/PDF, a anexação de processo é persistida no mesmo SQLite e o segundo GET deve publicar PASTA, AGUARDE, PROCESSO_ANEXADO, PDF em índices distintos. POST 405 e GET posterior comprovam ausência de mutação da leitura/recusa. Não há código de produção novo, novo BDD ou RED funcional comprovado.

**Cassette-first:** [inspeção oficial #42920191914](https://github.com/franklinbaldo/ies/actions/runs/42920191914) teve SHA-1/SHA-256 PASS nas cassettes UnB `documento-numero-e-busca` (15 HTTP) e `documento-mover-cancelar` (60 HTTP); o manifesto temporário foi removido. Esses cards **não** capturam a transição pasta + anexação + PDF. Confronto semântico bilateral e live: NOT_RUN, não equiparar à inspeção de hashes.

**CI no momento desta memória:** [rápido #42920401306](https://github.com/franklinbaldo/ies/actions/runs/42920401306) PASS no SHA `6e42a225`, após corrigir Rustfmt sem suprimir o hook. [Completo #42920498962](https://github.com/franklinbaldo/ies/actions/runs/42920498962) ainda em andamento, com os seis jobs não-prek já aprovados na última consulta. Teste novo HTTP: resultado pendente, novo E2E CLI binário desta variante: NOT_RUN.

**Próximo passo:** conferir no runner o job `prek` que inclui suíte Rust, corrigir eventuais falhas e só então declarar 7/7 e encaminhar revisão 🦅/integração 🐘/🦡. Com esta memória em `main`, a branch pode ficar um commit atrás; verificar e sincronizar via merge de dois pais quando necessário. Não encerrar #496 nem #60.

**Cafezinho:** li a [#388](https://github.com/franklinbaldo/ies/issues/388) e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781784325): índice na apresentação muda, identidade no banco não. Nenhuma credencial, SEI externo, subcódigo ou alteração de TLS.
