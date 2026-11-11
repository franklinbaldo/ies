---
type: AgentMemory
title: "Três ablações verdes integradas e migração de criação desatualizada"
description: "Merges #743, #744 e #742 no SHA validado; diagnóstico da #526 (esquema agora v38), alerta de hook redundante de dead code na #721."
agent: texugo
timestamp: "2026-11-11T02:52:58Z"
tags: [antichurn, ablacao, integracao, ci, causas-tipadas, migracoes, sqlite]
---

# Para retomar

**Fatos confirmados:** li instruções completas, ficha, ciclo cassette-first e cinco memórias recentes; reli os comentários do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Integrei três PRs na `main`, com LGTM técnico do 🦅 e **CI oficial 7/7 + rápido PASS no head exato**:
- [#743](https://github.com/franklinbaldo/ies/pull/743) → [`e91fa09`](https://github.com/franklinbaldo/ies/commit/e91fa099ec7671928cfdd00d5bda847f1613a990), remoção de dois `allow(clippy::too_many_arguments)` via `TelaServida`/`DestinoConsultado`, preservados POST e causas de releitura.
- [#744](https://github.com/franklinbaldo/ies/pull/744) → [`8daf9fb`](https://github.com/franklinbaldo/ies/commit/8daf9fbb729c397fc39c0b289339e14d23d328d5), teste classifica causa por enum Rust em vez de texto; RED de `downcast_ref` errado foi corrigido e GREEN 7/7 comprovado.
- [#742](https://github.com/franklinbaldo/ies/pull/742) → [`f1f4545`](https://github.com/franklinbaldo/ies/commit/f1f4545b33df69a33083e8b58776de670fc103f5), preserva `ErrorKind::NotADirectory` ao consultar destino privado, contraprova de bytes ancestrais preservados.

Sem sobreposição funcional entre as três fatias, sem CI pós-merge. Atualizei [#721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6785548062), [#718](https://github.com/franklinbaldo/ies/issues/718#issuecomment-6785548483) e [#720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6785566695); permanecem abertas para outros sítios. O hook `sem-dead-code` da #739 já foi integrado apesar de duplicar os Clippys e proibir exceção `cfg_attr(not(unix), allow(dead_code))` aceita pela #721: proposta de ablação/compatibilidade, não falha observada.

**Desbloqueio da [#526](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6785557183):** #521 já integrada, mas a branch antiga está behind 2564, Draft/conflitante. A `main` possui migrações 0001–0038, logo a fatia de criação precisa da próxima versão livre (hoje v39), merge verdadeiro, preservação de todo o esquema e revalidação E2E causal; não tocar na branch alheia.

**Limites de prova:** #743 tem suíte sintética de CLI/anti-IES existente PASS no CI, mas nenhum BDD novo; #744 teste tipado PASS; #742 teste local de filesystem PASS, E2E específico não executado. Inspeções antigas de integridade de cassettes não provam confronto semântico bilateral; este permanece NOT_RUN, assim como live. Nenhum acesso ao SEI, credenciais, produção, subcódigo ou gate criado. Cafezinho relembrou a diferença entre POST de consulta e POST mutador.
