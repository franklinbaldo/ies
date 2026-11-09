---
type: AgentMemory
title: "Revogação entre árvore e GET de conteúdo exige releitura causal"
description: "PR #550 testa CLI real diante de cancelamento no SQLite entre href publicado e GET; CI 7/7, paridade externa ainda não demonstrada."
agent: tartaruga
timestamp: "2026-11-09T09:29:40Z"
tags: [bdd60, anti-ies, cli, navegacao, sqlite, cassette, http]
---

# Para retomar

**Fato:** [PR #550](https://github.com/franklinbaldo/ies/pull/550) adicionou teste E2E sintético de BDD-60-05: a árvore publica o href enquanto DOC está disponível; o mesmo SQLite cancela DOC antes de atender `documento_visualizar`; a CLI real recebe HTTP 403, exit 1, sem conteúdo parcial, retry ou escrita. Estado final verificado por leitura direta do banco. Código de produção inalterado. [CI completo #42593239727](https://github.com/franklinbaldo/ies/actions/runs/42593239727) **7/7 PASS** no SHA `dfe40fc`; rápido [#42593190462](https://github.com/franklinbaldo/ies/actions/runs/42593190462) PASS. Falhas intermediárias foram Rustfmt/Clippy, **não RED funcional**. Branch incorporou a main por merge verdadeiro.

**Limite da prova:** [inspeção Actions #42592387297](https://github.com/franklinbaldo/ies/actions/runs/42592387297) verificou hashes e estrutura de 2 ZIPs GO/UnB, **107 interações HTTP**. A corrida não consta da captura. IES↔anti-IES PASS sintético; ambos↔cassette semântico NOT_RUN; treinamento NOT_RUN; REA NOT_RUN.

**Cafezinho:** a conversa sobre o mensageiro e a folha `02` sugere que um endereço publicado atesta somente que o caminho foi anunciado, não que o conteúdo continue acessível no momento do GET. Analogia, não evidência do SEI.

**Próximo:** revisão 🦅 e integração 🐘/🦡 da PR #550. Issue #60 continua aberta; evitar duplicar este caso e priorizar confronto independente com observações originais ou lacuna do BDD existente.
