---
type: AgentMemory
title: "PR #755 sincronizada com a main sem Gherkin; colisão TLS resolvida"
description: "Merge verdadeiro, correção do caminho temporário de E2E e CI completo 7/7 no head bee2061; falta re-review e integração."
agent: tartaruga
timestamp: "2026-11-11T05:41:52Z"
tags: [issue720, issue719, cli, anti-ies, ci, testes-paralelos, cassettes]
---

# Retomada

A [PR #755](https://github.com/franklinbaldo/ies/pull/755) permaneceu aberta e Ready. Merge verdadeiro [2a91bb7](https://github.com/franklinbaldo/ies/commit/2a91bb7ea6c9c88c1a617065eec4351205a5f5d4) incluiu a decisão de eliminar Gherkin sem alterar a ablação Rust da tela de troca. O CI do merge [#42966549654](https://github.com/franklinbaldo/ies/actions/runs/42966549654) encontrou uma colisão de pasta temporária `AlreadyExists` num E2E paralelo de comentários, não uma falha funcional de navegação. Adicionei contador atômico e teste de mesmo timestamp ao helper TLS no [commit bee2061](https://github.com/franklinbaldo/ies/commit/bee20613c217d7e67d92b46349e77f473c1372b0).

**GREEN comprovado:** [CI completo #42966985129](https://github.com/franklinbaldo/ies/actions/runs/42966985129) 7/7 e [rápido #42966957829](https://github.com/franklinbaldo/ies/actions/runs/42966957829) PASS no head exato `bee2061`. O teste da anomalia de atributo duplicado continua **sintético**; três cassettes tinham hashes conferidos anteriormente, mas não capturam duplicidade. Confronto bilateral específico, novo E2E da anomalia e live: NOT_RUN. Não usar o sucesso do simulador como prova externa.

**Próximo:** re-review focal 🦅, integração 🐘/🦡, verificar se a main avançou após esta memória. [Handoff #720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6788104510); [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) revisitado, comentário opcional. Sem credenciais, produção, subcódigos novos, alteração TLS ou retry de escrita.
