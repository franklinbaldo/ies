---
type: AgentMemory
title: "Cancelamento temporal do histórico com CLI real e SQLite"
description: "PR #604 protege a transição árvore para histórico após cancelamento local; CI completo 7/7 verde, duas cassettes UnB conferidas, corrida externa não observada."
agent: tartaruga
timestamp: "2026-11-09T20:51:17Z"
tags: [bdd60, anti-ies, cassette-first, cli, http, sqlite, ci]
---

# Para retomar

**Fato:** [PR #604](https://github.com/franklinbaldo/ies/pull/604) foi aberta sobre `main@597ad0ee`, Ready, sem merge, com teste novo `bdd_60_05_cli_recusa_historico_cancelado_apos_arvore_sem_fallback`. A CLI real, usando TLS local e anti-IES HTTP sobre SQLite compartilhado, lê uma árvore com histórico oferecido; imediatamente antes do GET da lista, a origem sintética cancela o documento. O teste confere erro 403/exit 1 com e sem `--ocultar-inacessiveis`, nenhuma versão recuperada, um só GET da lista, nenhum GET de corpo, ausência de POST mutador e persistência do cancelamento restrita ao DOC certo. Código produtivo não mudou; **RED funcional não executado**, só regressão de proteção existente.

**CI:** [completo #42696206241](https://github.com/franklinbaldo/ies/actions/runs/42696206241) passou **7/7** no SHA exato `30f3fc5dfe6708ff98a2c961ea5a337b9c0644c9`; [rápido #42696080163](https://github.com/franklinbaldo/ies/actions/runs/42696080163) verde. Rustfmt e Clippy exigiram correções intermediárias, feitas sem supressões; despacho temporal dos testes concentrado em `tests/support/documento_interprocessos_transicoes.rs`.

**Cassette-first:** [inspeção #42694571286](https://github.com/franklinbaldo/ies/actions/runs/42694571286) verificou SHA-1 do ZIP e SHA-256 dos membros para `documento-mover-cancelar-unb` (60 interações) e `liberacoes-e-versoes-leitura-unb` (27). Isso comprova integridade e operações registradas, **não** a corrida de cancelamento entre árvore e GET de versões. Provas: **IES ↔ anti-IES: PASS sintético neste teste**; **ambos ↔ cassette para corrida: NOT_RUN**, pois não gravada (confrontos anteriores UnB seguem parciais); **treinamento/live: NOT_RUN**.

**Cafezinho:** li #388 e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6750983550) a diferença entre uma ação listada e uma ação ainda autorizada. Analogia não equivale a observação.

**Próximo:** [handoff na #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6751118077); 🦅 revisão, 🐘/🦡 integração. Após esta memória entrar em `main`, a branch ficará atrás em um commit documental e deverá ser sincronizada por merge verdadeiro antes de integrar. Sem produção, credenciais, novos subcódigos ou mudança de TLS.
