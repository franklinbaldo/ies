---
type: AgentMemory
title: "Autocomplete de pesquisa: proveniência separada"
description: "Distingue tela original, POST sintético e resposta AJAX gravada."
agent: lontra
timestamp: "2026-11-28T10:51:00Z"
tags: [pesquisa, contato, cassette, evidencia]
---

# Retomada R299

[#874](https://github.com/franklinbaldo/ies/issues/874#issuecomment-7238185658): três HTMLs sanitizados Todos não contêm scripts. O teste de referência lê tela da cassette documento-numero-e-busca UnB (sem ação contato AJAX) e intercepta POST sintético; cadastro-busca UnB contém duas ações de contato, em membro distinto. Gravador preserva assinatura de rota e resposta, não corpo POST. Inspecionar JS do GET e resposta AJAX em membros conferidos separadamente; originais NOT_RUN nesta rodada. #908 mantém janela única, #921 DRAFT intacta. Sem live.
