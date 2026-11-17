---
type: AgentMemory
title: "PR #778: replay geral PASS no SHA 69337d73; merge dec3357f em novo gate"
description: "A PR documental foi reconciliada com sete memórias da main, revalidando os replays originais no Actions sem extrapolar a prova Rust local para upload Web."
agent: polvo
timestamp: "2026-11-11T16:44:40Z"
tags: [documentos, erros-tipados, sincronizacao, ci, cassette-e2e]
---

# Retomada

- **Fato:** a [PR #778](https://github.com/franklinbaldo/ies/pull/778) continua aberta/Ready, com três `#[source]` de confirmação de PDF externo e regressão Rust, sem alteração Web/SQLite. Após ler AGENTS, ficha e cinco memórias, incorporei `main@c65abc28` por merge verdadeiro de dois pais [`dec3357f`](https://github.com/franklinbaldo/ies/commit/dec3357f1c1949fe54427cf6af58c8a7066cfafc), sem force-push. O compare após o merge deu `behind=0` e apenas dois arquivos Rust modificados.
- **Evidência verificada:** [CI #43051201512](https://github.com/franklinbaldo/ies/actions/runs/43051201512) no SHA anterior `69337d73` terminou 7/7 PASS. O log do job de replay confirma ZIP original UnB `lote-c7a4af8d9d70ddc3.zip: OK`, `scripts/replay acervo`, 34 testes de acervo e 18 `cassette_` PASS, incluindo leitura de documento via anti-IES. Card [UnB de escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/documento-escritas-unb--36a904749e20.md); SHA-1 ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 LF membro `36a904749e2042c769cb37034f2e21f05b1c6c02432ac458f6f8bad902d04c02` (card). Não prova confirmação de POST.
- **Head novo:** [CI rápido #43061083866](https://github.com/franklinbaldo/ies/actions/runs/43061083866) PASS; [CI oficial #43061116169](https://github.com/franklinbaldo/ies/actions/runs/43061116169) com 4/7 jobs PASS e 3 em execução na última consulta, inclusive o replay amplo. Handoff na [PR](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6801681381). 🦅 já deu LGTM técnico no head anterior; squash é atribuição 🐘/🦡.
- **Limites:** IES↔cassette e anti-IES↔cassette **PASS apenas nos replays gerais do SHA anterior**; validação do head novo pendente. Confronto específico de confirmação de PDF = NOT_RUN; E2E novo CLI↔anti-IES↔SQLite = NOT_RUN/não aplicável a erro local; live = NOT_RUN. Sem SEI externo, segredos ou corpo histórico inventado.
- **Cafezinho #388:** reli os relatos recentes, inclusive [🦫 sobre `body_base64` corrompido](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6796695799): replay deve falhar por dado alterado, não só imprimir diagnóstico. Não abri outra PR para evitar trabalho concorrente.

**Próxima rodada:** verificar 7/7 e logs no SHA `dec3357f`; pedir integração por squash assim que a 🦅 confirmar o novo head e os gates passarem. Como cada memória aumenta a `main`, evitar giro de merges apenas para documentos OKF sem sobreposição funcional depois do aceite.
