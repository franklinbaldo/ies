---
type: AgentMemory
title: "Revisão de anotação, comentários, marcadores e identidade documental"
description: "Pareceres #654/#655/#656/#657: confirmação sintética, gates atuais, concorrência SQLite v33 e correção HTTP, com limites de prova explícitos."
agent: gaviao
timestamp: "2026-11-10T09:51:19Z"
tags: [revisao, bdd, anti-ies, cassettes, sqlite, ci, marcadores, comentarios, anotacao, identidade, cafezinho]
---

# Para retomar

**#654:** [parecer `8404696`](https://github.com/franklinbaldo/ies/pull/654#issuecomment-6764175362). BDD-ANOT-02 PASS sintético no CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42787291857): CLI real, POST único, releitura e 403 pós-POST com saída 3. A PR estava behind=50 na última revisão; merge verdadeiro da main e novo CI antes de integrar. Cassette semântico bilateral e live NOT_RUN.

**#655:** [parecer `e6f240f`](https://github.com/franklinbaldo/ies/pull/655#issuecomment-6764175974) atualizado no mesmo comentário após o [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42793350367). O head anterior falhou em `crates/iesei/tests/banco_cassettes.rs:37` porque v33 não era reconhecida pelo assert legado; corrigido. BDD-257-01 sintético tem CLI real, TLS, anti-IES, SQLite e recusa lista parcial/desconhecida; bilaterais/live NOT_RUN. Atrás da main (behind=2) e concorre por v33 com #656.

**#656:** [parecer `4ca8e55`](https://github.com/franklinbaldo/ies/pull/656#issuecomment-6764176717) tinha prévia funcional hipotética e conflito SQLite; [novo parecer `09a476b`](https://github.com/franklinbaldo/ies/pull/656#issuecomment-6764250984) acompanha ampliação à confirmação. **Falhas demonstradas no CI rápido #42793876089:** `tests/aies_marcadores_cli.rs` tem **20.701 bytes** (limite 20.000) e Rustfmt falha nele e em `src/processo_marcar/catalogo.rs`; separar o teste por responsabilidade e formatar, sem diminuir cobertura. #655 e #656 propõem ambos SQL v33, só uma pode ocupar essa versão. Manifesto temporário de inspeção ainda no diff. Novo fluxo confirmatório não tem PASS no head reprovado; não confundir inspeção de hash com cassette semântico.

**#657:** [parecer inicial `91783af`](https://github.com/franklinbaldo/ies/pull/657#issuecomment-6764177377) e [parecer do head `40d94af`](https://github.com/franklinbaldo/ies/pull/657#issuecomment-6764251654). O CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42793508917) passou no head novo. Teste específico é anti-IES HTTP ↔ SQLite, não CLI E2E; impede desvio de rota por `id_documento` duplicado. Main avançou behind=2, exige nova sincronização.

**Provas:** nível 1 sintético onde efetivamente executado; integridade UnB/GO não é nível 2 sem confronto semântico bilateral; live/treinamento NOT_RUN pelo Gavião. Não vetar uma hipótese apenas por faltar cassette/live. Nenhuma implementação, merge, credencial ou acesso SEI pelo revisor.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente a [observação da 🐢 sobre comparar a pasta completa na #653](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6763343348). Um teste pode demonstrar ausência em dois nomes e ainda deixar outro arquivo escapar: medir o estado completo é evidência mais forte. Hipótese de continuidade de teste, não observação do SEI.

**Retomar:** revisar apenas heads ainda não comentados; conferir gates oficiais SHA a SHA, tratar primeiro os dois problemas demonstrados na #656 e a concorrência v33; não interferir nos branches dos implementadores.
