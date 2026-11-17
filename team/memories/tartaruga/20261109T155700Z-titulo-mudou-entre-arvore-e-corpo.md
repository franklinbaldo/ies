---
type: AgentMemory
title: "Corrida de identidade da CLI entre árvore e GET do documento"
description: "PR #578: novo E2E do binário recusa título documental alterado no SQLite após a árvore; CI 7/7, cassette UnB verificada e evidências classificadas separadamente."
agent: tartaruga
timestamp: "2026-11-09T15:57:00Z"
tags: [bdd60, identidade, cli, anti-ies, sqlite, tls, cassette, ci]
---

# Para retomar

**Fato:** [PR #578](https://github.com/franklinbaldo/ies/pull/578) Ready, ainda sem merge, acrescenta regressão BDD-60-05 no binário real: `ies --json documento 0011519 --processo 00001.000001/2026-01 --sem-cache` atravessa TLS de loopback e anti-IES com SQLite. A árvore oferece DOC_A, o número SEI do mesmo ID 42 muda no banco antes do GET de `documento_visualizar`, e o título servido passa a referir DOC_B. A CLI recusa com exit code 1 e causa tipada existente `titulo_do_conteudo_de_outro_documento`, sem retorno parcial, segundo GET, fallback ou POST mutador. Verificação direta do SQLite atesta a transição e preservação do documento 43. Código produtivo intacto; os guardas da CLI já existiam, por isso **sem RED funcional** nesta fatia. O suporte de estado foi separado do proxy TLS para não ultrapassar 20 KB.

**Gates:** [CI completo #42648911945](https://github.com/franklinbaldo/ies/actions/runs/42648911945) **7/7 PASS** e [CI rápido #42648894033](https://github.com/franklinbaldo/ies/actions/runs/42648894033) PASS no SHA `32cc66a287c74f20dc67052f51e09cf4a6c1b0ae`. O Clippy rigoroso exigiu remover borrões de empréstimo após a extração do helper; corrigido sem supressão. A `main` avançou com uma memória documental da revisão 🦅 após esses gates, a conciliar pela integração.

**Cassette-first:** [Actions #42647547164](https://github.com/franklinbaldo/ies/actions/runs/42647547164) conferiu SHA-1/SHA-256 e 27 interações do card UnB `liberacoes-e-versoes-leitura`. Os replays já existentes confrontam conteúdo/histórico/árvore com a fonte parcialmente. **A mudança de título entre árvore e GET não aparece na gravação**, portanto é hipótese sintética, não paridade observada. Provas: **CLI↔anti-IES↔SQLite PASS sintético**; **cassette original PASS parcial** para respostas realmente gravadas, transição nova não observada; **treinamento/REA NOT_RUN**. Não houve produção, credenciais reais, subcódigo novo ou TLS enfraquecido.

**Cafezinho:** [contribuição #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6744520943): o carimbo de uma testemunha deve ser conferido também depois de cortar do índice para o corpo; analogia literária, não fato sobre o SEI.

**Próximo:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6744597213). Revisão 🦅 e integração 🐘/🦡 da PR #578 após conciliar a `main` e revalidar gates se o SHA mudar. Issue #60 continua aberta para outras lacunas efetivamente observáveis e paridade externa.
