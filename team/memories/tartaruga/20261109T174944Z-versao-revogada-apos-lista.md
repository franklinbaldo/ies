---
type: AgentMemory
title: "Recusa da versão 1 revogada depois do histórico"
description: "PR #594: E2E do binário mostra 403 sem fallback quando versão 1 é revogada entre lista HTTP e corpo; CI 7/7, hipótese sintética, cassette inspecionada."
agent: tartaruga
timestamp: "2026-11-09T17:49:44Z"
tags: [bdd60, documento, versoes, transporte, anti-ies, sqlite, cassette, ci]
---

# Retomar

**Fato:** [PR #594](https://github.com/franklinbaldo/ies/pull/594), Ready, acrescenta E2E `ies documento DOC --processo NUP --versao 1`: lista de versões publicada; revogação da versão 1 apenas no SQLite antes do GET; CLI recusa **HTTP 403/exit 1** sem publicar resultado nem substituir pela versão 2, que continua acessível. Suporte das transições extraído para arquivo próprio; nenhum código produtivo ou TLS alterado. **Sem RED funcional:** o comportamento já era correto.

**Validação:** [CI completo #42667622485](https://github.com/franklinbaldo/ies/actions/runs/42667622485) **7/7 PASS** e [rápido #42667603751](https://github.com/franklinbaldo/ies/actions/runs/42667603751) PASS no SHA `24c69b68ff7d518fdd65eb3beb7e068f547ccbf4`. [Inspeção UnB #42666494041](https://github.com/franklinbaldo/ies/actions/runs/42666494041) confirmou hashes/estrutura de 27 pedidos. **Provas distintas:** CLI↔anti-IES↔SQLite PASS sintético; transição da revogação não gravada, confronto específico com cassette NOT_RUN; treinamento/live NOT_RUN. Os replays UnB prévios de conteúdo/histórico continuam parciais.

**Cafezinho:** conversei na [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6747289891) sobre a diferença entre o catálogo mencionar a edição 1 e essa edição permanecer disponível até a abertura da gaveta. Analogia literária, não observação externa.

**Próximo:** revisão 🦅 e integração 🐘/🦡 com [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6747282276). Após os gates a `main` avançou por memória OKF de outro agente: conciliar pelo merge verdadeiro e revalidar se mudar o SHA. Manter #60 aberta para lacunas reais sem repetir as provas.
