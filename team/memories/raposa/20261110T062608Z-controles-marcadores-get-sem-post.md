---
type: AgentMemory
title: "GET de marcadores: controles oferecidos não comprovam POST"
description: "Na PR #644, E2E da CLI/anti-IES/SQLite confronta controles do GET; hashes UnB/GO passaram no Actions, sem replay semântico nem escrita. Clippy exigiu extrair o teste."
agent: raposa
timestamp: "2026-11-10T06:26:08Z"
tags: [ux, processo-marcar, anti-ies, bdd, cassettes, testes]
---

# Para retomar

**Fato:** a [PR #644](https://github.com/franklinbaldo/ies/pull/644) começou por um teste E2E dos controles `btnAdicionar`, `btnRemover` e `acaoRemover` do `andamento_marcador_gerenciar`. A projeção GET do anti-IES agora representa esses controles com argumento JS escapado, mas **não** oferece POST ou confirmação. O caso sem marcador continua 302 comprovadamente vazio, nunca HTML de tabela vazia. O primeiro CI parou em Rustfmt e, depois, Clippy apontou a extensão do teste e `if_not_else`; correções feitas sem desabilitar gates. Verificar o resultado final do [CI oficial #42768328211](https://github.com/franklinbaldo/ies/actions/runs/42768328211) e sincronizar a branch com a `main`.

**Cassettes:** a [inspeção #42767722412](https://github.com/franklinbaldo/ies/actions/runs/42767722412) verificou SHA-1/SHA-256 dos cards UnB/GO e 160 interações. É estrutura, **não** replay semântico de HTML ou causalidade de POST. Treinamento/live NOT_RUN, sem requisições ao SEI. BDD-255-01/02 continuam pendentes além deste recorte; `--confirmar` segue fechado. Não repetir o POST de efeito incerto da #264.

**Cafezinho:** reli e [comentei na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6760627379) a semelhança com o `btnPesquisar` ausente que o 🐙 observou na #640. Leitor que aceita uma página não demonstra que a interface projetada está completa. Isso orienta o teste, não certifica escrita.

**CONTINUAR:** conferir o CI da PR no SHA final, revisão 🦅 e integração 🐘/🦡; avançar BDD-255-01/02 com formulário de catálogo, envio único causal e releitura, sem novos subcódigos nem produção.
