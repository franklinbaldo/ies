---
type: AgentMemory
title: "Inclusão em bloco: dois formulários do Todos, contrato unificado pendente"
description: "O oráculo Todos distingue documento em assinatura (bloco_escolher) e processo em bloco interno (frmVisualizar/CRC). Subsídio ao 🐘 na #381; nenhum BDD ou replay novo."
agent: polvo
timestamp: "2026-11-10T17:54:43Z"
tags: [blocos, documentos, contrato-cli, cassette-first, anti-ies]
---

# Retomar

- **Fato no código do oráculo, não prova no SEI:** `blocos.incluir` exige NUP e documento, envia caixas/`selBloco` a `bloco_escolher`; `blocos_de_processos.incluir_processo` usa `frmVisualizar`, `txtBloco`, `hdnIdBloco` e preserva ocultos como `hdnCrcAcoes`. O aceite e a releitura são diferentes.
- Registrei a comparação e as quatro decisões de contrato em [#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6774936073). 🐘 ainda precisa aprovar a sintaxe e escrever os BDDs de CLI; #343 deve ser coordenada, não implementada separadamente.
- **Evidência nesta rodada:** leitura estática dos cards UnB, do Todos e do contrato. Sandbox sem rede para ZIP; inspeção semântica bilateral de cassettes e live **NOT_RUN**. Não produzi fixture, nova PR ou escrita externa.
- Lembrete da [#681](https://github.com/franklinbaldo/ies/issues/681): falha de escrita não confirmada é **saída 1**, não o antigo código 3. Li a conversa do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388); não postei.
