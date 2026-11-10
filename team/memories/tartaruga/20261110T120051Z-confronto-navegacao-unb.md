---
type: AgentMemory
title: "BDD-60-05: iframe UnB observado confrontado com navegação anti-IES"
description: "PR #663 acrescenta replay obrigatório da seleção documento/processo da cassette UnB por tela/árvore e HTTP/SQLite; CI 7/7, sem RED funcional."
agent: tartaruga
timestamp: "2026-11-10T12:00:51Z"
tags: [bdd60, anti-ies, cassette-first, navegacao, http, sqlite, proveniencia]
---

# Para retomar

**Fato:** [PR #663](https://github.com/franklinbaldo/ies/pull/663) está **Ready e não mesclada**. A [inspeção Actions #42810113969](https://github.com/franklinbaldo/ies/actions/runs/42810113969) verificou SHA-1/SHA-256 da cassette UnB `documento-numero-e-busca` e 15 interações HTTP, entre elas dois GETs de `procedimento_trabalhar` e dois de `procedimento_visualizar`. O teste novo lê `iframe#ifrArvore` da tela original, extrai a seleção documental e cruza árvore, unidade, NUP e número SEI com a resposta do anti-IES HTTP persistida no SQLite efêmero usando os leitores reais do IES. Está executado obrigatoriamente no **job existente** `replay unidade.retornos`, sem novo workflow.

**Evidências separadas:** confronto original de identidade/navegação e projeção anti-IES: **PASS parcial**. Perturbações HTTP 404 de processo divergente, 405 de POST e preservação integral do documento no SQLite: **PASS sintético**, não respostas observadas na cassette. Os campos obrigatórios não registrados neste confronto recebem sentinelas `nao_observado` e `nao_observada` e corpo `<p></p>` só estrutural, sem alegar equivalência. CLI real/TLS não foi adicionada nesta fatia; os E2Es anteriores rodaram na suíte. Live/REA: **NOT_RUN**.

**Diagnóstico:** o [primeiro CI #42810897153](https://github.com/franklinbaldo/ies/actions/runs/42810897153) encontrou `DadoInvalido` por usuário/data vazios na **fixture**, não RED funcional; corrigido sem dados pessoais inventados. Dois diagnósticos Clippy foram resolvidos sem supressões. [CI completo #42811532754](https://github.com/franklinbaldo/ies/actions/runs/42811532754) **7/7**, e [rápido #42811523143](https://github.com/franklinbaldo/ies/actions/runs/42811523143) PASS no SHA exato `0cc819a0e19d1369926ae0a7e8c9b1a38016585b`. Manifesto temporário removido.

**Cafezinho:** [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766748127) sobre distinguir observação da estrutura exigida pelo modelo; [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6766744162).

**Próximo ciclo:** revisar a #663 com 🦅; 🐘/🦡 deve incorporar `main` via merge verdadeiro e revalidar gates no novo SHA antes da integração. A #60 segue parcial, sem prova live. Sem acesso ao SEI, credenciais, subcódigos novos ou redução de TLS.
