---
type: AgentMemory
title: "Editor #89: consultar observações de cassette antes de modelar rotas"
description: "Há cassettes de documento.salvar em sete instalações; a PR #458 usa documento_editar sintético, enquanto os cards registram documento_alterar, editor_montar e editor_salvar."
agent: polvo
timestamp: "2026-11-08T11:51:38Z"
tags: [documento, editor, anti-ies, cassette, evidencia]
---

# Achado verificado em 08/11/2026

- [Evidências de documento.salvar](../../../contracts/evidencias/documento-salvar.md) apontam cassettes da UnB, PEN1–PEN4, UFRJ e Goiás, já existentes antes da PR #458.
- Os cards [UnB](../../../cassettes/unb/documento-escritas-unb--36a904749e20.md), [PEN1](../../../cassettes/pen1/documento-escritas-pen1--a6cac1d02f3e.md) e [UFRJ](../../../cassettes/ufrj/documento-escritas-ufrj--2241d2a8f71b.md) registram `documento_alterar`, `editor_montar` e `editor_salvar`, inclusive POSTs, com ZIP, membro e hashes.
- Na [PR #458](https://github.com/franklinbaldo/ies/pull/458), `crates/anti-ies/src/editor_documento.rs` e `src/documento_salvar_cli/remoto.rs` usam `acao=documento_editar` e HTML/formulário sintéticos. São testes E2E sintéticos, **não** replay de cassette nem paridade observada.
- **Próxima fatia:** ler os cards e os membros originais verificados, identificar URLs/campos/efeitos observados, confrontar separadamente o leitor IES e a resposta anti-IES, e ajustar a vertical IES/iesei/SQLite/anti-IES. Usar fixture sintética apenas para lacunas e transições não observadas, **sem bloquear desenvolvimento** pela falta de captura. Não afirmar replay, hash conferido ou REA executado sem execução.
- O catálogo [docs/cassettes.md](../../../docs/cassettes.md) adverte: o corpus não preserva todos os corpos de POST; resposta HTTP, sozinha, não prova efeito de escrita. O teste deve verificar a releitura independente.
