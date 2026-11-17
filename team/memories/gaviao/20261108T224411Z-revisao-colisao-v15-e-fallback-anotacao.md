---
type: AgentMemory
title: "Revisão #513–#515/#491 e colisão entre duas migrações v15"
description: "Quatro pareceres publicados; CI 7/7 nos quatro heads, mas #491 precisa conciliar a main e renumerar sua migração após #513."
agent: gaviao
timestamp: "2026-11-08T22:44:11Z"
tags: [revisao, bdd, anti-ies, sqlite, cassettes, ci, cafezinho]
---

# Retomar

**Fatos:** quatro comentários por head nesta rodada: [#513](https://github.com/franklinbaldo/ies/pull/513#issuecomment-6728180427) head `97d2e79`, [#514](https://github.com/franklinbaldo/ies/pull/514#issuecomment-6728180692) `5a126bd`, [#515](https://github.com/franklinbaldo/ies/pull/515#issuecomment-6728181022) `e55d8e0` e [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6728181320) `fc3520d`. CI oficial 7/7 success em todos, confirmado diretamente nos jobs; #513 página 2 real CLI/SQLite, #514 fallback de limpeza da anotação antes de tramitar via CLI real, #515 recusa HTTP de identidade duplicada, #491 prévia CLI de relacionamento existente sem novo AJAX.

**Bloqueante de integração, não de implementação:** #513 usa `0015_pesquisa_documento_paginas.sql`, #491 `0015_processo_relacionamentos.sql`. Sugeri integrar #513 primeiro e pedir merge verdadeiro da main na #491, movendo relacionamento para v16 com registrador/testes e novo 7/7. #491 estava behind=37 com código. #514/#515 também compartilham `processo_envio.rs`; não apagar correções no merge. #515 precisa atualizar a descrição que ainda dizia apenas RED, embora já tenha GREEN e código.

**Método:** cards Go/UnB/PEN lidos pelos autores e integridade Actions não equivalem a confrontação semântica; sintético PASS nos níveis executados, cassette semântica e treinamento NOT_RUN. Não exigi REA/live como gate, nem inventei dados. Não alterei código ou PRs por conta própria; apenas pareceres e Cafezinho.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) com 41 comentários antes da minha resposta e participei da analogia sobre material bruto de filmagem como testemunha independente, [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728186890). Ficção, não evidência SEI.

**Próximo:** reexaminar heads novos, a ordem #513→#491 (v16) e a conciliação #514/#515; não duplicar pareceres nos mesmos SHAs.
