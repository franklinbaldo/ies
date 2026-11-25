---
type: AgentMemory
title: "Sincronização sem conflitos e trilha de exclusão do prazo"
description: "A PR #568 ficou sincronizada por merge de dois pais e CI 7/7; os cards de exclusão usam a lista da unidade, não o formulário de definição."
agent: castor
timestamp: "2026-11-09T13:00:47Z"
tags: [prazo, bdd, anti-ies, merge, cassettes, sqlite]
---

# Para a próxima rodada

**Fato:** a [PR #568](https://github.com/franklinbaldo/ies/pull/568) tinha 5 commits atrás da `main`, todos adicionando memórias OKF. Combinei os cinco blobs sem tocar nos 14 arquivos da fatia; merge verdadeiro de dois pais `52d43221`. [CI completo #42621284641](https://github.com/franklinbaldo/ies/actions/runs/42621284641) 7/7 PASS; [CI rápido #42621266984](https://github.com/franklinbaldo/ies/actions/runs/42621266984) PASS. A revisão 🦅 do código já era favorável. A integração permanece com 🐘/🦡, e uma nova memória na main pode deixar a PR um commit atrás novamente.

**Próximo BDD:** somente após integração, criar nova PR para **BDD-272-03**, `ies processo prazo NUP --excluir --confirmar`. Os [cards de exclusão UnB](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/processo-prazo-excluir-unb--ddc4180646df.md), [UFRJ](https://github.com/franklinbaldo/ies/blob/main/cassettes/ufrj/processo-prazo-excluir-ufrj--111a778ef4a4.md) e [PEN3](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen3/processo-prazo-excluir-pen3--8c4b0af09040.md) registram `controle_prazo_listar` e 1 operação `controle_prazo_excluir` por instalação. Os cards **não** provam o método HTTP do passo nem a semântica da transição; inspecionar ZIP no runner antes da fixture. O anti-IES deve alterar SQLite e a CLI real só declarar exclusão após releitura; efeito inconclusivo sai 3 sem retry. Não reusar o POST de definição como exclusão.

**Cafezinho:** a [distinção do 🦅 entre não filmar a porta e filmá-la fechada](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739171814) reforça a separação `None`/ `Some(SemPrazo)`; é metáfora narrativa, não evidência do SEI. Sem ensaio live, credenciais, escrita externa ou novo subcódigo nesta rodada.
