---
type: AgentMemory
title: "O mesmo id no AJAX não comprova o mesmo vínculo"
description: "RED/GREEN da PR #542 impede sem_efeito falso quando tabela e complemento atribuem NUPs diferentes ao mesmo id; limites das três provas preservados."
agent: castor
timestamp: "2026-11-09T06:13:02Z"
tags: [relacionamentos, identidade, rust, ajax, anti-ies, cassette, bdd]
---

# Junta para a próxima rodada

**Fato técnico:** a [PR #542](https://github.com/franklinbaldo/ies/pull/542), BDD-258-01, corrige `prever`: se a tela associa o id de destino a NUP diferente do que o AJAX publica, não inferir `SemEfeito`. Usar a causa já existente `DestinoDaConsultaDivergente`. O [RED #42570917024](https://github.com/franklinbaldo/ies/actions/runs/42570917024) apresentou **1 falha e 2.129 acertos**; o [GREEN #42571284464](https://github.com/franklinbaldo/ies/actions/runs/42571284464) aprovou **7/7** no head `114b7c4e`. A PR segue Ready e sem merge no fechamento desta nota.

**Evidência delimitada:** o teste negativo é hipótese sintética de observações contraditórias, **não** ocorrência PEN1. [Inspeção #42570775631](https://github.com/franklinbaldo/ies/actions/runs/42570775631) verificou integridade dos dois cards (53 interações), sem confronto semântico completo. E2E anterior da CLI real↔anti-IES↔SQLite permaneceu verde; treinamento **NOT_RUN**, sem requisições externas.

**Retomar:** conferir integração da #542, depois procurar a primeira divergência de confronto independente com as respostas originais; não liberar `--confirmar` no release sem validação em treinamento. A conversa [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6733571131) ligou a contradição dos dois NUPs à imagem de um pino repetido em duas juntas: analogia, não nova prova do SEI.
