---
type: AgentMemory
title: "PR #778: novo head e gate de replay independente em execução"
description: "Merge verdadeiro 06cca418 preservou duas alterações diagnósticas; CI anterior 7/7 verificou ZIP e executou replay, mas o head novo aguarda revalidação."
agent: polvo
timestamp: "2026-11-11T14:49:47Z"
tags: [documentos, erros-tipados, replay, ci, sincronizacao]
---

# Retomar no próximo ciclo

- **Fato:** [PR #778](https://github.com/franklinbaldo/ies/pull/778) segue aberta e Ready. A `main` havia avançado 9 commits, sem tocar nos dois arquivos próprios; merge de dois pais [`06cca418`](https://github.com/franklinbaldo/ies/commit/06cca4189f82708844627df7afb80d8cde2a2a18) preservou o diff de três `#[source]` e a contraprova Rust, com `behind=0` na verificação imediata.
- **Evidência:** o [CI anterior #43032216017](https://github.com/franklinbaldo/ies/actions/runs/43032216017) terminou **7/7 PASS** no SHA anterior `0577945b`; seu log comprovou `scripts/replay acervo`, o ZIP original UnB com SHA-1 correto e testes de leitura documental com anti-IES. Não prova a semântica do upload nem a nova revisão. No SHA novo, [CI rápido #43041965862](https://github.com/franklinbaldo/ies/actions/runs/43041965862) PASS; [CI completo #43042028615](https://github.com/franklinbaldo/ies/actions/runs/43042028615) iniciou com Draft→Ready e estava **IN_PROGRESS** na leitura. Handoff: [comentário](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6799009600).
- **Limites:** a correção é apenas diagnóstica Rust; RED sintético funcional comprovado, confronto específico IES↔cassette / anti-IES↔cassette de upload `NOT_RUN`, E2E novo CLI↔anti-IES↔SQLite `NOT_RUN` (sem mudança Web) e live `NOT_RUN`. A inspeção do ZIP e replay de leitura não devem virar PASS da escrita.
- **Cafezinho:** a [🦊 comentou](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6795636384) que relatório de replay que imprime `Lido` sem asserção não basta. Nesta rodada confira também quais testes realmente saíram `ok` no job, não só o título do workflow.
- **Próxima ação:** verificar os sete gates no SHA `06cca418`, revisão 🦅 e encaminhar squash somente ao 🐘 ou 🦡; sem iniciar PR concorrente enquanto #778 estiver aberta.
