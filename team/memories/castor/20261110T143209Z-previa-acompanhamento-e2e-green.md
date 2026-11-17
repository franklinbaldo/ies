---
type: AgentMemory
title: "Acompanhamento BDD-348-01/03: fotografia explícita e 7 gates verdes"
description: "PR #666 reconciliada por merge verdadeiro, marcada Ready e CI oficial 7/7 no SHA 0eb5738; CLI Rust e anti-IES/SQLite v35 exercitados sinteticamente; replay bilateral e live pendentes."
agent: castor
timestamp: "2026-11-10T14:32:09Z"
tags: [ies, anti-ies, acompanhamento, bdd, sqlite, cassettes, ci]
---

# Retomar

**Fato:** a [PR #666](https://github.com/franklinbaldo/ies/pull/666) integrou `main@790d852` à branch por commit de merge verdadeiro `0eb5738` (dois pais; a diferença da base era só uma memória da Raposa). O [CI oficial #42834401155](https://github.com/franklinbaldo/ies/actions/runs/42834401155) passou **7/7** no SHA `0eb5738`, incluindo `cargo test --workspace --all-targets` pelo prek, e o [CI rápido #42834375246](https://github.com/franklinbaldo/ies/actions/runs/42834375246) passou. A PR saiu de Draft para Ready. Regressão sintética da CLI real + HTTPS loopback + anti-IES + SQLite v35 cobre G+H, somente H, lista explicitamente vazia e lista não observada, sem POST de escrita.

**Limite:** [inspeção UnB #42823266516](https://github.com/franklinbaldo/ies/actions/runs/42823266516) verificou hashes/estrutura de 97 interações, **não** replay semântico bilateral; este e treinamento/live permanecem `NOT_RUN`. BDD-348-02, confirmação por formulário servido e releitura causal, ainda pendente. Não confundir `GET` com ausência de efeitos fora da bancada.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388); a discussão da 🐘 sobre `trim()` em texto composto só por espaços é outro exemplo de não substituir observação por interpretação.

**CONTINUAR:** após a publicação desta memória na `main`, sincronizar novamente a #666 por merge verdadeiro, validar o SHA final e registrar handoff para 🦅/🐘/🦡. Não editar BDD canônico nem declarar paridade/live.
