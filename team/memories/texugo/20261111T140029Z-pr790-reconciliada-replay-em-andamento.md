---
type: AgentMemory
title: "PR #790 sincronizada na main; replay final foi reacionado"
description: "Merge verdadeiro da main na #790 preservou só a troca de um teste vazio por negativa Rust; CI oficial do SHA 51c287e ainda precisa concluir."
agent: texugo
timestamp: "2026-11-11T14:00:29.296Z"
tags: [antichurn, ablacao, cassette-e2e, github-actions, testes]
---

# Retomar

**Fato:** na [PR #790](https://github.com/franklinbaldo/ies/pull/790) reconciliei a `main@714b9195` com a branch própria pelo merge de dois pais [`51c287e`](https://github.com/franklinbaldo/ies/commit/51c287e052abb91152d6e722dc399436f5ea5b19), sem force-push. Validação de conteúdo: delta contra main = apenas `src/status_web.rs`, um teste de login anteriormente vazio agora recusa HTML de formulário. Não alterei produção ou outra branch. O [CI rápido #43033778739](https://github.com/franklinbaldo/ies/actions/runs/43033778739) passou; [CI oficial #43033847947](https://github.com/franklinbaldo/ies/actions/runs/43033847947) foi disparado pelo ciclo Draft→Ready e estava em execução no head novo. O [CI anterior #43024492362](https://github.com/franklinbaldo/ies/actions/runs/43024492362) passou 7/7 mas não autoriza integração do novo SHA. Relato de status no [comentário da PR](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6797794656).

**Evidência e limite:** o replay anterior verificou 495 cards/29 ZIPs e executou 24 testes do acervo + 18 `cassette_`, comprovado no log; nenhum card específico registra o HTML adversarial de login. A negativa da #790 é **sintética e local**; IES↔cassette login, anti-IES↔cassette login e CLI↔anti-IES↔SQLite login continuam **NOT_RUN**, como o live. Não transferir automaticamente o PASS de replays de outras operações para o login.

**Próxima rodada:** verificar no [run oficial #43033847947](https://github.com/franklinbaldo/ies/actions/runs/43033847947) os sete jobs, os replays executados, hashes e testes Rust no head exato; se concluídos, avaliar sincronização com main, ausência de conflito e squash da #790. Se falhar, corrigir só a branch própria. Reavaliar #781 e #792 sem editar branch de colegas. Visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), agora com 183 comentários; a conversa mais recente do Castor reforça que `body_base64` corrompido não pode virar corpo vazio.
