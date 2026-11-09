---
type: AgentMemory
title: "RED de identidade documental, revisão de projeções e dois heads verdes"
description: "Revisões #537/#535 com gates pendentes ou falhos, #536/#531 verdes; prova de identidade não substitui CLI nem confronto semântico."
agent: gaviao
timestamp: "2026-11-09T04:39:34.520Z"
tags: [revisao, bdd, red, anti-ies, cassettes, sqlite, ci, cafezinho]
---

# Retomar

**Fatos:** pareceres por head em [#536](https://github.com/franklinbaldo/ies/pull/536#issuecomment-6732376102) `7f06d1f` e [#531](https://github.com/franklinbaldo/ies/pull/531#issuecomment-6732376419) `ed3b656`, ambos com CI oficial **7/7**. A #536 melhora a recusa e a prévia pública de atribuição, sem habilitar `--confirmar`; a #531 corrige forma vazia do formulário PEN1 e preserva POST local seguido de releitura da CLI, mas **BDD-258-03 não concluiu a confirmação via CLI**.

Em [#535](https://github.com/franklinbaldo/ies/pull/535#issuecomment-6732376760) `80060c2`, CI **6/7**, Rustfmt e `clippy::assert_is_empty` em `documento_versoes.rs:278`; correção pontual, sem apagar teste. A [#537](https://github.com/franklinbaldo/ies/pull/537#issuecomment-6732377140) teve RED reproduzido no CI `42562601847` (esperado 400, obtido 200 quando `id_documento` conflita com `id_protocolo`); o novo head `664a433` implementa recusa e passou CI rápido, mas CI oficial ainda estava em andamento. Teste é HTTP/SQLite, não BDD-60-05 completo. Semântica cassette e treinamento seguem separados.

**Continuidade:** rever CI e head final da #537 e correção da #535; não repetir pareceres nos SHAs já cobertos. A #526 continua com migração SQLite **v18 concorrente**, precisa integrar `main`, mover a criação para **v19**, preservar navegação e obter novo 7/7; não reimplementar o E2E.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). A observação de 🦫 sobre **tabela efetivamente ausente após remoção** na cassette PEN1 é um contracampo útil: ausência observada é diferente de estado jamais observado. Não transformar a metáfora em evidência do SEI.
