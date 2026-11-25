---
type: AgentMemory
title: "BDD-60-05: recusas humanas pelo NUP estão cobertas no E2E"
description: "PR #697 no head 125debec: dois negativos da CLI humana sobre anti-IES/SQLite passam no CI 7/7; card UnB prova integridade, não a recusa. Aguardar Gavião e integração."
agent: raposa
timestamp: "2026-11-10T19:41:47Z"
tags: [ux, bdd-60-05, documento, anti-ies, sqlite, cassette-first, ci]
---

# Próxima leitura

Fato: [PR #697](https://github.com/franklinbaldo/ies/pull/697) testa a CLI Rust real com `--processo NUP` para número repetido e número de outro processo. As mensagens humanas diferem, ambas falham sem tentar corpo alternativo. [CI oficial #42886603194](https://github.com/franklinbaldo/ies/actions/runs/42886603194) passou 7/7 no SHA `125debec` e [rápido #42886554720](https://github.com/franklinbaldo/ies/actions/runs/42886554720) passou. Sem RED funcional, pois a CLI já recusava corretamente.

Limite: [inspeção da cassette UnB #42883498393](https://github.com/franklinbaldo/ies/actions/runs/42883498393) valida hashes e 15 interações, mas não esses negativos. Confronto semântico bilateral específico e treinamento NOT_RUN. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) lembra a diferença entre identidade publicada e tolerância de busca; contexto de NUP não é escolha livre.

Retomar: conferir SHA e CI após incorporar mudanças da `main`, buscar revisão independente do 🦅 Gavião e entregar ao 🐘/🦡. Não criar cenários sem `--processo` sem BDD previamente escrito pelo 🐘.
